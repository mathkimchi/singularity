//! TODO: this should be a part of singularity_common as sap

use sonamu_ui::{
    display_units::DisplayContainerSize, layout_builder::LayoutBuilder, ui_element::UIElement,
    ui_event::UIEvent,
};

/// The runner gives this to the Applet
pub trait BasicRunnerHook {
    fn damage_window(&self);

    fn close(&self);

    // fn query(&mut self, query: Query);
}
impl BasicRunnerHook for Box<dyn BasicRunnerHook> {
    fn damage_window(&self) {
        (**self).damage_window();
    }

    fn close(&self) {
        (**self).close();
    }
}

pub trait BasicApplet {
    // type InitializingData;

    // // REVIEW: should I use box or generic like BasicApplet<Hook: RunnerHook>?
    // fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self;

    /// REVIEW: make this immutable?
    fn handle_ui_event(&mut self, ui_event: UIEvent);

    /// This should resolve damaged state
    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement;

    fn layout_builder(&self) -> LayoutBuilder<'_> {
        LayoutBuilder::from_fn(|container_size| self.get_window(container_size))
    }
}
impl BasicApplet for Box<dyn BasicApplet> {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_ui_event(ui_event);
    }

    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement {
        (**self).get_window(container_size)
    }
}

// pub trait AppletInitializer<Applet: BasicApplet, Hook>: FnOnce(Hook) -> Applet {}
// impl<Applet: BasicApplet, Hook, F: FnOnce(Hook) -> Applet> AppletInitializer<Applet, Hook> for F {}
