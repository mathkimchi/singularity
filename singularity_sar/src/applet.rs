use singularity_ui::{ui_element::UIElement, ui_event::UIEvent};

/// The runner gives this to the Applet
pub trait RunnerHook {
    fn update_display(&mut self, display: &UIElement);

    // fn query(&mut self, query: Query);
}

pub trait Applet {
    fn handle_ui_event(&mut self, ui_event: UIEvent);

    fn set_hook(&mut self, hook: Box<dyn RunnerHook>);
}
