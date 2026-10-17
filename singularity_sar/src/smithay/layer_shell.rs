//! `wlr-layer-shell`: surfaces outside the node tree (eg a status bar), anchored to the output's edges.
//!
//! The background and bottom layers are drawn under the root's content, the top and overlay layers over it.
//! Exclusive zones shrink the root's content.

use super::SmithayState;
use crate::runner::AppletRunner;
use smithay::{
    reexports::wayland_server::protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    utils::{Logical, Rectangle, Size},
    wayland::{
        compositor::with_states,
        shell::wlr_layer::{
            Anchor, ExclusiveZone, Layer as WlrLayer, LayerSurface, LayerSurfaceCachedState,
            LayerSurfaceData, WlrLayerShellHandler, WlrLayerShellState,
        },
    },
};
use sonamu_ui::{display_units::DisplayContainerSize, ui_element::UITree};

/// A layer surface and where it goes, in logical units from the output's top-left
struct Placed {
    surface: LayerSurface,
    layer: WlrLayer,
    area: Rectangle<i32, Logical>,
}

struct Arrangement {
    /// Top-most layer first
    placed: Vec<Placed>,
    /// What's left of the output after the exclusive zones
    usable: Rectangle<i32, Logical>,
}

/// What's drawn for the layer surfaces, back to front
pub(super) struct LayerScene {
    /// Background and bottom layers
    pub below: Vec<UITree>,
    /// Top and overlay layers
    pub above: Vec<UITree>,
}

/// Position and size along one axis within `origin..origin + length`
fn place_axis(
    [start, end]: [bool; 2],
    size: i32,
    [margin_start, margin_end]: [i32; 2],
    origin: i32,
    length: i32,
) -> (i32, i32) {
    // a size of 0 stretches between the margins (the protocol requires both anchors)
    let stretched = size == 0;
    let size = if stretched {
        (length - margin_start - margin_end).max(0)
    } else {
        size
    };
    let loc = if stretched || (start && !end) {
        origin + margin_start
    } else if end && !start {
        origin + length - size - margin_end
    } else {
        origin + (length - size) / 2
    };
    (loc, size)
}

fn place(cached: &LayerSurfaceCachedState, bounds: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
    let (anchor, margin) = (cached.anchor, cached.margin);
    let (x, w) = place_axis(
        [anchor.contains(Anchor::LEFT), anchor.contains(Anchor::RIGHT)],
        cached.size.w,
        [margin.left, margin.right],
        bounds.loc.x,
        bounds.size.w,
    );
    let (y, h) = place_axis(
        [anchor.contains(Anchor::TOP), anchor.contains(Anchor::BOTTOM)],
        cached.size.h,
        [margin.top, margin.bottom],
        bounds.loc.y,
        bounds.size.h,
    );
    Rectangle::new((x, y).into(), (w, h).into())
}

/// The edge an exclusive zone applies to: the explicit one, or the only edge
/// (optionally with both perpendicular ones) the surface is anchored to
fn exclusive_edge(anchor: Anchor, explicit: Option<Anchor>) -> Option<Anchor> {
    if explicit.is_some() {
        return explicit;
    }
    let horizontal = Anchor::LEFT | Anchor::RIGHT;
    let vertical = Anchor::TOP | Anchor::BOTTOM;
    [
        (Anchor::TOP, horizontal),
        (Anchor::BOTTOM, horizontal),
        (Anchor::LEFT, vertical),
        (Anchor::RIGHT, vertical),
    ]
    .into_iter()
    .find_map(|(edge, perpendicular)| {
        (anchor == edge || anchor == edge | perpendicular).then_some(edge)
    })
}

/// Takes `amount` off `edge` of `usable`
fn shrink(usable: &mut Rectangle<i32, Logical>, edge: Anchor, amount: i32) {
    if edge == Anchor::TOP {
        usable.loc.y += amount;
        usable.size.h -= amount;
    } else if edge == Anchor::BOTTOM {
        usable.size.h -= amount;
    } else if edge == Anchor::LEFT {
        usable.loc.x += amount;
        usable.size.w -= amount;
    } else if edge == Anchor::RIGHT {
        usable.size.w -= amount;
    }
    usable.size = (usable.size.w.max(0), usable.size.h.max(0)).into();
}

/// Layer surfaces are only arranged once they have their first configure (or are committing now to get it),
/// since before that they haven't set their anchor and size.
fn is_configured(surface: &LayerSurface, committing: Option<&WlSurface>) -> bool {
    committing == Some(surface.wl_surface())
        || with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .is_some_and(|data| data.lock().unwrap().initial_configure_sent)
        })
}

