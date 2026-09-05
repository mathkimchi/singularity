use crate::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{NodularRunnerHook, StandardApplet},
    standard_keybinds::handle_standard_keybinds,
};
use singularity_common::sap::packets::{StandardEvent, WlSurfaceId};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::UIElement,
    ui_event::{Key, UIEvent},
};

/// NOTE: ignores modifiers
/// I'm too tired for ts
/// https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
fn key_to_keycode(key: Key) -> Option<u32> {
    Some(include!(concat!(env!("OUT_DIR"), "/keycode_matches.rs")) + 8)
}

pub struct WlSurfaceApplet {
    surface_id: WlSurfaceId,
    key_event_queue: calloop::channel::Sender<(WlSurfaceId, u32)>,
    hook: Box<dyn NodularRunnerHook + 'static>,
}
impl CreatableNodularApplet<(WlSurfaceId, calloop::channel::Sender<(WlSurfaceId, u32)>)>
    for WlSurfaceApplet
{
    fn new(
        (surface_id, key_event_queue): (WlSurfaceId, calloop::channel::Sender<(WlSurfaceId, u32)>),
        hook: Box<dyn NodularRunnerHook + 'static>,
    ) -> Self {
        Self {
            surface_id,
            key_event_queue,
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

                match ui_event {
                    UIEvent::KeyPress(key, _key_modifiers) => {
                        if let Some(keycode) = key_to_keycode(key) {
                            self.key_event_queue
                                .send((self.surface_id, keycode))
                                .unwrap();

                            // self.hook.damage_window();
                        }
                    }
                    UIEvent::MousePress(_, _) => {}
                    UIEvent::WindowResized(_) => {}
                }
            }
            StandardEvent::FocusChanged(_) => {}
            StandardEvent::Highlighted(_) => {}
            StandardEvent::CloseRequest => {}
            StandardEvent::SurfaceDamageAck => {}
            StandardEvent::TreeviewDamageAck => {}
            StandardEvent::WlSurfaceRegistered {
                surface_id: _,
                key_event_queue: _,
            } => {}
        }
    }

    fn get_window_standard_applet(&self, _container_size: DisplayContainerSize) -> UIElement {
        UIElement::Subsurface(self.surface_id.as_u64())
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        dbg!("TODO: proper wl title");
        singularity_common::utils::tree::world_tree::WorldTree::Base("Wl Surface".to_string())
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        singularity_common::utils::tree::world_tree::WorldTreePath::new_into()
    }
}
