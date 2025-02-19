use crate::packets::{SDEEvent, SDERequest};
use singularity_sap::{
    standard_packets::display_packets::ResizeEvent,
    universal_stream::universal_server_stream::{QueryDataResponder, UniversalServerStream},
};
use singularity_sporg::project_settings::TabData;
use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};
use std::os::unix::net::UnixStream;

pub struct TabHandler {
    communication: UniversalServerStream<UnixStream>,

    pub tab_name: String,
    pub tab_area: DisplayArea,
    pub tab_display: UIElement,
    pub tab_data: TabData,
    // /// REVIEW: idk if this will ever be used
    // /// I realized I can't kill threads anyways
    // _tab_thread: JoinHandle<()>,
}
impl TabHandler {
    // /// TODO: allow setting focus
    // pub fn new<F: 'static + TabCreator>(
    //     mut tab_creator: F,
    //     initial_tab_data: TabData,
    //     tab_area: DisplayArea,
    // ) -> Self {
    //     let (tab_channels, manager_channels) = create_channels();

    //     // create tab thread with manager proxy
    //     let tab_thread = thread::spawn(move || {
    //         tab_creator.create_tab(ManagerHandler {
    //             manager_channels,
    //             inner_area: tab_area,
    //             // TODO
    //             focus: false,
    //         })
    //     });

    //     Self {
    //         tab_channels,
    //         _tab_thread: tab_thread,
    //         tab_name: String::new(),
    //         tab_area,
    //         tab_data: initial_tab_data,
    //     }
    // }

    pub fn send_event(&mut self, event: SDEEvent) {
        self.communication.send_event(event);
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
}
