//! This is where the hierarchy stuff is implemented.

use std::collections::BTreeMap;

use singularity_common::utils::tree::world_tree::{
    WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation,
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use sonamu_ui::{display_units::DisplayContainerSize, ui_element::UIElement};

pub mod caching_applet;
pub mod recursive_node_applet;
pub mod root_node_applet;

pub enum NodularEvent {
    /// More or less means that the selector is over this tab but isn't actually selected
    Highlighted(bool),
    Focused(bool),
}

/// Nodular applets are applets that can be in the applet hierarchy.
pub trait NodularApplet: BasicApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent);

    fn get_treeview(&self) -> WorldTree<String>;

    /// Assume focus updates when treeview updates
    fn get_focus_path(&self) -> WorldTreePath;
}
// TODO: look into Box::downcast
impl BasicApplet for Box<dyn NodularApplet> {
    fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_ui_event(ui_event);
    }

    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement {
        (**self).get_window(container_size)
    }
}
impl NodularApplet for Box<dyn NodularApplet> {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_nodular_event(nodular_event)
    }

    fn get_treeview(&self) -> WorldTree<String> {
        // REVIEW: I don't know what ** does
        (**self).get_treeview()
    }

    fn get_focus_path(&self) -> WorldTreePath {
        // REVIEW: I don't know what ** does
        (**self).get_focus_path()
    }
}

pub type NodularAppletInitializer =
    Box<dyn FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>>;

pub trait AppletSpawnerTrait {
    fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer>;

    /// Clone on its own is not dyn compatible because it outputs -> Self.
    /// Using the AppletSpawnerTrait, AppletSpawner workaround for that.
    fn duplicate(&self) -> AppletSpawner;
}
pub type AppletSpawner = Box<dyn AppletSpawnerTrait>;

impl Clone for AppletSpawner {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}

pub trait NodularRunnerHook: BasicRunnerHook {
    fn damage_treeview(&self);

    /// REVIEW: the boxes and generics
    fn add_child(&self, initializer: NodularAppletInitializer);

    fn change_focus(&self, operation: WorldTreeTraversalOperation);

    /// NOTE: look at 2026-03-08 for more detail on future things
    fn register_applet_spawner(&self, name: String, applet_spawner: AppletSpawner);
    fn get_applet_spawners(&self) -> BTreeMap<String, AppletSpawner>;
    fn find_applet_spawner(&self, name: String) -> Option<AppletSpawner>;
}
impl BasicRunnerHook for Box<dyn NodularRunnerHook> {
    fn damage_window(&self) {
        (**self).damage_window();
    }

    fn close(&self) {
        (**self).close();
    }
}
impl NodularRunnerHook for Box<dyn NodularRunnerHook> {
    fn damage_treeview(&self) {
        (**self).damage_treeview();
    }

    fn add_child(&self, initializer: NodularAppletInitializer) {
        (**self).add_child(initializer);
    }

    fn change_focus(&self, operation: WorldTreeTraversalOperation) {
        (**self).change_focus(operation);
    }

    fn register_applet_spawner(&self, name: String, applet_spawner: AppletSpawner) {
        (**self).register_applet_spawner(name, applet_spawner);
    }
    fn get_applet_spawners(&self) -> BTreeMap<String, AppletSpawner> {
        (**self).get_applet_spawners()
    }
    /// TODO: take &str
    fn find_applet_spawner(&self, name: String) -> Option<AppletSpawner> {
        (**self).find_applet_spawner(name)
    }
}
