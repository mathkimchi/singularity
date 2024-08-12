use super::SubappUI;
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    style::Color,
    text::ToLine,
    widgets::Widget,
};
use std::path::PathBuf;

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

    fn save_to_temp_file(&self) {
        std::fs::write(
            self.file_path.to_str().unwrap().to_string()
                + ".temp"
                + &std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string(),
            self.temp_text_lines.join("\n"),
        )
        .unwrap();
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

                if is_focused {
                    // cursor_cell.modifier |= Modifier::SLOW_BLINK;
                    cursor_cell.bg = Color::Yellow;
                }
            }
        }

        if self.border {
            ratatui::widgets::Block::bordered()
                .title(format!("{} - Editor", self.get_title()))
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
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.save_to_temp_file();
            }
            _ => {}
        }

        self.clamp_everything();
    }
}
