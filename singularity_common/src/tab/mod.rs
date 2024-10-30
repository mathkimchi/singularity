use packets::{Event, Query, Request, Response};
use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};
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
    fn create_tab(&mut self, manager_handler: ManagerHandler);
}
// impl<F, O> TabCreator for F
// where
//     F: FnOnce(ManagerHandler) -> O + Send,
// {
//     fn create_tab(self, manager_handler: ManagerHandler) {
//         self(manager_handler);
//     }
// }
impl TabCreator for Box<dyn TabCreator> {
    fn create_tab(&mut self, manager_handler: ManagerHandler) {
        self.as_mut().create_tab(manager_handler)
    }
}

/// REVIEW: the parallel between this and `Component` is undeniable, maybe look into some higher level of abstraction in this
pub trait BasicTab<InitArgs: 'static + Send>: Send + Sized {
    fn initialize(init_args: &mut InitArgs, manager_handler: &ManagerHandler) -> Self;
    fn render(&mut self, manager_handler: &ManagerHandler) -> Option<UIElement>;
    fn handle_event(&mut self, event: Event, manager_handler: &ManagerHandler);

    fn new_tab_creator(mut init_args: InitArgs) -> impl TabCreator {
        struct Inner(Box<dyn FnMut(ManagerHandler) + Send>);
        impl TabCreator for Inner {
            fn create_tab(&mut self, manager_handler: ManagerHandler) {
                self.0(manager_handler)
            }
        }

        Inner(Box::new(move |mut manager_handler: ManagerHandler| {
            let mut tab = Self::initialize(&mut init_args, &manager_handler);

            'mainloop: loop {
                if let Some(new_display_buffer) = tab.render(&manager_handler) {
                    manager_handler.update_ui_element(new_display_buffer);
                };

                for event in manager_handler.collect_events() {
                    match event {
                        Event::Close => {
                            break 'mainloop;
                        }
                        Event::Resize(inner_area) => {
                            manager_handler.inner_area = inner_area;
                            tab.handle_event(event, &manager_handler);
                        }
                        event => {
                            tab.handle_event(event, &manager_handler);
                        }
                    }
                }
            }
        }))
    }
}

/// Represents communication with tab on manager side
/// TODO: think of better name
pub struct TabChannels {
    pub event_tx: Sender<Event>,
    pub request_rx: Receiver<Request>,
    pub query_rx: Receiver<Query>,
    pub response_tx: Sender<Response>,

    pub ui_element: Arc<Mutex<UIElement>>,
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
    let display_buffer: Arc<Mutex<UIElement>> =
        Arc::new(Mutex::new(UIElement::Container(Vec::new())));

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
///
/// REVIEW: Shall I transport this to the manager?
pub struct TabHandler {
    tab_channels: TabChannels,

    pub tab_name: String,
    tab_area: DisplayArea,

    /// REVIEW: idk if this will ever be used
    /// I realized I can't kill threads anyways
    _tab_thread: JoinHandle<()>,
}
impl TabHandler {
    pub fn new<F: 'static + TabCreator>(mut tab_creator: F, tab_area: DisplayArea) -> Self {
        let (tab_channels, manager_channels) = create_channels();

        // create tab thread with manager proxy
        let tab_thread = thread::spawn(move || {
            tab_creator.create_tab(ManagerHandler {
                manager_channels,
                inner_area: tab_area,
            })
        });

        Self {
            tab_channels,
            _tab_thread: tab_thread,
            tab_name: String::new(),
            tab_area,
        }
    }

    // pub fn new_from_box(tab_creator: Box<dyn TabCreator + Send>, tab_area: DisplayArea) -> Self {
    //     let (tab_channels, manager_channels) = create_channels();

    //     // create tab thread with manager proxy
    //     let tab_thread = thread::spawn(move || {
    //         tab_creator.create_tab(ManagerHandler {
    //             manager_channels,
    //             inner_area: tab_area,
    //         })
    //     });

    //     Self {
    //         tab_channels,
    //         _tab_thread: tab_thread,
    //         tab_name: String::new(),
    //         tab_area,
    //     }
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

    pub fn get_area(&self) -> DisplayArea {
        self.tab_area
    }

    pub fn set_area(&mut self, new_area: DisplayArea) {
        self.tab_area = new_area;

        self.send_event(Event::Resize(new_area));
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
