use singularity_macros::{Packet, PacketUnion};
use singularity_sap::{
    byte_stream::{ByteReaderWrapper, ByteStream, CombinedByteStream},
    datable::{ToData, TryFromData},
    packet::{IdType, PacketTrait},
    standard_packets::display_packets::{
        CloseWarningEvent, DisplayEvent, FocusedEvent, RequestChangeName, RequestUpdateWindow,
        SessionDataQuery, UnfocusedEvent,
    },
    universal_stream::universal_client_stream::UniversalClientStream,
};
use singularity_sttk::components::text_box::TextBox;
use singularity_ui::{color::Color, ui_element::UIElement, ui_event::KeyModifiers};
use std::{io::Stdout, path::PathBuf};

#[derive(PacketUnion, Packet)]
pub enum Event {
    DisplayEvent(DisplayEvent),
}

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
    pub fn new<P>(
        file_path: P,
        client_stream: &mut UniversalClientStream<impl ByteStream, Event>,
    ) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let text_box = TextBox::new(Self::get_content(&file_path));
        let file_path = PathBuf::from(file_path);

        client_stream.send_request(RequestChangeName::new(
            &file_path.file_name().unwrap().to_str().unwrap(),
        ));

        Self {
            file_path,
            text_box,
            save_to_temp: false,
        }
    }

    fn get_content(file_path: impl AsRef<std::path::Path>) -> String {
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

    pub fn initialize_tab(
        client_stream: &mut UniversalClientStream<impl ByteStream, Event>,
    ) -> Self {
        Self::new(
            serde_json::from_value::<String>(client_stream.query(SessionDataQuery).unwrap().0)
                .unwrap(),
            client_stream,
        )
    }

    pub fn render_tab(&mut self) -> UIElement {
        // highlight cursor
        let cursor_fg = Color::BLACK;
        // TODO: focus
        // let cursor_bg = if manager_handler.focus {
        //     Color::LIGHT_YELLOW
        // } else {
        //     Color::CYAN
        // };
        let cursor_bg = Color::LIGHT_YELLOW;

        UIElement::CharGrid(self.text_box.render_grid_with_color((cursor_fg, cursor_bg)))
            .fill_bg(Color::DARK_GRAY)
            .bordered(Color::LIGHT_GREEN)
    }

    pub fn handle_tab_event(&mut self, event: DisplayEvent) {
        match event {
            DisplayEvent::UIEvent(ref ui_event) => match ui_event {
                singularity_ui::ui_event::UIEvent::KeyPress(key, KeyModifiers::CTRL)
                    if key.raw_code == 31 =>
                {
                    self.save_to_file();
                }
                _ => {
                    self.text_box.handle_event(event);
                }
            },
            DisplayEvent::Focused(FocusedEvent) => {}
            DisplayEvent::Unfocused(UnfocusedEvent) => {}
            DisplayEvent::Resize(_) => {}
            DisplayEvent::Close(CloseWarningEvent) => {}
        }
    }
}

fn main() {
    let mut client_stream: UniversalClientStream<
        CombinedByteStream<ByteReaderWrapper, Stdout>,
        Event,
    > = UniversalClientStream::new(CombinedByteStream::take_from_stdio());

    let mut editor = Editor::initialize_tab(&mut client_stream);

    // update window on start and when there is an event
    client_stream.send_request(RequestUpdateWindow {
        contents: editor.render_tab(),
    });

    loop {
        let events = client_stream.wait_read_events();
        for Event::DisplayEvent(event) in events {
            editor.handle_tab_event(event);
        }

        // update window on start and when there is an event
        client_stream.send_request(RequestUpdateWindow {
            contents: editor.render_tab(),
        });
    }
}
