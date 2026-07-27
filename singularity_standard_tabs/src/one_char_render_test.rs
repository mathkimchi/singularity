use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::BasicApplet;
use singularity_sttk::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        AppletSpawner, AppletSpawnerTrait, NodularApplet, NodularAppletInitializer,
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
impl BasicApplet for OneCharRenderTestApplet {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        if let UIEvent::KeyPress(key, _) = ui_event
            && let Some(c) = key.to_char()
        {
            self.0 = c;
            self.1.damage_window();
            self.1.damage_treeview();
        }
    }

    fn get_window(
        &self,
        _container_size: sonamu_ui::display_units::DisplayContainerSize,
    ) -> sonamu_ui::ui_element::UIElement {
        CharGrid::from(self.0.to_string().as_str()).element()
    }
}
impl NodularApplet for OneCharRenderTestApplet {
    fn handle_nodular_event(
        &mut self,
        _nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.0.to_string())
    }

    fn get_focus_path(&self) -> WorldTreePath {
        WorldTreePath::new_into()
    }
}
