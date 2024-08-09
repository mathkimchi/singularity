use super::SubappUI;
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Color,
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};

pub struct DemoSubapp {
    pub title: String,
    pub content: String,
    pub cursor_location: usize,
}
impl DemoSubapp {
    // NOTE: rly just for convenience
    pub fn box_from_title(title: &str) -> Box<dyn SubappUI> {
        Box::new(Self {
            title: title.to_string(),
            content: "Placeholder".to_string(),
            cursor_location: "Placeholder".len(),
        })
    }
}
impl SubappUI for DemoSubapp {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn render(
        &self,
        area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
        is_focused: bool,
    ) {
        Paragraph::new(self.content.clone())
            .wrap(Wrap { trim: false })
            .block(ratatui::widgets::Block::bordered().title(self.title.clone()))
            .render(area, buffer);

        if is_focused {
            // FIXME: multiline

            let cursor_cell =
                buffer.get_mut(area.x + 1 + (self.cursor_location as u16), area.y + 1);
            // cursor_cell.modifier |= Modifier::SLOW_BLINK;
            cursor_cell.bg = Color::Yellow;
        }
    }

    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(character),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.content.insert(self.cursor_location, character);
                self.cursor_location += 1;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.cursor_location != 0 {
                    // remove the character before the cursor
                    self.content.remove(self.cursor_location - 1);
                    self.cursor_location -= 1;
                }
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_location += 1;

                self.cursor_location = self.cursor_location.clamp(0, self.content.len());
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_location = self.cursor_location.saturating_sub(1);

                // extra safety
                self.cursor_location = self.cursor_location.clamp(0, self.content.len());
            }
            _ => {}
        }
    }
}

/// First step to text editor
pub struct TextReader {
    // file_path: Path,
    text: String,
    scroll: u16,

    /// debug purpose
    /// TODO remove
    border: bool,
}
impl TextReader {
    pub fn from_text(text: String) -> Self {
        Self {
            text,
            scroll: 0,
            border: true,
        }
    }

    pub fn subapp_from_file<P>(file_path: P) -> Box<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let text = std::fs::read_to_string(file_path).unwrap();
        Box::new(Self::from_text(text))
    }

    pub fn lorem_ipsum_box() -> Box<Self> {
        const LOREM_IPSUM: &str = r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."#;

        Box::new(Self::from_text(
            LOREM_IPSUM.to_string() + LOREM_IPSUM + LOREM_IPSUM,
        ))
    }
}
impl SubappUI for TextReader {
    fn get_title(&self) -> String {
        "Reader".to_string()
    }

    fn render(
        &self,
        area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
        _is_focused: bool,
    ) {
        // no wrap for now bc hard
        let mut y = area.y + 1;
        for line in self.text.split('\n').collect::<Vec<&str>>()[self.scroll as usize..].iter() {
            if y >= area.bottom() - 1 {
                break;
            }

            let mut x = area.x + 1;
            for word in line.split(' ') {
                buffer.set_line(x, y, &Line::from(word), area.right().saturating_sub(x + 1));
                x += word.len() as u16 + 1;
            }

            y += 1;
        }

        if self.border {
            ratatui::widgets::Block::bordered()
                .title(self.get_title())
                .render(area, buffer);
        }
    }

    fn handle_input(&mut self, event: ratatui::crossterm::event::Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('b'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.border = !self.border;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageUp,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.scroll = self.scroll.saturating_sub(1);

                // FIXME: does not work with word wrap
                self.scroll = self.scroll.clamp(0, self.text.split("\n").count() as u16);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageDown,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.scroll += 1;

                // FIXME: does not work with word wrap
                self.scroll = self.scroll.clamp(0, self.text.split("\n").count() as u16);
            }
            _ => {}
        }
    }
}

// /// Currently Just treats everything like plaintext.
// /// TODO debugger with lldb
// ///
// /// I don't actually know how text editors are usually coded,
// /// I think they don't actually modify the file directly until save,
// /// instead having a temporary duplicate file with unsaved changes.
// pub struct TextEditor {}
