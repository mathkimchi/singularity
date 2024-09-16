use singularity_common::{
    elements::text_box::TextBox,
    tab::{
        packets::{Event, Request},
        ManagerHandler,
    },
    ui::DisplayBuffer,
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
    save_to_temp: bool,
}
impl Editor {
    pub fn new<P>(file_path: P, manager_handler: &ManagerHandler) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let text_box = Self::generate_textbox(&file_path);
        let file_path = PathBuf::from(file_path);

        manager_handler.send_request(Request::ChangeName(
            file_path.file_name().unwrap().to_str().unwrap().to_string(),
        ));

        Self {
            text_box,
            file_path,
            save_to_temp: false,
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

    pub fn render(&mut self, _manager_handler: &ManagerHandler) -> Option<DisplayBuffer> {
        // let mut ratatui_buffer = Buffer::empty(manager_handler.inner_area);

        // self.text_box
        //     .render(manager_handler.inner_area, &mut ratatui_buffer, true);

        // Some(ratatui_buffer.content)

        todo!()
    }

    pub fn handle_event(&mut self, _event: Event, _manager_handler: &ManagerHandler) {
        // use ratatui::crossterm::event::{
        //     Event as TUIEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        // };

        // match event {
        //     Event::TUIEvent(tui_event) => match tui_event {
        //         TUIEvent::Key(KeyEvent {
        //             modifiers: KeyModifiers::CONTROL,
        //             code: KeyCode::Char('s'),
        //             kind: KeyEventKind::Press,
        //             ..
        //         }) => {
        //             self.save_to_file();
        //         }
        //         tui_event => {
        //             self.text_box.handle_input(tui_event);
        //         }
        //     },
        //     Event::Resize(_) => {}
        //     Event::Close => panic!("Event::Close should not have been forwarded"),
        // }
    }
}
