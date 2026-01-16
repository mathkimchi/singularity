use singularity_ui::{ui_element::UIElement, ui_event::UIEvent};

/// The runner gives this to the Applet
pub trait BasicRunnerHook {
    fn update_display(&self, display: &UIElement);

    fn close(&self);

    // fn query(&mut self, query: Query);
}
impl BasicRunnerHook for Box<dyn BasicRunnerHook> {
    fn update_display(&self, display: &UIElement) {
        (**self).update_display(display);
    }

    fn close(&self) {
        (**self).close();
    }
}

pub trait BasicApplet {
    // type InitializingData;

    // // REVIEW: should I use box or generic like BasicApplet<Hook: RunnerHook>?
    // fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self;

    fn handle_ui_event(&mut self, ui_event: UIEvent);
}
impl BasicApplet for Box<dyn BasicApplet> {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_ui_event(ui_event);
    }
}

// pub trait AppletInitializer<Applet: BasicApplet, Hook>: FnOnce(Hook) -> Applet {}
// impl<Applet: BasicApplet, Hook, F: FnOnce(Hook) -> Applet> AppletInitializer<Applet, Hook> for F {}
