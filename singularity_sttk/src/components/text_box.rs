use sonamu_ui::{
    ui_element::CharGrid,
    ui_event::{Key, UIEvent},
};

// use super::Component;

/// just plaintext
pub struct TextBox {
    text: CharGrid,

    // /// (x, y) or (col, row)
    // scroll: (u16, u16),
    /// (x, y) or (col, row)
    cursor_logical_position: (usize, usize),
}
impl TextBox {
    pub fn new(text: String) -> Self {
        Self {
            text: CharGrid::from(text),
            cursor_logical_position: (0, 0),
        }
    }

    /// TODO: TS will not work anymore; store text as rope not as the UI bro...
    pub fn get_text_as_string(&self) -> String {
        self.text.get_text_as_string()
    }

    fn clamp_everything(&mut self) {
        {
            // clamp cursor
            // NOTE: should clamp cursor y before cursor x
            self.cursor_logical_position.1 = self
                .cursor_logical_position
                .1
                .clamp(0, self.text.content.len() - 1);

            self.cursor_logical_position.0 = self
                .cursor_logical_position
                .0
                .clamp(0, self.text.content[self.cursor_logical_position.1].len());
        }
    }

    /// char can not be new line
    /// knows location from cursor
    fn write_character(&mut self, character: sonamu_ui::ui_element::CharCell) {
        self.text.content[self.cursor_logical_position.1]
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

            let new_cursor_x = self.text.content[self.cursor_logical_position.1 - 1].len();

            let mut string_to_add = self.text.content.remove(self.cursor_logical_position.1);
            self.text.content[self.cursor_logical_position.1 - 1].append(&mut string_to_add);

            self.cursor_logical_position.1 -= 1;
            self.cursor_logical_position.0 = new_cursor_x;

            return;
        }

        self.text.content[self.cursor_logical_position.1]
            .remove(self.cursor_logical_position.0 - 1);
        self.cursor_logical_position.0 -= 1;
    }

    /// knows location from cursor
    fn write_new_line(&mut self) {
        let remaining_text = self.text.content[self.cursor_logical_position.1]
            .split_off(self.cursor_logical_position.0);

        self.text
            .content
            .insert(self.cursor_logical_position.1 + 1, remaining_text);

        self.cursor_logical_position.0 = 0;
        self.cursor_logical_position.1 += 1;
    }

    pub fn render_grid_with_color(
        &self,
        (cursor_fg, cursor_bg): (sonamu_ui::color::Color, sonamu_ui::color::Color),
    ) -> CharGrid {
        let mut text_clone = self.text.clone();

        // add this in case the cursor is rightmost
        text_clone.content[self.cursor_logical_position.1]
            .push(sonamu_ui::ui_element::CharCell::new(' '));

        // highlight cursor
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].bg =
            cursor_bg;
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].fg =
            cursor_fg;

        text_clone
    }

    pub fn render_grid(&self) -> CharGrid {
        let mut text_clone = self.text.clone();

        // add this in case the cursor is rightmost
        text_clone.content[self.cursor_logical_position.1]
            .push(sonamu_ui::ui_element::CharCell::new(' '));

        // highlight cursor
        use sonamu_ui::color::Color;
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].bg =
            Color::LIGHT_YELLOW;
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].fg =
            Color::BLACK;

        text_clone
    }

    pub fn render(&self) -> sonamu_ui::ui_element::UIElement {
        sonamu_ui::ui_element::UIElement::CharGrid(self.render_grid())
    }

    pub fn handle_event(&mut self, ui_event: UIEvent) {
        use sonamu_ui::ui_event::{KeyModifiers, KeyTrait};

        match ui_event {
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 108 => {
            //     // arrow down
            //     self.cursor_logical_position.1 += 1;
            // }
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 103 => {
            //     // arrow up
            //     self.cursor_logical_position.1 = self.cursor_logical_position.1.saturating_sub(1);
            // }
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 106 => {
            //     // arrow right
            //     self.cursor_logical_position.0 += 1;
            // }
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 105 => {
            //     // arrow left
            //     if let Some(new_cursor_x) = self.cursor_logical_position.0.checked_sub(1) {
            //         self.cursor_logical_position.0 = new_cursor_x;
            //     } else {
            //         // TODO wrap to prev line
            //     }
            // }
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 14 => {
            //     // backspace key
            //     self.delete_character();
            // }
            // UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 28 => {
            //     // Enter key
            //     self.write_new_line();
            // }
            UIEvent::KeyPress(Key::ArrowKeyDown, KeyModifiers::NONE) => {
                // arrow down
                self.cursor_logical_position.1 += 1;
            }
            UIEvent::KeyPress(Key::ArrowKeyUp, KeyModifiers::NONE) => {
                // arrow up
                self.cursor_logical_position.1 = self.cursor_logical_position.1.saturating_sub(1);
            }
            UIEvent::KeyPress(Key::ArrowKeyRight, KeyModifiers::NONE) => {
                // arrow right
                self.cursor_logical_position.0 += 1;
            }
            UIEvent::KeyPress(Key::ArrowKeyLeft, KeyModifiers::NONE) => {
                // arrow left
                if let Some(new_cursor_x) = self.cursor_logical_position.0.checked_sub(1) {
                    self.cursor_logical_position.0 = new_cursor_x;
                } else {
                    // TODO wrap to prev line
                }
            }
            UIEvent::KeyPress(Key::Backspace, KeyModifiers::NONE) => {
                // backspace key
                self.delete_character();
            }
            UIEvent::KeyPress(Key::Enter, KeyModifiers::NONE) => {
                // Enter key
                self.write_new_line();
            }
            UIEvent::KeyPress(key, KeyModifiers::NONE | KeyModifiers::SHIFT)
                if key
                    .to_char()
                    .is_some_and(|c| c.is_ascii_graphic() || c == ' ') =>
            {
                // NOTE: I wish rust will soon implement if let within matches
                if let Some(c) = key.to_char() {
                    self.write_character(sonamu_ui::ui_element::CharCell::new(c));
                }
            }
            _ => {}
        }

        self.clamp_everything();
    }
}
impl Default for TextBox {
    fn default() -> Self {
        Self::new(String::new())
    }
}
impl From<String> for TextBox {
    fn from(value: String) -> Self {
        Self::new(value.lines().map(|s| s.to_string()).collect())
    }
}
