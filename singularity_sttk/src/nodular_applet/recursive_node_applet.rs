use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;
use std::sync::{Arc, Mutex};

struct SubAppletHolder {
    applet: Box<dyn NodularApplet>,
    window: Arc<Mutex<UIElement>>,
}

pub struct RecursiveNodeApplet {
    inner_applet: SubAppletHolder,
    children: Arc<Mutex<Vec<SubAppletHolder>>>,

    /// If inner (this node's value) is focused, this is None
    focused_child_index: Arc<Mutex<Option<usize>>>,

    hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
}
impl RecursiveNodeApplet {
    fn child_hook(
        outer_hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
        window: Arc<Mutex<UIElement>>,
    ) -> Box<dyn NodularRunnerHook> {
        struct ChildHook {
            outer_hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
            window: Arc<Mutex<UIElement>>,
        }
        impl BasicRunnerHook for ChildHook {
            fn update_display(&mut self, display: &UIElement) {
                *self.window.lock().unwrap() = display.clone();

                // TODO: update if focused
            }

            fn close(&mut self) {
                todo!()
            }
        }
        impl NodularRunnerHook for ChildHook {
            fn add_child(&mut self, initializer: Box<NodularAppletInitializer>) {
                todo!()
            }
        }

        Box::new(ChildHook { outer_hook, window })
    }

    fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        let hook = Arc::new(Mutex::new(hook));

        let children = Arc::new(Mutex::new(Vec::new()));
        let focused_child_index = Arc::new(Mutex::new(None));

        let inner_applet = {
            let inner_applet_window = Arc::new(Mutex::new(UIElement::Nothing));

            struct InnerHook {
                outer_children: Arc<Mutex<Vec<SubAppletHolder>>>,
                // outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
                outer_hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
                window: Arc<Mutex<UIElement>>,
            }
            impl BasicRunnerHook for InnerHook {
                fn update_display(&mut self, display: &UIElement) {
                    // self.outer_hook.lock().unwrap().update_display(display);

                    *self.window.lock().unwrap() = display.clone();

                    // TODO: update if focused
                }

                fn close(&mut self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.outer_hook.lock().unwrap().close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn add_child(&mut self, initializer: Box<NodularAppletInitializer>) {
                    let child_applet_window = Arc::new(Mutex::new(UIElement::Nothing));

                    let child_applet = initializer(RecursiveNodeApplet::child_hook(
                        self.outer_hook.clone(),
                        child_applet_window.clone(),
                    ));

                    self.outer_children.lock().unwrap().push(SubAppletHolder {
                        applet: child_applet,
                        window: child_applet_window,
                    });
                }
            }

            let inner_hook = InnerHook {
                outer_hook: hook.clone(),
                outer_children: children.clone(),
                window: inner_applet_window.clone(),
            };

            SubAppletHolder {
                applet: inner_initiator(Box::new(inner_hook)),
                window: inner_applet_window,
            }
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
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook: Box<dyn BasicRunnerHook>| Self::new(inner_initiator, hook)
    }
}
impl BasicApplet for RecursiveNodeApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        match *self.focused_child_index.lock().unwrap() {
            None => {
                // focus is on inner
                self.inner_applet.applet.handle_ui_event(ui_event);
            }
            Some(focused_child_index) => {
                self.children.lock().unwrap()[focused_child_index]
                    .applet
                    .handle_ui_event(ui_event);
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

pub struct TiledRecursiveNodeApplet {}
impl TiledRecursiveNodeApplet {}

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
