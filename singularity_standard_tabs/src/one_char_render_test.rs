use singularity_common::{sap::packets::StandardEvent, utils::tree::world_tree::WorldTreePath};
use singularity_sttk::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer, StandardApplet,
        recursive_node_applet::RecursiveNodeApplet,
    },
};
use sonamu_ui::{
    ui_element::CharGrid,
    ui_event::{KeyTrait, UIEvent},
};

pub struct OneCharRenderTestApplet(
    char,
    Box<dyn singularity_sttk::nodular_applet::NodularRunnerHook + 'static>,
);
impl CreatableNodularApplet<()> for OneCharRenderTestApplet {
    fn new(
        (): (),
        hook: Box<dyn singularity_sttk::nodular_applet::NodularRunnerHook + 'static>,
    ) -> Self {
        Self(' ', hook)
    }
}
impl OneCharRenderTestApplet {
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct Spawner;
        impl AppletSpawnerTrait for Spawner {
            fn create_initializer(&self, _args: &[&str]) -> Option<NodularAppletInitializer> {
                Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
                    OneCharRenderTestApplet::get_boxed_initiator(()),
                ))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(Spawner)
    }
}
impl StandardApplet for OneCharRenderTestApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        match standard_event {
            StandardEvent::UIEvent(ui_event) => {
                if let UIEvent::KeyPress(key, _) = ui_event
                    && let Some(c) = key.to_char()
                {
                    self.0 = c;
                    self.1.damage_window();
                    self.1.damage_treeview();
                }
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
        _container_size: sonamu_ui::display_units::DisplayContainerSize,
    ) -> sonamu_ui::ui_element::UIElement {
        CharGrid::from(self.0.to_string().as_str()).element()
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.0.to_string())
    }

    fn get_focus_path(&self) -> WorldTreePath {
        WorldTreePath::new_into()
    }
}
