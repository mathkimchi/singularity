use crate::nodular_applet::{NodularApplet, NodularRunnerHook};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};

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
            fn update_display(&self, display: &singularity_ui::ui_element::UIElement) {
                self.outer_hook.update_display(display);
            }

            fn close(&self) {
                self.outer_hook.close();
            }
        }
        impl NodularRunnerHook for InnerHook {
            fn update_treeview(&self, _treeview: &singularity_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: Box<super::NodularAppletInitializer>) {}
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
}

// /// Holds the recursive_node_applet, is held by a Basic Applet runner (applet runner).
// pub struct RootNodeApplet {
//     applet: RecursiveNodeApplet,
//     window: Arc<Mutex<UIElement>>,

//     hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
// }
// impl RootNodeApplet {}
// impl BasicApplet for RootNodeApplet {
//     fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
//         ui_event;
//     }
// }
