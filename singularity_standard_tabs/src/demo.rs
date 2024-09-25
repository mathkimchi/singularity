use singularity_common::tab::{
    packets::{Event, Request},
    ManagerHandler,
};
use singularity_ui::UIElement;

pub struct DemoTab {
    string: String,
}
impl DemoTab {
    pub fn new(string: String, manager_handler: &ManagerHandler) -> Self {
        manager_handler.send_request(Request::ChangeName("Hi".to_string()));

        Self { string }
    }

    pub fn render(&mut self, _manager_handler: &ManagerHandler) -> Option<UIElement> {
        dbg!("rendering demo");
        Some(UIElement::Text(self.string.clone()))
    }

    pub fn handle_event(&mut self, _event: Event, _manager_handler: &ManagerHandler) {
        // use ratatui::crossterm::event::{
        //     Event as TUIEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        // };

        // match event {
        //     Event::TUIEvent(tui_event) => match tui_event {
        //         TUIEvent::Key(KeyEvent {
        //             modifiers: KeyModifiers::CONTROL,
        //             code: KeyCode::Char('s'),
        //             kind: KeyEventKind::Press,
        //             ..
        //         }) => {
        //             self.save_to_file();
        //         }
        //         tui_event => {
        //             self.text_box.handle_input(tui_event);
        //         }
        //     },
        //     Event::Resize(_) => {}
        //     Event::Close => panic!("Event::Close should not have been forwarded"),
        // }
    }
}
