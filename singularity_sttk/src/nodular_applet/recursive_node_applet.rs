use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;
use std::sync::{Arc, Mutex};

pub struct RecursiveNodeApplet {
    inner_applet: Box<dyn NodularApplet>,
    children: Arc<Mutex<Vec<Box<dyn NodularApplet>>>>,

    /// If inner (this node's value) is focused, this is None
    focused_child_index: Arc<Mutex<Option<usize>>>,

    hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
}
impl RecursiveNodeApplet {
    fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        hook: Box<dyn NodularRunnerHook>,
    ) -> Self {
        let hook = Arc::new(Mutex::new(hook));

        let children = Arc::new(Mutex::new(Vec::new()));
        let focused_child_index = Arc::new(Mutex::new(None));

        let inner_applet = {
            struct InnerHook {
                outer_children: Arc<Mutex<Vec<Box<dyn NodularApplet>>>>,
                outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
            }
            impl BasicRunnerHook for InnerHook {
                fn update_display(&mut self, display: &UIElement) {
                    // TODO
                    // FIXME

                    self.outer_hook.lock().unwrap().update_display(display);
                }

                fn close(&mut self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.outer_hook.lock().unwrap().close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn update_miniview(&mut self, miniview: &UIElement) {
                    todo!()
                }

                fn add_child(&mut self, initializer: Box<NodularAppletInitializer>) {
                    let _ = initializer;
                    todo!()
                }
            }

            let inner_hook = InnerHook {
                outer_hook: hook.clone(),
                outer_children: children.clone(),
            };
            inner_initiator(Box::new(inner_hook))
        };

        Self {
            inner_applet,
            children,
            // default is inner applet focused
            focused_child_index,
            hook,
        }
    }

    /// Partial application
    pub fn get_initiator(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        move |hook: Box<dyn NodularRunnerHook>| Self::new(inner_initiator, hook)
    }
}
impl BasicApplet for RecursiveNodeApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        match *self.focused_child_index.lock().unwrap() {
            None => {
                // focus is on inner
                self.inner_applet.handle_ui_event(ui_event);
            }
            Some(focused_child_index) => {
                self.children.lock().unwrap()[focused_child_index].handle_ui_event(ui_event);
            }
        }
    }
}
impl NodularApplet for RecursiveNodeApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        let _ = nodular_event;
        todo!()
    }
}
