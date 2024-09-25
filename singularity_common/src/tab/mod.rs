use packets::{Event, Query, Request, Response};
use singularity_ui::{DisplayArea, UIElement};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub mod packets;

/// REVIEW: name this tab runner?
pub trait TabCreator: Send {
    /// Create and start running the create_tab
    fn create_tab(self, manager_handler: ManagerHandler);
}
impl<F, O> TabCreator for F
where
    F: FnOnce(ManagerHandler) -> O + Send,
{
    fn create_tab(self, manager_handler: ManagerHandler) {
        self(manager_handler);
    }
}

/// REVIEW: I don't know if this is code is good or an abomination
pub fn basic_tab_creator<Tab, InitArgs, Initializer, Renderer, EventHandler>(
    init_args: InitArgs,
    initializer: Initializer,
    mut renderer: Renderer,
    mut event_handler: EventHandler,
    // ) -> impl TabCreator
) -> impl FnOnce(ManagerHandler) + Send
where
    InitArgs: Send,
    Initializer: FnOnce(InitArgs, &ManagerHandler) -> Tab + Send,
    Renderer: FnMut(&mut Tab, &ManagerHandler) -> Option<UIElement> + Send,
    EventHandler: FnMut(&mut Tab, Event, &ManagerHandler) + Send,
{
    move |mut manager_handler: ManagerHandler| {
        let mut tab: Tab = initializer(init_args, &manager_handler);

        // TODO: there's gotta be a better way
        'mainloop: loop {
            // don't render until size has been set
            for event in manager_handler.collect_events() {
                match event {
                    Event::Close => {
                        return;
                    }
                    Event::Resize(inner_area) => {
                        manager_handler.inner_area = inner_area;
                        event_handler(&mut tab, event, &manager_handler);
                        break 'mainloop;
                    }
                    event => {
                        event_handler(&mut tab, event, &manager_handler);
                    }
                }
            }
        }

        'mainloop: loop {
            if let Some(new_display_buffer) = renderer(&mut tab, &manager_handler) {
                manager_handler.update_ui_element(new_display_buffer);
            };

            for event in manager_handler.collect_events() {
                match event {
                    Event::Close => {
                        break 'mainloop;
                    }
                    Event::Resize(inner_area) => {
                        manager_handler.inner_area = inner_area;
                        event_handler(&mut tab, event, &manager_handler);
                    }
                    event => {
                        event_handler(&mut tab, event, &manager_handler);
                    }
                }
            }
        }
    }
}

/// Represents communication with tab on manager side
/// TODO: think of better name
struct TabChannels {
    event_tx: Sender<Event>,
    request_rx: Receiver<Request>,
    query_rx: Receiver<Query>,
    response_tx: Sender<Response>,

    ui_element: Arc<Mutex<UIElement>>,
}

/// Represents communication with manager on tab side
struct ManagerChannels {
    event_rx: Receiver<Event>,
    request_tx: Sender<Request>,
    query_tx: Sender<Query>,
    response_rx: Receiver<Response>,

    ui_element: Arc<Mutex<UIElement>>,
}

fn create_channels() -> (TabChannels, ManagerChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();
    let (query_tx, query_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();
    let display_buffer: Arc<Mutex<UIElement>> = Arc::new(Mutex::new(UIElement::Div(Vec::new())));

    (
        TabChannels {
            event_tx,
            request_rx,
            query_rx,
            response_tx,
            ui_element: display_buffer.clone(),
        },
        ManagerChannels {
            event_rx,
            request_tx,
            query_tx,
            response_rx,
            ui_element: display_buffer,
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
        let tab_thread = thread::spawn(move || {
            tab_creator.create_tab(ManagerHandler {
                manager_channels,
                inner_area: DisplayArea::default(),
            })
        });

        Self {
            tab_channels,
            _tab_thread: tab_thread,
            tab_name: String::new(),
        }
    }

    // pub fn new_from_box(tab_creator: Box<dyn FnOnce(ManagerHandler) + Send>) -> Self {
    //     Self::new(tab_creator)
    // }

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

    pub fn get_ui_element(&self) -> UIElement {
        self.tab_channels.ui_element.lock().unwrap().clone()
    }
}

/// Represents manager on tab side, is a wrapper for ManagerChannels
pub struct ManagerHandler {
    manager_channels: ManagerChannels,

    pub inner_area: DisplayArea,
}
impl ManagerHandler {
    pub fn send_request(&self, request: Request) {
        self.manager_channels
            .request_tx
            .send(request)
            .expect("failed to send request")
    }

    pub fn query(&self, query: Query) -> Response {
        self.manager_channels
            .query_tx
            .send(query)
            .expect("failed to send query");

        self.manager_channels
            .response_rx
            .recv()
            .expect("failed to get response")
    }

    pub fn update_ui_element(&mut self, ui_element: UIElement) {
        *self.manager_channels.ui_element.lock().unwrap() =
            // std::mem::take(&mut self.intermediate_display_buffer);
            ui_element;
    }

    pub fn get_event_rx(&self) -> &Receiver<Event> {
        &self.manager_channels.event_rx
    }

    pub fn collect_events(&self) -> Vec<Event> {
        self.get_event_rx().try_iter().collect()
    }
}
