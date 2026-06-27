use crate::WaylandApplet;
use singularity_common::utils::tree::world_tree::{WorldTree, WorldTreePath};
use singularity_sar::applet::BasicApplet;
use singularity_sttk::{
    nodular_applet::NodularApplet, standard_keybinds::handle_standard_keybinds,
};

impl BasicApplet for WaylandApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

        self.input_sender.send(ui_event).unwrap();

        self.hook.damage_window();
        // self.hook.damage_treeview();
    }

    fn get_window(&self) -> singularity_ui::ui_element::UIElement {
        if let Ok(image) = self.image.lock()
            && let Some(ref image) = *image
        {
            singularity_ui::ui_element::UIElement::Image(image.clone())
        } else {
            singularity_ui::ui_element::UIElement::from("Wayland app loading...".to_string())
        }
    }
}
impl NodularApplet for WaylandApplet {
    fn handle_nodular_event(
        &mut self,
        _nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        todo!()
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        // TODO: actual name
        WorldTree::Base("Wayland App".to_string())
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        WorldTreePath::new_into()
    }
}
