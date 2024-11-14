use singularity_common::{
    ask_query,
    components::{text_box::TextBox, Component},
    tab::{
        packets::{Event, Request},
        BasicTab, ManagerHandler,
    },
};
use singularity_ui::{color::Color, ui_element::UIElement, ui_event::KeyModifiers};
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

    text_box: TextBox,

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
        let text_box = TextBox::new(Self::get_content(&file_path));
        let file_path = PathBuf::from(file_path);

        manager_handler.send_request(Request::ChangeName(
            file_path.file_name().unwrap().to_str().unwrap().to_string(),
        ));

        Self {
            file_path,
            text_box,
            save_to_temp: false,
        }
    }

    fn get_content<P>(file_path: P) -> String
    where
        P: AsRef<std::path::Path>,
    {
        std::fs::read_to_string(&file_path).unwrap()
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

        std::fs::write(new_path, self.text_box.get_text_as_string()).unwrap();
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
        // highlight cursor
        let cursor_fg = Color::BLACK;
        let cursor_bg = if manager_handler.focus {
            Color::LIGHT_YELLOW
        } else {
            Color::CYAN
        };

        Some(
            UIElement::CharGrid(self.text_box.render_grid_with_color(cursor_fg, cursor_bg))
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN),
        )
    }

    fn handle_tab_event(&mut self, event: Event, _manager_handler: &ManagerHandler) {
        match event {
            Event::UIEvent(ref ui_event) => match ui_event {
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::CTRL)
                    if key.raw_code == 31 =>
                {
                    self.save_to_file();
                }
                _ => {
                    self.text_box.handle_event(event);
                }
            },
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
