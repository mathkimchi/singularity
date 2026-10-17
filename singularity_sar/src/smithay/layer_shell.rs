//! wlr-layer-shell: surfaces anchored to the output's edges (eg status bars, wallpapers).
//! They are not in the node tree: they're drawn under or over the root's content,
//! and their exclusive zones shrink the area the root's content gets.

use super::SmithayState;
use crate::runner::AppletRunner;
use smithay::{
    reexports::wayland_server::protocol::wl_output::WlOutput,
    utils::{Logical, Point, Rectangle, Size},
    wayland::shell::wlr_layer::{
        Anchor, ExclusiveZone, Layer, LayerSurface, LayerSurfaceCachedState, WlrLayerShellHandler,
        WlrLayerShellState,
    },
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{Style, UITree},
};

/// A layer surface after layout
#[derive(Debug)]
pub(super) struct LayerPlacement {
    surface: LayerSurface,
    layer: Layer,
    /// In logical units, in the output
    rect: Rectangle<i32, Logical>,
}

/// The edge that the exclusive zone is measured from, if the anchor allows one
fn exclusive_edge(state: &LayerSurfaceCachedState) -> Option<Anchor> {
    let anchor = state.anchor;
    if let Some(edge) = state.exclusive_edge
        && anchor.contains(edge)
    {
        return Some(edge);
    }
    let (horizontal, vertical) = (Anchor::LEFT | Anchor::RIGHT, Anchor::TOP | Anchor::BOTTOM);
    [
        (Anchor::TOP, horizontal),
        (Anchor::BOTTOM, horizontal),
        (Anchor::LEFT, vertical),
        (Anchor::RIGHT, vertical),
    ]
    .into_iter()
    .find(|&(edge, perpendicular)| {
        // the edge alone, or with both perpendicular edges
        anchor == edge || anchor == edge | perpendicular
    })
    .map(|(edge, _)| edge)
}

/// Where along one axis (`start`/`end` anchors) a surface goes in `bounds`, like wlroots
fn place_axis(
    bounds: (i32, i32),
    requested: i32,
    (start, end): (bool, bool),
    (margin_start, margin_end): (i32, i32),
) -> (i32, i32) {
    let (bounds_start, bounds_len) = bounds;
    let len = if requested == 0 {
        (bounds_len - margin_start - margin_end).max(1)
    } else {
        requested
    };
    let pos = match (start, end) {
        (true, true) => bounds_start + margin_start + (bounds_len - margin_start - margin_end - len) / 2,
        (true, false) => bounds_start + margin_start,
        (false, true) => bounds_start + bounds_len - len - margin_end,
        (false, false) => bounds_start + (bounds_len - len) / 2,
    };
    (pos, len)
}

impl SmithayState {
    pub(super) fn new_layer_shell_state(display_handle: &super::DisplayHandle) -> WlrLayerShellState {
        WlrLayerShellState::new::<AppletRunner>(display_handle)
    }

    /// The area left for the root's content, in px of the window
    pub fn content_area(&self) -> ([f64; 2], DisplayContainerSize) {
        let usable = self.usable_area;
        let to_px = |logical: i32| (f64::from(logical) * self.scale).round();
        (
            [to_px(usable.loc.x), to_px(usable.loc.y)],
            DisplayContainerSize::new(
                to_px(usable.size.w) as u32,
                to_px(usable.size.h) as u32,
            ),
        )
    }

