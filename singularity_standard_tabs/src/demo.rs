use singularity_common::tab::{
    packets::{Event, Request},
    ManagerHandler,
};
use singularity_ui::{
    ui_event::{Key, KeyModifiers, KeyTrait},
    UIElement,
};

pub struct DemoTab {
    string: String,
}
impl DemoTab {
    pub fn new(string: String, manager_handler: &ManagerHandler) -> Self {
        manager_handler.send_request(Request::ChangeName("Hi".to_string()));

        Self { string }
    }

    pub fn render(&mut self, _manager_handler: &ManagerHandler) -> Option<UIElement> {
        Some(UIElement::Text(self.string.clone()))
    }

    pub fn handle_event(&mut self, event: Event, _manager_handler: &ManagerHandler) {
        match event {
            Event::UIEvent(ui_event) => match ui_event {
                singularity_ui::ui_event::UIEvent::Key {
                    key: Key::Backspace,
                    pressed: true,
                    repeat: false,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    self.string.pop();
                }
                singularity_ui::ui_event::UIEvent::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: KeyModifiers::NONE,
                    ..
                } => {
                    dbg!(key);
                    if let Some(c) = key.to_alphabet() {
                        self.string.push(c);
                    }
                }
                _ => {}
            },
            Event::Resize(_) => {
                // dbg!("resized");
            }
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
