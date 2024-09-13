use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub mod temp_tab;

pub enum Event {
    KeyPress(char),
    /// TODO: close forcibly
    Close,
}

pub enum Request {
    ChangeName(String),
}

pub trait TabCreator: Send {
    /// Create and start running the create_tab
    fn create_tab(self, manager_channel: ManagerChannels);
}

/// Represents tab channels on manager side
/// TODO: think of better name
pub struct TabChannels {
    pub event_tx: Sender<Event>,
    pub request_rx: Receiver<Request>,
}

/// Represents manager channels on tab side
pub struct ManagerChannels {
    pub event_rx: Receiver<Event>,
    pub request_tx: Sender<Request>,
}

fn create_tab_manager_channels() -> (TabChannels, ManagerChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();

    (
        TabChannels {
            event_tx,
            request_rx,
        },
        ManagerChannels {
            event_rx,
            request_tx,
        },
    )
}

/// Represents tab on manager side
pub struct TabHandler {
    tab_channels: TabChannels,

    /// REVIEW: idk if this will ever be used
    /// I realized I can't kill threads anyways
    _tab_thread: JoinHandle<()>,

    pub tab_name: String,
}
impl TabHandler {
    pub fn new<F: 'static + TabCreator>(tab_creator: F) -> Self {
        let (tab_channels, manager_channels) = create_tab_manager_channels();

        // create tab thread with manager proxy
        let tab_thread = thread::spawn(move || tab_creator.create_tab(manager_channels));

        Self {
            tab_channels,
            _tab_thread: tab_thread,
            tab_name: String::new(),
        }
    }

    pub fn send_event(&mut self, event: Event) {
        self.tab_channels
            .event_tx
            .send(event)
            .expect("Failed to send event to tab");
    }

    pub fn collect_requests(&mut self) -> Vec<Request> {
        // returns all pending requests (I assume that means this ends instead of waiting)
        self.tab_channels.request_rx.try_iter().collect()
    }
}
