use singularity_ui::{ui_element::UIElement, ui_event::UIEvent};

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
    fn get_window(&self) -> UIElement;

    /// Was initially going to use a shared boolean for this but never mind
    fn is_window_dirty(&self) -> bool;
}
impl BasicApplet for Box<dyn BasicApplet> {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        // REVIEW: I don't know what ** does
        (**self).handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        (**self).get_window()
    }

    fn is_window_dirty(&self) -> bool {
        (**self).is_window_dirty()
    }
}

// pub trait AppletInitializer<Applet: BasicApplet, Hook>: FnOnce(Hook) -> Applet {}
// impl<Applet: BasicApplet, Hook, F: FnOnce(Hook) -> Applet> AppletInitializer<Applet, Hook> for F {}
