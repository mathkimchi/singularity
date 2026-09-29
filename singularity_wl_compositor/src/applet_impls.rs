use crate::WaylandApplet;
use singularity_common::{
    sap::packets::StandardEvent,
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use singularity_sttk::{
    nodular_applet::StandardApplet, standard_keybinds::handle_standard_keybinds,
};
use sonamu_ui::display_units::DisplayContainerSize;

impl StandardApplet for WaylandApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        match standard_event {
            StandardEvent::UIEvent(ui_event) => {
                if handle_standard_keybinds(&ui_event, &self.hook) {
                    return;
                }

                self.input_sender.send(ui_event).unwrap();

                self.hook.damage_window();
                // self.hook.damage_treeview();
            }
            StandardEvent::FocusChanged(_) => todo!(),
            StandardEvent::Highlighted(_) => todo!(),
            StandardEvent::CloseRequest => todo!(),
            StandardEvent::SurfaceDamageAck => todo!(),
            StandardEvent::TreeviewDamageAck => todo!(),
            StandardEvent::WlSurfaceRegistered { .. } => todo!(),
        }
    }

    fn get_window_standard_applet(
        &self,
        _container_size: DisplayContainerSize,
    ) -> sonamu_ui::ui_element::UIElement {
        if let Ok(image) = self.image.lock()
            && let Some(ref image) = *image
        {
            sonamu_ui::ui_element::UIElement::Image(image.clone())
        } else {
            sonamu_ui::ui_element::UIElement::from("Wayland app loading...".to_string())
        }
    }

    fn get_treeview(&self) -> WorldTree<String> {
        // TODO: actual name
        WorldTree::Base("Wayland App".to_string())
    }

    fn get_focus_path(&self) -> WorldTreePath {
        WorldTreePath::new_into()
    }
}
