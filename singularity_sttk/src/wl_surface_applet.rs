use crate::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{AppletContext, NodeRequest, StandardApplet},
    standard_keybinds::standard_keybind_request,
};
use singularity_common::{
    sap::packets::{StandardEvent, WlSurfaceEvent, WlSurfaceId},
    utils::tree::world_tree::WorldTree,
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{UIElement, UITree},
    ui_event::UIEvent,
};

/// Shows a wl surface (a wl client's toplevel) and forwards input to it.
///
/// The tree's holder tracks which node shows which surface,
/// and closes the node once the surface is destroyed.
pub struct WlSurfaceApplet {
    surface_id: WlSurfaceId,
    wl_event_queue: calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
    hook: AppletContext,
    /// The toplevel's title, empty until the client sets one
    title: String,
}
impl
    CreatableNodularApplet<(
        WlSurfaceId,
        calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
    )> for WlSurfaceApplet
{
    fn new(
        (surface_id, wl_event_queue): (
            WlSurfaceId,
            calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
        ),
        hook: AppletContext,
    ) -> Self {
        Self {
            surface_id,
            wl_event_queue,
            hook,
            title: String::new(),
        }
    }
}
impl StandardApplet for WlSurfaceApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        let ui_event = match standard_event {
            StandardEvent::UIEvent(ui_event) => ui_event,
            // titles of unregistered surfaces are broadcast to every node, so check the id
            StandardEvent::WlSurfaceTitle(surface_id, title) => {
                if surface_id == self.surface_id {
                    self.title = title;
                    self.hook.damage_treeview();
                }
                return;
            }
            _ => return,
        };

        let wl_event = match standard_keybind_request(&ui_event) {
            // the tile closes once the client destroys its toplevel (`WlSurfaceDestroyed`),
            // so the client may refuse (eg to ask about unsaved changes)
            Some(NodeRequest::Close) => WlSurfaceEvent::Close,
            Some(request) => {
                self.hook.request(request);
                return;
            }
            None => match ui_event {
                UIEvent::Key {
                    raw_keycode,
                    pressed,
                    ..
                } => WlSurfaceEvent::Key {
                    // evdev -> xkb
                    keycode: raw_keycode + 8,
                    pressed,
                },
                UIEvent::Mouse(mouse_event) => WlSurfaceEvent::Mouse(mouse_event),
                UIEvent::WindowResized(new_size) => WlSurfaceEvent::Resize(new_size),
            },
        };
        // the compositor may be gone while shutting down
        let _ = self.wl_event_queue.send((self.surface_id, wl_event));
    }

    /// Asks the client; the tile closes once it destroys its toplevel
    fn close_requested(&mut self) -> bool {
        let _ = self
            .wl_event_queue
            .send((self.surface_id, WlSurfaceEvent::Close));
        false
    }

    fn get_window_standard_applet(&self, _container_size: DisplayContainerSize) -> UITree {
        UITree::leaf(UIElement::Subsurface(self.surface_id.as_u64()))
    }

    fn get_treeview(&self) -> WorldTree<String> {
        WorldTree::Base(if self.title.is_empty() {
            "Wl Surface".to_string()
        } else {
            self.title.clone()
        })
    }
}
