use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::{Paragraph, Widget},
};

pub trait Block {
    fn get_name(&self) -> &String;

    /// FIXME: currently, graphics are not agnostic
    fn draw(&self) -> Paragraph;

    /// FIXME: currently, events are not agnostic
    fn handle_input(&mut self, event: Event);
}

pub struct DemoBlock {
    pub name: String,
    pub content: String,
}
impl DemoBlock {
    // NOTE: rly just for convenience
    pub fn box_from_name(name: &str) -> Box<dyn Block> {
        Box::new(Self {
            name: name.to_string(),
            content: "Placeholder".to_string(),
        })
    }
}
impl Block for DemoBlock {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn draw(&self) -> Paragraph {
        // kinda annoying that ratatui also has something called block
        Paragraph::new(self.content.clone())
            .block(ratatui::widgets::Block::bordered().title(self.name.clone()))
    }

    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(character),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.content.push(character);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.content.pop();
            }
            _ => {}
        }
    }
}
