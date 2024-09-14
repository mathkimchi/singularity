use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    style::Color,
    text::ToLine,
};

/// just plaintext
pub struct TextBox {
    /// storing by lines makes operations easier
    text_lines: Vec<String>,

    /// (x, y) or (col, row)
    scroll: (u16, u16),
    /// (x, y) or (col, row)
    cursor_logical_position: (usize, usize),

    /// FIXME: there has got to be a better way
    most_recent_area: Rect,
}
impl TextBox {
    pub fn new(text_lines: Vec<String>) -> Self {
        Self {
            text_lines,
            scroll: (0, 0),
            cursor_logical_position: (0, 0),
            most_recent_area: Rect::ZERO,
        }
    }

    pub fn get_text_lines(&self) -> &Vec<String> {
        &self.text_lines
    }

    pub fn get_text_as_string(&self) -> String {
        self.get_text_lines().join("\n")
    }

    fn clamp_everything(&mut self, display_area: Rect) {
        self.scroll.1 = self.scroll.1.clamp(0, self.text_lines.len() as u16 - 1);

        {
            // NOTE: should clamp cursor y before cursor x
            self.cursor_logical_position.1 = self
                .cursor_logical_position
                .1
                .clamp(0, self.text_lines.len() - 1);

            self.cursor_logical_position.0 = self
                .cursor_logical_position
                .0
                .clamp(0, self.text_lines[self.cursor_logical_position.1].len());
        }

        // clamp scroll y again, this time to ensure that cursor is visible
        // sometimes, this isn't desired behavior though
        self.scroll.1 = self.scroll.1.clamp(
            (self.cursor_logical_position.1 as u16 + 1).saturating_sub(display_area.height),
            self.cursor_logical_position.1 as u16,
        );

        // clamp scroll x to ensure that cursor is visible
        // sometimes, this isn't desired behavior though
        self.scroll.0 = self.scroll.0.clamp(
            (self.cursor_logical_position.0 as u16 + 1).saturating_sub(display_area.width),
            self.cursor_logical_position.0 as u16,
        );
    }

    /// char can not be new line
    /// knows location from cursor
    fn write_character(&mut self, character: char) {
        self.text_lines[self.cursor_logical_position.1]
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

            let new_cursor_x = self.text_lines[self.cursor_logical_position.1 - 1].len();

            let string_to_add = self.text_lines.remove(self.cursor_logical_position.1);
            self.text_lines[self.cursor_logical_position.1 - 1] =
                self.text_lines[self.cursor_logical_position.1 - 1].clone()
                    + string_to_add.as_str();

            self.cursor_logical_position.1 -= 1;
            self.cursor_logical_position.0 = new_cursor_x;

            return;
        }

        self.text_lines[self.cursor_logical_position.1].remove(self.cursor_logical_position.0 - 1);
        self.cursor_logical_position.0 -= 1;
    }

    /// knows location from cursor
    fn write_new_line(&mut self) {
        let remaining_text = self.text_lines[self.cursor_logical_position.1]
            .split_off(self.cursor_logical_position.0);

        self.text_lines
            .insert(self.cursor_logical_position.1 + 1, remaining_text);

        self.cursor_logical_position.0 = 0;
        self.cursor_logical_position.1 += 1;
    }

    pub fn render(&mut self, display_area: Rect, display_buffer: &mut Buffer, is_focused: bool) {
        self.most_recent_area = display_area;

        for (relative_row, line) in self.text_lines.as_slice()[self.scroll.1 as usize
            ..(usize::min(
                self.text_lines.len(),
                (self.scroll.1 + display_area.height) as usize,
            ))]
            .iter()
            .enumerate()
        {
            // because of the slice, this loop iterates over only the displayed lines
            // `relative_row` is 0 for the first displayed line and increments for the next line

            let logical_row = relative_row + self.scroll.1 as usize;
            let absolute_display_y = display_area.top() + relative_row as u16;

            display_buffer.set_line(
                display_area.x,
                absolute_display_y,
                &line
                    .split_at_checked(self.scroll.0 as usize)
                    .unwrap_or(("", ""))
                    .1
                    .to_line(),
                display_area.width,
            );

            if self.cursor_logical_position.1 == logical_row {
                // this line has the cursor

                let cursor_cell = &mut display_buffer[(
                    display_area.x + self.cursor_logical_position.0 as u16 - self.scroll.0,
                    absolute_display_y,
                )];

                if is_focused {
                    // cursor_cell.modifier |= Modifier::SLOW_BLINK;
                    cursor_cell.bg = Color::Yellow;
                }
            }
        }
    }

    pub fn handle_input(&mut self, event: Event) {
        match event {
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

        self.clamp_everything(self.most_recent_area);
    }
}
impl Default for TextBox {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
impl From<String> for TextBox {
    fn from(value: String) -> Self {
        Self::new(value.lines().map(|s| s.to_string()).collect())
    }
}
