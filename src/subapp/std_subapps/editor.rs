use super::SubappUI;
use crate::{elements::text_box::TextBox, manager::ManagerProxy};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    style::Color,
    text::ToLine,
    widgets::Widget,
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

    text_box: TextBox,

    /// debug purpose
    /// TODO remove
    border: bool,
    save_to_temp: bool,
}
impl Editor {
    pub fn new<P>(file_path: P) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Self {
            text_box: Self::generate_textbox(&file_path),
            file_path: PathBuf::from(file_path),
            border: true,
            save_to_temp: true,
        }
    }

    pub fn generate_textbox<P>(file_path: P) -> TextBox
    where
        P: AsRef<std::path::Path>,
    {
        let content_string = std::fs::read_to_string(&file_path).unwrap();
        TextBox::from(content_string)
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
        _manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    ) {
        // the total area includes 1 unit thick border on all sides
        let text_area = ratatui::prelude::Rect::new(
            total_area.x + 1,
            total_area.y + 1,
            total_area.width - 2,
            total_area.height - 2,
        );

        self.text_box.render(text_area, display_buffer, is_focused);

        if self.border {
            ratatui::widgets::Block::bordered()
                .title(format!("{} - Editor", self.get_title()))
                .render(total_area, display_buffer);
        }
    }

    fn handle_input(&mut self, _manager_proxy: &mut ManagerProxy, event: Event) {
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
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.save_to_file();
            }
            event => {
                self.text_box.handle_input(event);
            }
        }
    }
}
