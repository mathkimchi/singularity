use singularity_common::sap::packets::{StandardEvent, WlSurfaceId};
use sonamu_ui::{display_units::DisplayContainerSize, ui_element::UIElement};

use crate::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{NodularRunnerHook, StandardApplet},
};

pub struct WlSurfaceApplet {
    surface_id: WlSurfaceId,
    hook: Box<dyn NodularRunnerHook + 'static>,
}
impl CreatableNodularApplet<WlSurfaceId> for WlSurfaceApplet {
    fn new(surface_id: WlSurfaceId, hook: Box<dyn NodularRunnerHook + 'static>) -> Self {
        Self { surface_id, hook }
    }
}
impl StandardApplet for WlSurfaceApplet {
    fn handle_standard_event(&mut self, _standard_event: StandardEvent) {
        dbg!("TODO: handle event wl surface applet");
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
