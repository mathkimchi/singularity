use crate::nodular_applet::{NodularApplet, NodularRunnerHook};
use singularity_common::{sync::EncapsulatedLock, utils::tree::world_tree::WorldTree};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

/// Implements caching for an app.
pub struct CachingApplet {
    applet: Mutex<Box<dyn NodularApplet>>,
    window: EncapsulatedLock<UIElement>,
    is_window_dirty: Arc<AtomicBool>,
    treeview: EncapsulatedLock<WorldTree<String>>,
    treeview_damaged: Arc<AtomicBool>,
}
impl CachingApplet {
    pub fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        outer_hook: Box<dyn NodularRunnerHook>,
        window: EncapsulatedLock<UIElement>,
        // window_damaged: Arc<AtomicBool>,
        treeview: EncapsulatedLock<WorldTree<String>>,
    ) -> Self {
        let is_window_dirty = Arc::new(AtomicBool::new(true));
        let treeview_damaged = Arc::new(AtomicBool::new(true));

        struct InnerHook {
            window_damaged: Arc<AtomicBool>,
            treeview_damaged: Arc<AtomicBool>,
            outer_hook: Box<dyn NodularRunnerHook>,
        }
        impl BasicRunnerHook for InnerHook {
            fn damage_window(&self) {
                if !self
                    .window_damaged
                    .swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    // was previously not damaged
                    self.outer_hook.damage_window();
                }
            }

            fn close(&self) {
                self.outer_hook.close();
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, treeview: &UIElement) {
            //     self.treeview.set(treeview.clone());

            //     self.outer_hook.update_treeview(treeview);
            // }

            fn add_child(&self, initializer: super::NodularAppletInitializer) {
                self.outer_hook.add_child(initializer);
            }

            fn damage_treeview(&self) {
                if !self
                    .treeview_damaged
                    .swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    // was previously not damaged
                    self.outer_hook.damage_treeview();
                }
            }

            fn change_focus(
                &self,
                operation: singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation,
            ) {
                self.outer_hook.change_focus(operation);
            }

            fn register_applet_spawner(&self, name: String, applet_spawner: super::AppletSpawner) {
                self.outer_hook
                    .register_applet_spawner(name, applet_spawner);
            }
            fn get_applet_spawners(
                &self,
            ) -> std::collections::BTreeMap<String, super::AppletSpawner> {
                self.outer_hook.get_applet_spawners()
            }
            fn find_applet_spawner(&self, name: String) -> Option<super::AppletSpawner> {
                self.outer_hook.find_applet_spawner(name)
            }
        }

        let inner_hook = InnerHook {
            window_damaged: is_window_dirty.clone(),
            treeview_damaged: treeview_damaged.clone(),
            outer_hook,
        };

        Self {
            applet: Mutex::new(inner_initiator(Box::new(inner_hook))),
            window,
            is_window_dirty,
            treeview,
            treeview_damaged,
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

    /* #[deprecated]
    pub fn placeholder() -> Self {
        /// Since I need an applet to make multi-applet holder
        /// and the actual applet needs a hook to the multi-applet holder,
        /// so I am going to make this placeholder first then make the holder
        /// then make the actual applet.
        struct PlaceholderApp;
        impl BasicApplet for PlaceholderApp {
            fn handle_ui_event(&mut self, _ui_event: UIEvent) {
                panic!("Placeholder app's functions should not be called")
            }

            fn get_window(&self) -> UIElement {
                panic!("Placeholder app's functions should not be called")
            }
        }
        impl NodularApplet for PlaceholderApp {
            fn handle_nodular_event(&mut self, _nodular_event: NodularEvent) {
                panic!("Placeholder app's functions should not be called")
            }

            fn get_treeview(&self) -> WorldTree<String> {
                panic!("Placeholder app's functions should not be called")
            }

            fn get_focus_path(&self) -> WorldTreePath {
                panic!("Placeholder app's functions should not be called")
            }
        }

        Self {
            applet: Mutex::new(Box::new(PlaceholderApp)),
            window: EncapsulatedLock::new(UIElement::Nothing),
            window_damaged: Arc::new(AtomicBool::new(false)),
            treeview: EncapsulatedLock::new(WorldTree::Base("Placeholder".to_string())),
            treeview_damaged: Arc::new(AtomicBool::new(false)),
        }
    } */
}
impl BasicApplet for CachingApplet {
    fn get_window(&self) -> UIElement {
        if self
            .is_window_dirty
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            self.window.set(self.applet.lock().unwrap().get_window());
        }

        // if the child is still dirty
        if self.applet.lock().unwrap().is_window_dirty() {
            self.is_window_dirty
                .store(true, std::sync::atomic::Ordering::Relaxed);
        }

        self.window.get()
    }

    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        self.immut_handle_ui_event(ui_event);
    }

    fn is_window_dirty(&self) -> bool {
        self.is_window_dirty
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
impl NodularApplet for CachingApplet {
    fn handle_nodular_event(&mut self, nodular_event: super::NodularEvent) {
        self.immut_handle_nodular_event(nodular_event);
    }

    fn get_treeview(&self) -> WorldTree<String> {
        if self
            .treeview_damaged
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            self.treeview
                .set(self.applet.lock().unwrap().get_treeview());
        }

        self.treeview.get()
    }

    /// TODO: cache this
    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        self.applet.lock().unwrap().get_focus_path()
    }
}
