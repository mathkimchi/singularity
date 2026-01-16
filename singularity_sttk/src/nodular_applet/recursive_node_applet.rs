use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;
use std::sync::{Arc, Mutex, Weak, atomic::AtomicUsize};

struct SubAppletHolder {
    applet: Mutex<Box<dyn NodularApplet>>,
    window: Arc<Mutex<UIElement>>,
}

/// The main divided applet holds an Arc to this and applets hold Weak to this.
/// REVIEW: rename
struct SharedResource {
    applets: Mutex<Vec<Arc<SubAppletHolder>>>,
    focus_index: AtomicUsize,
    hook: Box<dyn BasicRunnerHook>,
}
impl SharedResource {
    fn add_child(
        shared_resource: Weak<Self>,
        child_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) {
        let child_holder = {
            let inner_applet_window = Arc::new(Mutex::new(UIElement::Nothing));

            struct InnerHook {
                // outer_children: Arc<Mutex<Vec<SubAppletHolder>>>,
                // // outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
                // outer_hook: Arc<Box<dyn BasicRunnerHook>>,
                window: Arc<Mutex<UIElement>>,
                // outer_focused_child_index: Arc<Mutex<usize>>,
                shared_resource: Weak<SharedResource>,

                // the index of this hook's corresponding app in the shared resource list of applets
                index: usize,
            }
            impl BasicRunnerHook for InnerHook {
                fn update_display(&self, display: &UIElement) {
                    // self.outer_hook.lock().unwrap().update_display(display);

                    *self.window.lock().unwrap() = display.clone();

                    // TODO: update if focused
                    // REVIEW: This is unwrap of unwrap seems potentially dangerous
                    if self
                        .shared_resource
                        .upgrade()
                        .unwrap()
                        .focus_index
                        .load(std::sync::atomic::Ordering::Relaxed)
                        == self.index
                    {
                        // this child is focused
                        self.shared_resource
                            .upgrade()
                            .unwrap()
                            .hook
                            .update_display(display);
                    }
                }

                fn close(&self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn add_child(&self, initializer: Box<NodularAppletInitializer>) {
                    SharedResource::add_child(self.shared_resource.clone(), initializer);
                }
            }

            let inner_hook = InnerHook {
                window: inner_applet_window.clone(),
                index: shared_resource
                    .upgrade()
                    .unwrap()
                    .applets
                    .lock()
                    .unwrap()
                    .len(),
                shared_resource: shared_resource.clone(),
            };

            println!("Hi");

            Arc::new(SubAppletHolder {
                applet: Mutex::new(child_initializer(Box::new(inner_hook))),
                window: inner_applet_window,
            })
        };

        shared_resource
            .upgrade()
            .unwrap()
            .applets
            .lock()
            .unwrap()
            .push(child_holder);
    }
}

/// Has a list of inner applets and displays them in vertical or horizontal division.
pub struct DividedApplet {
    shared_resource: Arc<SharedResource>,
}
impl DividedApplet {
    fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        let hook = hook;
        let applets = Mutex::new(Vec::new());
        let focus_index = AtomicUsize::new(0);

        let s = Self {
            shared_resource: Arc::new(SharedResource {
                applets,
                focus_index,
                hook,
            }),
        };

        SharedResource::add_child(Arc::downgrade(&s.shared_resource), inner_initiator);

        s
    }

    /// Partial application
    pub fn get_initiator(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook: Box<dyn BasicRunnerHook>| Self::new(inner_initiator, hook)
    }
}
impl BasicApplet for DividedApplet {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.shared_resource.applets.lock().unwrap()[self
            .shared_resource
            .focus_index
            .load(std::sync::atomic::Ordering::Relaxed)]
        .applet
        .lock()
        .unwrap()
        .handle_ui_event(ui_event);
    }
}
impl NodularApplet for DividedApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        let _ = nodular_event;
        todo!()
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
