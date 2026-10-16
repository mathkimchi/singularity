use singularity_common::sap::packets::StandardEvent;
use singularity_sttk::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        AppletContext, AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer, StandardApplet,
    },
};
use sonamu_ui::{
    ui_element::CharGrid,
    ui_event::{KeyTrait, UIEvent},
};

pub struct OneCharRenderTestApplet(char, AppletContext);
impl CreatableNodularApplet<()> for OneCharRenderTestApplet {
    fn new((): (), hook: AppletContext) -> Self {
        Self(' ', hook)
    }
}
impl OneCharRenderTestApplet {
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct Spawner;
        impl AppletSpawnerTrait for Spawner {
            fn create_initializer(&self, _args: &[&str]) -> Option<NodularAppletInitializer> {
                Some(OneCharRenderTestApplet::get_boxed_initiator(()))
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
                if let UIEvent::Key {
                    symbol: Some(key),
                    pressed: true,
                    ..
                } = ui_event
                    && let Some(c) = key.to_char()
                {
                    self.0 = c;
                    self.1.damage_window();
                    self.1.damage_treeview();
                }
            }
            // looks the same either way
            StandardEvent::FocusChanged(_) | StandardEvent::Highlighted(_) => {}
            // nothing to ask about first
            StandardEvent::CloseRequest => self.1.close(),
            // handled by the applet's client
            StandardEvent::DamageAck(_)
            | StandardEvent::Pluck(_)
            | StandardEvent::Place(..)
            | StandardEvent::Focus(_) => {}
            // doesn't show wl surfaces (only the root gets these)
            StandardEvent::WlSurfaceRegistered { .. } | StandardEvent::FocusWlSurface(_) => {}
            StandardEvent::WlSurfaceDestroyed(_) => {}
        }
    }

    fn get_window_standard_applet(
        &self,
        _container_size: sonamu_ui::display_units::DisplayContainerSize,
    ) -> sonamu_ui::ui_element::UITree {
        CharGrid::from(self.0.to_string().as_str()).element().into()
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.0.to_string())
    }
}
