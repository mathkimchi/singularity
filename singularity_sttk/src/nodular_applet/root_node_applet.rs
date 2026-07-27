use crate::nodular_applet::{
    AppletSpawner, NodularApplet, NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet,
};
use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use sonamu_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
    ui_element::UIElement,
};
use std::{collections::BTreeMap, sync::RwLock};

/// Holds the recursive_node_applet, is held by a Basic Applet runner (applet runner).
pub struct RootNodeApplet {
    /// NOTE: this could be generic
    applet: RecursiveNodeApplet,
    // window: Arc<Mutex<UIElement>>,
    // hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
    // applet_spawner_registry: RwLock<BTreeMap<String, AppletSpawner>>,
}
impl RootNodeApplet {
    pub fn new(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        struct InnerHook {
            outer_hook: Box<dyn BasicRunnerHook>,
            applet_spawner_registry: RwLock<BTreeMap<String, AppletSpawner>>,
        }
        impl BasicRunnerHook for InnerHook {
            // fn update_display(&self, display: &sonamu_ui::ui_element::UIElement) {
            //     self.outer_hook.update_display(display);
            // }

            fn close(&self) {
                self.outer_hook.close();
            }

            fn damage_window(&self) {
                // REVIEW
                self.outer_hook.damage_window();
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, _treeview: &sonamu_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: super::NodularAppletInitializer) {
                todo!()
            }

            fn damage_treeview(&self) {
                // REVIEW
                self.outer_hook.damage_window();
            }

            fn change_focus(
                &self,
                _operation: singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation,
            ) {
                // REVIEW: do I need to do anything here?
            }

            fn register_applet_spawner(&self, name: String, applet_spawner: AppletSpawner) {
                self.applet_spawner_registry
                    .write()
                    .unwrap()
                    .insert(name, applet_spawner);
            }
            fn get_applet_spawners(
                &self,
            ) -> BTreeMap<String, AppletSpawner> {
                self.applet_spawner_registry.read().unwrap().clone()
            }
            fn find_applet_spawner(&self, name: String) -> Option<AppletSpawner> {
                self.applet_spawner_registry
                    .read()
                    .unwrap()
                    .get(&name)
                    .cloned()
            }
        }

        let applet_spawner_registry = RwLock::new(applet_spawner_registry);
        let inner_hook = InnerHook {
            outer_hook: hook,
            applet_spawner_registry,
        };

        Self {
            applet: inner_initializer(Box::new(inner_hook)),
        }
    }

    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook| Self::new(inner_initializer, applet_spawner_registry, hook)
    }

    fn get_treeview_display(&self) -> UIElement {
        let focused_path = self.applet.get_focus_path();
        let treeview = self.applet.get_treeview();

        println!("Focused path: {focused_path:?}");

        // CharGrid::from(treeview.outer_world_to_string())
        //     .element()
        //     .bordered(Color::LIGHT_GREEN)
        //     .fill_bg(Color::BLACK)

        // This is the horizontal split
        // UIElement::combine_displays((0..(focused_path.0.len() + 1)).map(|world_level_index| {
        UIElement::combine_displays((0..focused_path.0.len()).map(|world_level_index| {
            let world_path = WorldTreePath(
                focused_path.0[0..world_level_index]
                    .to_vec()
                    .into_boxed_slice(),
            );
            UIElement::from(
                treeview
                    .safe_get(&world_path)
                    .unwrap()
                    .outer_world_to_string(focused_path.0.get(world_level_index).cloned()),
            )
            .bordered(Color::LIGHT_GREEN)
        }))
        .fill_bg(Color::BLACK)
    }
}
impl BasicApplet for RootNodeApplet {
    fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
        self.applet.handle_ui_event(ui_event);
    }

    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement {
        UIElement::Container(vec![
            self.get_treeview_display()
                .contain(DisplayArea::new((0.0, 0.0), (0.2, 1.0))),
            self.applet
                .layout_builder()
                .bordered(Color::LIGHT_GREEN)
                .contained(DisplayArea::new((0.2, 0.0), (1.0, 1.0)))
                .get_ui_element(container_size),
        ])
    }
}
