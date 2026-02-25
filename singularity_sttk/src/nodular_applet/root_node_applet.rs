use crate::nodular_applet::{
    NodularApplet, NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet,
};
use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::{
    color::Color,
    display_units::DisplayArea,
    ui_element::{CharGrid, UIElement},
};

/// Holds a nodular applet but only supports Basic operations.
/// NOTE: This is really for debugging; for the actual, I'll implement RootNodeApplet.
pub struct NodularHolderApplet<InnerApplet: NodularApplet> {
    inner_applet: InnerApplet,
}
impl<InnerApplet: NodularApplet> NodularHolderApplet<InnerApplet> {
    pub fn new(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> InnerApplet,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        struct InnerHook {
            outer_hook: Box<dyn BasicRunnerHook>,
        }
        impl BasicRunnerHook for InnerHook {
            // fn update_display(&self, display: &singularity_ui::ui_element::UIElement) {
            //     self.outer_hook.update_display(display);
            // }

            fn close(&self) {
                self.outer_hook.close();
            }

            fn damage_window(&self) {
                self.outer_hook.damage_window();
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, _treeview: &singularity_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: Box<super::NodularAppletInitializer>) {}

            fn damage_treeview(&self) {
                todo!()
            }

            fn focus_out(&self) {}
        }

        let inner_hook = InnerHook { outer_hook: hook };

        Self {
            inner_applet: inner_initializer(Box::new(inner_hook)),
        }
    }

    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> InnerApplet,
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook| Self::new(inner_initializer, hook)
    }
}
impl<InnerApplet: NodularApplet> BasicApplet for NodularHolderApplet<InnerApplet> {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.inner_applet.handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        self.inner_applet.get_window()
    }
}

/// Holds the recursive_node_applet, is held by a Basic Applet runner (applet runner).
pub struct RootNodeApplet {
    /// NOTE: this could be generic
    applet: RecursiveNodeApplet,
    // window: Arc<Mutex<UIElement>>,
    // hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
}
impl RootNodeApplet {
    pub fn new(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        struct InnerHook {
            outer_hook: Box<dyn BasicRunnerHook>,
        }
        impl BasicRunnerHook for InnerHook {
            // fn update_display(&self, display: &singularity_ui::ui_element::UIElement) {
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
            // fn update_treeview(&self, _treeview: &singularity_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: Box<super::NodularAppletInitializer>) {
                todo!()
            }

            fn damage_treeview(&self) {
                // REVIEW
                self.outer_hook.damage_window();
            }

            fn focus_out(&self) {}
        }

        let inner_hook = InnerHook { outer_hook: hook };

        Self {
            applet: inner_initializer(Box::new(inner_hook)),
        }
    }

    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet,
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook| Self::new(inner_initializer, hook)
    }

    fn get_treeview_display(&self) -> UIElement {
        let focused_path = self.applet.get_focus_path();
        let treeview = self.applet.get_treeview();

        println!("Focused path: {:?}", focused_path);

        // CharGrid::from(treeview.outer_world_to_string())
        //     .element()
        //     .bordered(Color::LIGHT_GREEN)
        //     .fill_bg(Color::BLACK)

        // This is the horizontal split
        UIElement::combine_displays((0..(focused_path.0.len() + 1)).map(|world_level_index| {
            let world_path = WorldTreePath(
                focused_path.0[0..world_level_index]
                    .to_vec()
                    .into_boxed_slice(),
            );
            CharGrid::from(
                treeview
                    .safe_get(world_path)
                    .unwrap()
                    .outer_world_to_string(focused_path.0.get(world_level_index).cloned()),
            )
            .element()
            .bordered(Color::LIGHT_GREEN)
        }))
        .fill_bg(Color::BLACK)
    }
}
impl BasicApplet for RootNodeApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.applet.handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        UIElement::Container(vec![
            self.get_treeview_display()
                .contain(DisplayArea::new((0.0, 0.0), (0.2, 1.0))),
            self.applet
                .get_window()
                .contain(DisplayArea::new((0.2, 0.0), (1.0, 1.0))),
        ])
    }
}
