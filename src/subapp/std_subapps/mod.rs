use super::SubappUI;
use crate::project_manager::ManagerProxy;
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Color,
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};

pub mod editor;
pub mod file_manager;
pub mod task_organizer;

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
        &mut self,
        area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
        _manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    ) {
        Paragraph::new(self.content.clone())
            .wrap(Wrap { trim: false })
            .block(ratatui::widgets::Block::bordered().title(self.title.clone()))
            .render(area, buffer);

        if is_focused {
            // FIXME: multiline

            let cursor_cell = &mut buffer[(area.x + 1 + (self.cursor_location as u16), area.y + 1)];
            // cursor_cell.modifier |= Modifier::SLOW_BLINK;
            cursor_cell.bg = Color::Yellow;
        }
    }

    fn handle_input(&mut self, _manager_proxy: &mut ManagerProxy, event: Event) {
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
    wrap: bool,

    /// debug purpose
    /// TODO remove
    border: bool,
}
impl TextReader {
    pub fn from_text(text: String) -> Self {
        Self {
            text,
            scroll: 0,
            wrap: false,
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

    fn render_text_no_word_wrap(
        &self,
        text_area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
    ) {
        let mut y = text_area.y;
        for line in self.text.lines().collect::<Vec<&str>>()[self.scroll as usize..].iter() {
            if y >= text_area.bottom() {
                break;
            }

            let mut x = text_area.x;
            for word in line.split(' ') {
                buffer.set_line(x, y, &Line::from(word), text_area.right().saturating_sub(x));
                x += word.len() as u16 + 1;
            }

            y += 1;
        }
    }

    /// For word wrapping, I am going to
    /// [minimize number of lines](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap#Minimum_number_of_lines)
    /// rather than raggedness because in a code editor, I prefer predictability over beauty.
    fn render_text_with_wrap(
        &self,
        _text_area: ratatui::prelude::Rect,
        _buffer: &mut ratatui::prelude::Buffer,
    ) {
        todo!()
    }
}
impl SubappUI for TextReader {
    fn get_title(&self) -> String {
        "Reader".to_string()
    }

    fn render(
        &mut self,
        total_area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
        _manager_proxy: &mut ManagerProxy,
        _is_focused: bool,
    ) {
        // the total area includes 1 unit thick border on all sides
        let text_area = ratatui::prelude::Rect::new(
            total_area.x + 1,
            total_area.y + 1,
            total_area.width - 2,
            total_area.height - 2,
        );
        if self.wrap {
            self.render_text_with_wrap(text_area, buffer);
        } else {
            self.render_text_no_word_wrap(text_area, buffer);
        }

        if self.border {
            ratatui::widgets::Block::bordered()
                .title(self.get_title())
                .render(total_area, buffer);
        }
    }

    fn handle_input(
        &mut self,
        _manager_proxy: &mut ManagerProxy,
        event: ratatui::crossterm::event::Event,
    ) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('b'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // toggle border (debug purposes)

                self.border = !self.border;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('w'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // 'w' toggles wrap

                self.wrap = !self.wrap;
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
