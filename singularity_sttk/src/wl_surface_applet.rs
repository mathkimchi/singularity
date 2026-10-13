use crate::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{NodularRunnerHook, StandardApplet},
    standard_keybinds::handle_standard_keybinds,
};
use singularity_common::sap::packets::{StandardEvent, WlSurfaceEvent, WlSurfaceId};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{UIElement, UITree},
    ui_event::UIEvent,
};

pub struct WlSurfaceApplet {
    surface_id: WlSurfaceId,
    wl_event_queue: calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
    hook: Box<dyn NodularRunnerHook + 'static>,
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
        hook: Box<dyn NodularRunnerHook + 'static>,
    ) -> Self {
        Self {
            surface_id,
            wl_event_queue,
            hook,
        }
    }
}
impl StandardApplet for WlSurfaceApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        match standard_event {
            StandardEvent::UIEvent(ui_event) => {
                if handle_standard_keybinds(&ui_event, &self.hook) {
                    return;
                }

                let wl_event = match ui_event {
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
                };
                self.wl_event_queue
                    .send((self.surface_id, wl_event))
                    .unwrap();
            }
            StandardEvent::FocusChanged(_) => {}
            StandardEvent::Highlighted(_) => {}
            StandardEvent::CloseRequest => {}
            StandardEvent::SurfaceDamageAck => {}
            StandardEvent::TreeviewDamageAck => {}
            StandardEvent::WlSurfaceRegistered {
                surface_id: _,
                wl_event_queue: _,
            } => {}
            // the parent already focused this
            StandardEvent::FocusWlSurface(_) => {}
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                if surface_id == self.surface_id {
                    self.hook.close();
                }
            }
        }
    }

    fn get_window_standard_applet(&self, _container_size: DisplayContainerSize) -> UITree {
        UITree::leaf(UIElement::Subsurface(self.surface_id.as_u64()))
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        dbg!("TODO: proper wl title");
        singularity_common::utils::tree::world_tree::WorldTree::Base("Wl Surface".to_string())
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        singularity_common::utils::tree::world_tree::WorldTreePath::new_into()
    }

    fn contains_wl_surface(&self, surface_id: WlSurfaceId) -> bool {
        self.surface_id == surface_id
    }

    fn focused_wl_surface(&self) -> Option<WlSurfaceId> {
        Some(self.surface_id)
    }
}