    /// Places the layer surfaces and shrinks the usable area by their exclusive zones (like wlroots),
    /// telling each surface its size. Returns whether the usable area changed.
    ///
    /// TODO: one pass per layer, so a surface isn't shrunk by a later one in its own layer
    pub fn arrange_layers(&mut self) -> bool {
        let Some(mode) = self.output.current_mode() else {
            return false;
        };
        let full = Rectangle::from_size(self.to_logical(DisplayContainerSize::new(
            mode.size.w.cast_unsigned(),
            mode.size.h.cast_unsigned(),
        )));
        let mut usable = full;
        let mut placements = Vec::new();

        for layer in [Layer::Overlay, Layer::Top, Layer::Bottom, Layer::Background] {
            for surface in &self.layer_surfaces {
                let state = surface.with_cached_state(|state| *state);
                if state.layer != layer {
                    continue;
                }
                // `DontCare` surfaces ignore other zones
                let bounds = if state.exclusive_zone == ExclusiveZone::DontCare {
                    full
                } else {
                    usable
                };
                let anchor = state.anchor;
                let margin = state.margin;
                let (x, w) = place_axis(
                    (bounds.loc.x, bounds.size.w),
                    state.size.w,
                    (anchor.contains(Anchor::LEFT), anchor.contains(Anchor::RIGHT)),
                    (margin.left, margin.right),
                );
                let (y, h) = place_axis(
                    (bounds.loc.y, bounds.size.h),
                    state.size.h,
                    (anchor.contains(Anchor::TOP), anchor.contains(Anchor::BOTTOM)),
                    (margin.top, margin.bottom),
                );

                if let ExclusiveZone::Exclusive(zone) = state.exclusive_zone
                    && let Some(edge) = exclusive_edge(&state)
                {
                    let zone = zone as i32;
                    if edge == Anchor::TOP {
                        let taken = (zone + margin.top).min(usable.size.h);
                        usable.loc.y += taken;
                        usable.size.h -= taken;
                    } else if edge == Anchor::BOTTOM {
                        usable.size.h -= (zone + margin.bottom).min(usable.size.h);
                    } else if edge == Anchor::LEFT {
                        let taken = (zone + margin.left).min(usable.size.w);
                        usable.loc.x += taken;
                        usable.size.w -= taken;
                    } else {
                        usable.size.w -= (zone + margin.right).min(usable.size.w);
                    }
                }

                placements.push(LayerPlacement {
                    surface: surface.clone(),
                    layer,
                    rect: Rectangle::new(Point::from((x, y)), Size::from((w, h))),
                });
            }
        }

        for placement in &placements {
            placement
                .surface
                .with_pending_state(|state| state.size = Some(placement.rect.size));
            placement.surface.send_pending_configure();
        }
        self.layer_placements = placements;

        let changed = self.usable_area != usable;
        self.usable_area = usable;
        changed
    }

    /// Drawn in the window's px; the textures are absolute in the container they're put in.
    /// `below` is the background and bottom layers, otherwise top and overlay.
    pub fn layer_surface_elements(
        &self,
        below: bool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<UITree> {
        // `layer_placements` is topmost first
        self.layer_placements
            .iter()
            .rev()
            .filter(|placement| matches!(placement.layer, Layer::Background | Layer::Bottom) == below)
            .map(|placement| {
                let layers = Self::surface_tree_layers(placement.surface.wl_surface(), placement.rect.loc);
                UITree::new_container_styled(
                    Style::full_size(),
                    self.layers_to_textures(layers, None, device, queue),
                )
            })
            .collect()
    }

    /// Whether a surface is a layer surface that has been placed
    pub(super) fn is_layer_surface(&self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) -> bool {
        self.layer_shell_state
            .layer_surfaces()
            .any(|layer_surface| layer_surface.wl_surface() == surface)
    }
}

impl AppletRunner {
    /// Called when a layer surface commits: the first commit makes it placed, and its state may have changed
    pub(super) fn layer_surface_committed(&mut self, surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) {
        let state = &mut self.smithay_state;
        if !state.is_layer_surface(surface) {
            return;
        }
        if !state
            .layer_surfaces
            .iter()
            .any(|layer_surface| layer_surface.wl_surface() == surface)
        {
            let Some(layer_surface) = state
                .layer_shell_state
                .layer_surfaces()
                .find(|layer_surface| layer_surface.wl_surface() == surface)
            else {
                return;
            };
            state.layer_surfaces.push(layer_surface);
        }
        self.relayout();
    }

    /// Arranges the layer surfaces, and tells the root if its area changed
    pub(crate) fn relayout(&mut self) {
        if self.smithay_state.arrange_layers() {
            self.send_content_size();
        }
        self.needs_redraw = true;
    }
}

impl WlrLayerShellHandler for AppletRunner {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.smithay_state.layer_shell_state
    }

    /// Placed (and told its size) on its first commit
    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        _output: Option<WlOutput>,
        _layer: Layer,
        _namespace: String,
    ) {
        // there is only one output
        self.smithay_state.output.enter(surface.wl_surface());
    }

    fn layer_destroyed(&mut self, surface: LayerSurface) {
        self.smithay_state
            .layer_surfaces
            .retain(|layer_surface| *layer_surface != surface);
        self.relayout();
    }
}
