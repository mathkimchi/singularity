//! This is where the hierarchy stuff is implemented.

use singularity_common::utils::tree::rooted_tree::RootedTree;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::ui_element::UIElement;

pub mod applet_holder;
pub mod recursive_node_applet;
pub mod root_node_applet;

// pub struct RootApplet<Applet: BasicApplet> {
//     inner: Applet,
//     children: NodeApplet<Box<dyn BasicApplet>>,
// }
// impl<Applet: BasicApplet> BasicApplet for RootApplet<Applet> {
//     type InitializingData = Applet::InitializingData;

//     fn initialize(
//         initializing_data: Self::InitializingData,
//         hook: Box<dyn singularity_sar::applet::RunnerHook>,
//     ) -> Self {
//         todo!()
//     }

//     fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
//         todo!()
//     }
// }

pub enum NodularEvent {
    /// More or less means that the selector is over this tab but isn't actually selected
    Highlighted(bool),
    Focused(bool),
}

/// Nodular applets are applets that can be in the applet hierarchy.
pub trait NodularApplet: BasicApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent);

    fn get_treeview(&self) -> RootedTree<String>;
}
// TODO: look into Box::downcast
impl BasicApplet for Box<dyn NodularApplet> {
    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        (**self).get_window()
    }
}
impl NodularApplet for Box<dyn NodularApplet> {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_nodular_event(nodular_event);
    }

    fn get_treeview(&self) -> RootedTree<String> {
        // REVIEW: I don't know what ** does
        (**self).get_treeview()
    }
}

pub type NodularAppletInitializer =
    dyn FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>;

pub trait NodularRunnerHook: BasicRunnerHook {
    fn damage_treeview(&self);

    /// REVIEW: the boxes and generics
    fn add_child(&self, initializer: Box<NodularAppletInitializer>);
}
impl BasicRunnerHook for Box<dyn NodularRunnerHook> {
    fn damage_window(&self) {
        (**self).damage_window();
    }

    fn close(&self) {
        (**self).close();
    }
}