impl SmithayState {
    /// The output, in logical units
    fn output_area(&self) -> Rectangle<i32, Logical> {
        let size = self.output.current_mode().map_or_else(Size::default, |mode| {
            mode.size.to_f64().to_logical(self.scale).to_i32_round()
        });
        Rectangle::from_size(size)
    }

    /// Like wlroots: layers from the top, each shrinking the usable area for the ones placed after it.
    /// Surfaces with `ExclusiveZone::DontCare` ignore the usable area.
    fn arrange(&self, committing: Option<&WlSurface>) -> Arrangement {
        let output = self.output_area();
        let mut usable = output;
        let surfaces: Vec<_> = self
            .layer_shell_state
            .layer_surfaces()
            .filter(|surface| surface.alive() && is_configured(surface, committing))
            .collect();

        let mut placed = Vec::new();
        for layer in [
            WlrLayer::Overlay,
            WlrLayer::Top,
            WlrLayer::Bottom,
            WlrLayer::Background,
        ] {
            for surface in &surfaces {
                let cached = surface.with_cached_state(|cached| *cached);
                if cached.layer != layer {
                    continue;
                }
                let bounds = if cached.exclusive_zone == ExclusiveZone::DontCare {
                    output
                } else {
                    usable
                };
                let area = place(&cached, bounds);
                if let ExclusiveZone::Exclusive(zone) = cached.exclusive_zone
                    && let Some(edge) = exclusive_edge(cached.anchor, cached.exclusive_edge)
                {
                    // the zone is from the edge of the output, so it includes the margin
                    let margin = if edge == Anchor::TOP {
                        cached.margin.top
                    } else if edge == Anchor::BOTTOM {
                        cached.margin.bottom
                    } else if edge == Anchor::LEFT {
                        cached.margin.left
                    } else {
                        cached.margin.right
                    };
                    shrink(&mut usable, edge, zone.cast_signed() + margin);
                }
                placed.push(Placed {
                    surface: surface.clone(),
                    layer,
                    area,
                });
            }
        }
        Arrangement { placed, usable }
    }

    /// Tells each layer surface its size (the initial configure is `committing`'s first commit).
    /// Call when anything affecting the arrangement changes.
    pub(super) fn arrange_layers(&self, committing: Option<&WlSurface>) {
        for Placed { surface, area, .. } in self.arrange(committing).placed {
            surface.with_pending_state(|state| state.size = Some(area.size));
            surface.send_pending_configure();
        }
    }

    /// Where the root's content goes (the window minus the exclusive zones): its origin and size in window px
    pub(super) fn main_area(&self, window: DisplayContainerSize) -> ([u32; 2], DisplayContainerSize) {
        let usable = self.arrange(None).usable;
        let output = self.output_area();
        let px = |logical: i32| (f64::from(logical) * self.scale).round().max(0.0) as u32;
        let [left, top] = [px(usable.loc.x), px(usable.loc.y)];
        let right = px(output.size.w - usable.loc.x - usable.size.w);
        let bottom = px(output.size.h - usable.loc.y - usable.size.h);
        (
            [left, top],
            DisplayContainerSize::new(
                window.width.saturating_sub(left + right),
                window.height.saturating_sub(top + bottom),
            ),
        )
    }

    pub(super) fn layer_scene(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> LayerScene {
        let mut scene = LayerScene {
            below: Vec::new(),
            above: Vec::new(),
        };
        // back to front
        for Placed { surface, layer, area } in self.arrange(None).placed.into_iter().rev() {
            let elements = Self::surface_tree_layers(surface.wl_surface(), area.loc)
                .iter()
                .filter_map(|layer| self.layer_element(layer, device, queue))
                .collect::<Vec<_>>();
            if matches!(layer, WlrLayer::Background | WlrLayer::Bottom) {
                scene.below.extend(elements);
            } else {
                scene.above.extend(elements);
            }
        }
        scene
    }
}

impl WlrLayerShellHandler for AppletRunner {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.smithay_state.layer_shell_state
    }

    /// The single output is the Sonamu window.
    /// It is configured on its first commit, once it has set its anchor and size.
    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        _output: Option<WlOutput>,
        _layer: WlrLayer,
        _namespace: String,
    ) {
        self.smithay_state.output.enter(surface.wl_surface());
    }

    fn layer_destroyed(&mut self, _surface: LayerSurface) {
        // frees its exclusive zone
        self.smithay_state.arrange_layers(None);
        self.needs_redraw = true;
    }
}
