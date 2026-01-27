use crate::nodular_applet::{NodularApplet, NodularRunnerHook};
use singularity_common::sync::EncapsulatedLock;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;
use std::sync::Mutex;

/// Implements storing window and treeview holder.
pub struct SubAppletHolder {
    applet: Mutex<Box<dyn NodularApplet>>,
    window: EncapsulatedLock<UIElement>,
    treeview: EncapsulatedLock<UIElement>,
}
impl SubAppletHolder {
    pub fn set_window(&self, window: UIElement) {
        self.window.set(window);
    }

    pub fn get_window(&self) -> UIElement {
        self.window.get()
    }

    pub fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        outer_hook: Box<dyn NodularRunnerHook>,
        window: EncapsulatedLock<UIElement>,
        treeview: EncapsulatedLock<UIElement>,
    ) -> Self {
        struct InnerHook {
            window: EncapsulatedLock<UIElement>,
            treeview: EncapsulatedLock<UIElement>,
            outer_hook: Box<dyn NodularRunnerHook>,
        }
        impl BasicRunnerHook for InnerHook {
            fn update_display(&self, display: &UIElement) {
                self.window.set(display.clone());

                self.outer_hook.update_display(display);
            }

            fn close(&self) {
                self.outer_hook.close();
            }
        }
        impl NodularRunnerHook for InnerHook {
            fn update_treeview(&self, treeview: &UIElement) {
                self.treeview.set(treeview.clone());

                self.outer_hook.update_treeview(treeview);
            }

            fn add_child(&self, initializer: Box<super::NodularAppletInitializer>) {
                self.outer_hook.add_child(initializer);
            }
        }

        let inner_hook = InnerHook {
            window: window.clone(),
            treeview: treeview.clone(),
            outer_hook,
        };

        Self {
            applet: Mutex::new(inner_initiator(Box::new(inner_hook))),
            window,
            treeview,
        }
    }

    pub fn immut_handle_ui_event(&self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.applet.lock().unwrap().handle_ui_event(ui_event);
    }

    pub fn immut_handle_nodular_event(&self, nodular_event: super::NodularEvent) {
        self.applet
            .lock()
            .unwrap()
            .handle_nodular_event(nodular_event);
    }
}
impl BasicApplet for SubAppletHolder {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.immut_handle_ui_event(ui_event);
    }
}
impl NodularApplet for SubAppletHolder {
    fn handle_nodular_event(&mut self, nodular_event: super::NodularEvent) {
        self.immut_handle_nodular_event(nodular_event);
    }
}
