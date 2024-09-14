use packets::{DisplayBuffer, Event, Query, Request, Response};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
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

/// Represents communication with tab on manager side
/// TODO: think of better name
struct TabChannels {
    event_tx: Sender<Event>,
    request_rx: Receiver<Request>,
    query_rx: Receiver<Query>,
    response_tx: Sender<Response>,

    display_buffer: Arc<Mutex<DisplayBuffer>>,
}

/// Represents communication with manager on tab side
/// REVIEW: make a wrapper for this like TabHandler?
pub struct ManagerChannels {
    pub event_rx: Receiver<Event>,
    pub request_tx: Sender<Request>,
    query_tx: Sender<Query>,
    response_rx: Receiver<Response>,

    display_buffer: Arc<Mutex<DisplayBuffer>>,
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

    pub fn update_display_buffer(&mut self, new_display_buffer: DisplayBuffer) {
        *self.display_buffer.lock().unwrap() =
            // std::mem::take(&mut self.intermediate_display_buffer);
            new_display_buffer;
    }
}

fn create_channels() -> (TabChannels, ManagerChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();
    let (query_tx, query_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();
    let display_buffer: Arc<Mutex<DisplayBuffer>> = Arc::new(Mutex::new(DisplayBuffer::new()));

    (
        TabChannels {
            event_tx,
            request_rx,
            query_rx,
            response_tx,
            display_buffer: display_buffer.clone(),
        },
        ManagerChannels {
            event_rx,
            request_tx,
            query_tx,
            response_rx,
            display_buffer,
        },
    )
}

/// Represents tab on manager side, is a wrapper for TabChannels
pub struct TabHandler {
    tab_channels: TabChannels,

    /// REVIEW: idk if this will ever be used
    /// I realized I can't kill threads anyways
    _tab_thread: JoinHandle<()>,

    pub tab_name: String,
}
impl TabHandler {
    pub fn new<F: 'static + TabCreator>(tab_creator: F) -> Self {
        let (tab_channels, manager_channels) = create_channels();

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

    pub fn get_display_buffer(&self, min_area: usize) -> DisplayBuffer {
        let mut display_buffer = self.tab_channels.display_buffer.lock().unwrap().to_owned();

        if display_buffer.len() < min_area {
            display_buffer.resize(min_area, ratatui::buffer::Cell::EMPTY);
        }

        display_buffer
    }
}
