use singularity_ui::{ui_element::UIElement, ui_event::UIEvent};

/// The runner gives this to the Applet
pub trait RunnerHook {
    fn update_display(&mut self, display: &UIElement);

    // fn query(&mut self, query: Query);
}
impl<F> RunnerHook for F
where
    F: FnMut(&UIElement),
{
    fn update_display(&mut self, display: &UIElement) {
        self(display);
    }
}

pub trait BasicApplet {
    type InitializingData;

    // REVIEW: should I use box or generic like BasicApplet<Hook: RunnerHook>?
    fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self;

    fn handle_ui_event(&mut self, ui_event: UIEvent);
}
