use super::SubappUI;
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    style::Color,
    text::{Line, ToLine},
    widgets::{Paragraph, Widget, Wrap},
};
use std::path::PathBuf;

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
        text_area: ratatui::prelude::Rect,
        buffer: &mut ratatui::prelude::Buffer,
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

    fn handle_input(&mut self, event: ratatui::crossterm::event::Event) {
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

/// Currently Just treats everything like plaintext.
/// TODO debugger with lldb
///
/// I don't actually know how text editors are usually coded,
/// but I think they don't actually modify the file directly until save,
/// instead having a temporary duplicate file with unsaved changes.
/// I am going to store the temporary data in rust for now.
///
/// NOTE: I want to mention again that this is just a minimal proof of concept
/// because I realized the text editor rabbit hole goes much deeper than I care for at the moment.
/// I don't care about efficiency or even usability.
///
/// NOTE: different types of positions:
/// - absolude display position: where it would be on the display buffer
/// - logical position: where it would be on temp_text_lines[row]'s column-th character
/// - relative position: depends on what it is relative to, probably relative to text area
pub struct Editor {
    file_path: PathBuf,

    /// storing by lines makes operations easier
    temp_text_lines: Vec<String>,

    /// (x, y) or (col, row)
    scroll: (u16, u16),
    /// (x, y) or (col, row)
    cursor_logical_position: (usize, usize),

    most_recent_area: Rect,

    /// debug purpose
    /// TODO remove
    border: bool,
}
impl Editor {
    pub fn new<P>(file_path: P) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Self {
            temp_text_lines: Self::get_content_from_file(&file_path),
            file_path: PathBuf::from(file_path),
            scroll: (0, 0),
            cursor_logical_position: (0, 0),
            most_recent_area: Rect::ZERO,
            border: true,
        }
    }

    pub fn get_content_from_file<P>(file_path: P) -> Vec<String>
    where
        P: AsRef<std::path::Path>,
    {
        let content_string = std::fs::read_to_string(&file_path).unwrap();
        content_string.lines().map(|s| s.to_string()).collect()
    }

    fn clamp_everything(&mut self) {
        self.scroll.1 = self
            .scroll
            .1
            .clamp(0, self.temp_text_lines.len() as u16 - 1);

        {
            // NOTE: should clamp cursor y before cursor x
            self.cursor_logical_position.1 = self
                .cursor_logical_position
                .1
                .clamp(0, self.temp_text_lines.len() - 1);

            self.cursor_logical_position.0 = self.cursor_logical_position.0.clamp(
                0,
                self.temp_text_lines[self.cursor_logical_position.1].len(),
            );
        }

        // clamp scroll y again, this time to ensure that cursor is visible
        // sometimes, this isn't desired behavior though
        self.scroll.1 = self.scroll.1.clamp(
            (self.cursor_logical_position.1 as u16 + 2 + 1)
                .saturating_sub(self.most_recent_area.height),
            self.cursor_logical_position.1 as u16,
        );

        // clamp scroll x to ensure that cursor is visible
        // sometimes, this isn't desired behavior though
        self.scroll.0 = self.scroll.0.clamp(
            (self.cursor_logical_position.0 as u16 + 2 + 1)
                .saturating_sub(self.most_recent_area.width),
            self.cursor_logical_position.0 as u16,
        );
    }

    /// char can not be new line
    /// knows location from cursor
    fn write_character(&mut self, character: char) {
        self.temp_text_lines[self.cursor_logical_position.1]
            .insert(self.cursor_logical_position.0, character);
        self.cursor_logical_position.0 += 1;
    }

    /// knows location from cursor
    fn delete_character(&mut self) {
        if self.cursor_logical_position.0 == 0 {
            if self.cursor_logical_position.1 == 0 {
                // nothing to delete
                return;
            }

            let new_cursor_x = self.temp_text_lines[self.cursor_logical_position.1 - 1].len();

            let string_to_add = self.temp_text_lines.remove(self.cursor_logical_position.1);
            self.temp_text_lines[self.cursor_logical_position.1 - 1] =
                self.temp_text_lines[self.cursor_logical_position.1 - 1].clone()
                    + string_to_add.as_str();

            self.cursor_logical_position.1 -= 1;
            self.cursor_logical_position.0 = new_cursor_x;

            return;
        }

        self.temp_text_lines[self.cursor_logical_position.1]
            .remove(self.cursor_logical_position.0 - 1);
        self.cursor_logical_position.0 -= 1;
    }

    /// knows location from cursor
    fn write_new_line(&mut self) {
        let remaining_text = self.temp_text_lines[self.cursor_logical_position.1]
            .split_off(self.cursor_logical_position.0);

        self.temp_text_lines
            .insert(self.cursor_logical_position.1 + 1, remaining_text);

        self.cursor_logical_position.0 = 0;
        self.cursor_logical_position.1 += 1;
    }
}
impl SubappUI for Editor {
    fn get_title(&self) -> String {
        self.file_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    /// TODO: rn now no wrap
    fn render(
        &mut self,
        total_area: ratatui::prelude::Rect,
        display_buffer: &mut ratatui::prelude::Buffer,
        is_focused: bool,
    ) {
        self.most_recent_area = total_area;

        // the total area includes 1 unit thick border on all sides
        let text_area = ratatui::prelude::Rect::new(
            total_area.x + 1,
            total_area.y + 1,
            total_area.width - 2,
            total_area.height - 2,
        );

        for (relative_row, line) in self.temp_text_lines.as_slice()[self.scroll.1 as usize
            ..(usize::min(
                self.temp_text_lines.len(),
                (self.scroll.1 + text_area.height) as usize,
            ))]
            .iter()
            .enumerate()
        {
            // because of the slice, this loop iterates over only the displayed lines
            // `relative_row` is 0 for the first displayed line and increments for the next line

            let logical_row = relative_row + self.scroll.1 as usize;
            let absolute_display_y = text_area.top() + relative_row as u16;

            display_buffer.set_line(
                text_area.x,
                absolute_display_y,
                &line
                    .split_at_checked(self.scroll.0 as usize)
                    .unwrap_or(("", ""))
                    .1
                    .to_line(),
                text_area.width,
            );

            if self.cursor_logical_position.1 == logical_row {
                // this line has the cursor

                let cursor_cell = display_buffer.get_mut(
                    text_area.x + self.cursor_logical_position.0 as u16 - self.scroll.0,
                    absolute_display_y,
                );
                // cursor_cell.modifier |= Modifier::SLOW_BLINK;
                cursor_cell.bg = Color::Yellow;
            }
        }

        if self.border {
            ratatui::widgets::Block::bordered()
                .title(self.get_title())
                .render(total_area, display_buffer);
        }
    }

    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('b'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // toggle border (debug purposes)

                self.border = !self.border;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageUp,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.scroll.1 = self.scroll.1.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::PageDown,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.scroll.1 += 1;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_logical_position.1 += 1;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_logical_position.1 = self.cursor_logical_position.1.saturating_sub(1);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.cursor_logical_position.0 += 1;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let Some(new_cursor_x) = self.cursor_logical_position.0.checked_sub(1) {
                    self.cursor_logical_position.0 = new_cursor_x;
                } else {
                    // TODO wrap to prev line
                }
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char(character),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.write_character(character);
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.write_new_line();
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.delete_character();
            }
            _ => {}
        }

        self.clamp_everything();
    }
}
