use packets::{Event, Query, Request, Response};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub mod packets;
pub mod temp_tab;

pub trait TabCreator: Send {
    /// Create and start running the create_tab
    fn create_tab(self, manager_channel: ManagerChannels);
}
impl<F> TabCreator for F
where
    F: FnOnce(ManagerChannels) + Send,
{
    fn create_tab(self, manager_channel: ManagerChannels) {
        self(manager_channel)
    }
}

/// Represents tab channels on manager side
/// TODO: think of better name
pub struct TabChannels {
    pub event_tx: Sender<Event>,
    pub request_rx: Receiver<Request>,
    pub query_rx: Receiver<Query>,
    pub response_tx: Sender<Response>,
}

/// Represents manager channels on tab side
pub struct ManagerChannels {
    pub event_rx: Receiver<Event>,
    pub request_tx: Sender<Request>,
    query_tx: Sender<Query>,
    response_rx: Receiver<Response>,
}
impl ManagerChannels {
    pub fn send_request(&self, request: Request) {
        self.request_tx
            .send(request)
            .expect("failed to send request")
    }

    pub fn query(&self, query: Query) -> Response {
        self.query_tx.send(query).expect("failed to send query");

        self.response_rx.recv().expect("failed to get response")
    }
}

fn create_tab_manager_channels() -> (TabChannels, ManagerChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();
    let (query_tx, query_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    (
        TabChannels {
            event_tx,
            request_rx,
            query_rx,
            response_tx,
        },
        ManagerChannels {
            event_rx,
            request_tx,
            query_tx,
            response_rx,
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

    pub fn send_event(&self, event: Event) {
        self.tab_channels
            .event_tx
            .send(event)
            .expect("Failed to send event to tab");
    }

    pub fn collect_requests(&self) -> Vec<Request> {
        // returns all pending requests (I assume that means this ends instead of waiting)
        self.tab_channels.request_rx.try_iter().collect()
    }

    pub fn answer_query<F: FnOnce(Query) -> Response>(&self, f: F) {
        if let Ok(query) = self.tab_channels.query_rx.try_recv() {
            self.tab_channels
                .response_tx
                .send(f(query))
                .expect("failed to send response");
        }
    }
}
