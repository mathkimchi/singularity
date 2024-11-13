use singularity_common::{
    ask_query,
    tab::{
        packets::{Event, Request},
        BasicTab, ManagerHandler,
    },
};
use singularity_ui::{
    color::Color,
    ui_element::{CharCell, CharGrid, UIElement},
    ui_event::{KeyModifiers, KeyTrait},
};
use std::path::PathBuf;

/// Currently Just treats everything like plaintext.
/// This is just the textbox but with a wrapper to work with files.
///
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

    text: CharGrid,

    /// (x, y) or (col, row)
    cursor_logical_position: (usize, usize),

    /// debug purpose
    /// TODO remove
    save_to_temp: bool,
}
impl Editor {
    pub fn new<P>(file_path: P, manager_handler: &ManagerHandler) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let text = Self::get_content(&file_path);
        let file_path = PathBuf::from(file_path);

        manager_handler.send_request(Request::ChangeName(
            file_path.file_name().unwrap().to_str().unwrap().to_string(),
        ));

        Self {
            text,
            file_path,
            cursor_logical_position: (0, 0),
            save_to_temp: false,
        }
    }

    fn get_content<P>(file_path: P) -> CharGrid
    where
        P: AsRef<std::path::Path>,
    {
        let content_string = std::fs::read_to_string(&file_path).unwrap();
        CharGrid::from(content_string)
    }

    fn save_to_file(&self) {
        let new_path = if self.save_to_temp {
            self.file_path.to_str().unwrap().to_string()
                + ".temp"
                + &std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    .to_string()
        } else {
            self.file_path.to_str().unwrap().to_string()
        };

        std::fs::write(new_path, self.text.get_text_as_string()).unwrap();
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
    fn write_character(&mut self, character: singularity_ui::ui_element::CharCell) {
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
}
impl BasicTab for Editor {
    fn initialize_tab(manager_handler: &ManagerHandler) -> Self {
        Self::new(
            serde_json::from_value::<String>(
                ask_query!(manager_handler.get_query_channels(), TabData).session_data,
            )
            .unwrap(),
            manager_handler,
        )
    }

    fn render_tab(&mut self, manager_handler: &ManagerHandler) -> Option<UIElement> {
        let mut text_clone = self.text.clone();

        // add this in case the cursor is rightmost
        text_clone.content[self.cursor_logical_position.1].push(CharCell::new(' '));

        // highlight cursor
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].bg =
            if manager_handler.focus {
                Color::LIGHT_YELLOW
            } else {
                Color::CYAN
            };
        text_clone.content[self.cursor_logical_position.1][self.cursor_logical_position.0].fg =
            Color::BLACK;

        Some(
            UIElement::CharGrid(text_clone)
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN),
        )
    }

    fn handle_tab_event(&mut self, event: Event, _manager_handler: &ManagerHandler) {
        match event {
            Event::UIEvent(ui_event) => match ui_event {
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::CTRL)
                    if key.raw_code == 31 =>
                {
                    self.save_to_file();
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 108 =>
                {
                    // arrow down
                    self.cursor_logical_position.1 += 1;
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 103 =>
                {
                    // arrow up
                    self.cursor_logical_position.1 =
                        self.cursor_logical_position.1.saturating_sub(1);
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 106 =>
                {
                    // arrow right
                    self.cursor_logical_position.0 += 1;
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 105 =>
                {
                    // arrow left
                    if let Some(new_cursor_x) = self.cursor_logical_position.0.checked_sub(1) {
                        self.cursor_logical_position.0 = new_cursor_x;
                    } else {
                        // TODO wrap to prev line
                    }
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 14 =>
                {
                    // backspace key
                    self.delete_character();
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key.raw_code == 28 =>
                {
                    // Enter key
                    self.write_new_line();
                }
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if key
                        .to_char()
                        .is_some_and(|c| c.is_ascii_graphic() || c == ' ') =>
                {
                    // NOTE: I wish rust will soon implement if let within matches
                    if let Some(c) = key.to_char() {
                        self.write_character(CharCell::new(c));
                    }
                }
                _ => {}
            },
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }

        self.clamp_everything();
    }
}
