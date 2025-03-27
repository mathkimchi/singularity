use crate::packets::{SDEEvent, SDERequest};
use singularity_sap::{
    byte_stream::{ByteReaderWrapper, CombinedByteStream},
    standard_packets::display_packets::ResizeEvent,
    universal_stream::universal_server_stream::{QueryDataResponder, UniversalServerStream},
};
use singularity_sporg::project_settings::TabData;
use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};
use std::process::{Child, ChildStdin, Command, Stdio};

pub struct TabHandler {
    communication: UniversalServerStream<CombinedByteStream<ByteReaderWrapper, ChildStdin>>,

    pub tab_name: String,
    pub tab_area: DisplayArea,
    pub tab_display: UIElement,
    pub tab_data: TabData,
    tab_process: Child,
}
impl TabHandler {
    /// TODO: allow setting focus
    pub fn new(initial_tab_data: TabData, tab_area: DisplayArea) -> Self {
        let mut tab_spawn_command = Command::new(&initial_tab_data.tab_command.program)
            .args(&initial_tab_data.tab_command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let byte_stream = CombinedByteStream::take_from_child(&mut tab_spawn_command).unwrap();

        Self {
            communication: UniversalServerStream::new(byte_stream),
            tab_name: String::new(),
            tab_area,
            tab_display: UIElement::Nothing,
            tab_data: initial_tab_data,
            tab_process: tab_spawn_command,
        }
    }

    pub fn send_event(&mut self, event: SDEEvent) {
        self.communication.send_event_union(event);
    }

    #[must_use]
    pub fn handle_incoming(
        &mut self,
        query_responders: &mut Vec<&mut dyn QueryDataResponder>,
    ) -> Vec<SDERequest> {
        // returns all pending requests (I assume that means this ends instead of waiting)
        self.communication.handle_incoming(query_responders)
    }

    // pub fn get_respond_channels(&self) -> &RespondChannels {
    //     &self.tab_channels.respond_channels
    // }

    pub fn get_display(&self) -> &UIElement {
        // self.tab_channels.ui_element.lock().unwrap().clone()
        &self.tab_display
    }

    pub fn get_area(&self) -> DisplayArea {
        self.tab_area
    }

    pub fn set_area(&mut self, new_area: DisplayArea) {
        if self.tab_area == new_area {
            // optimization, but might cause annoying behavior
            return;
        }

        self.tab_area = new_area;

        self.send_event(SDEEvent::Resize(ResizeEvent(new_area)));
    }

    pub fn get_tab_data(&self) -> &TabData {
        &self.tab_data
    }

    pub fn kill(mut self) {
        self.tab_process.kill().unwrap();
    }
}
