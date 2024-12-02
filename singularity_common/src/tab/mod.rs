use crate::project::project_settings::TabData;
use packets::{create_query_channels, Event, QueryChannels, Request, RespondChannels};
use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub mod packets;
pub mod tile;

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
pub trait BasicTab: Send + Sized {
    fn initialize_tab(manager_handler: &ManagerHandler) -> Self;
    fn render_tab(&mut self, manager_handler: &ManagerHandler) -> Option<UIElement>;
    fn handle_tab_event(&mut self, event: Event, manager_handler: &ManagerHandler);

    fn new_tab_creator() -> impl TabCreator {
        struct Inner(Box<dyn FnMut(ManagerHandler) + Send>);
        impl TabCreator for Inner {
            fn create_tab(&mut self, manager_handler: ManagerHandler) {
                self.0(manager_handler)
            }
        }

        Inner(Box::new(move |mut manager_handler: ManagerHandler| {
            let mut tab = Self::initialize_tab(&manager_handler);

            'mainloop: loop {
                if let Some(new_display_buffer) = tab.render_tab(&manager_handler) {
                    manager_handler.update_ui_element(new_display_buffer);
                };

                for event in manager_handler.collect_events() {
                    match event {
                        Event::Close => {
                            break 'mainloop;
                        }
                        Event::Resize(inner_area) => {
                            manager_handler.inner_area = inner_area;
                            tab.handle_tab_event(event, &manager_handler);
                        }
                        Event::Focused => {
                            manager_handler.focus = true;
                            tab.handle_tab_event(event, &manager_handler);
                        }
                        Event::Unfocused => {
                            manager_handler.focus = false;
                            tab.handle_tab_event(event, &manager_handler);
                        }
                        Event::UIEvent(_) => {
                            tab.handle_tab_event(event, &manager_handler);
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
    pub respond_channels: RespondChannels,

    pub ui_element: Arc<Mutex<UIElement>>,
}

/// Represents communication with manager on tab side
struct ManagerChannels {
    event_rx: Receiver<Event>,
    request_tx: Sender<Request>,
    query_channels: QueryChannels,

    ui_element: Arc<Mutex<UIElement>>,
}

fn create_channels() -> (TabChannels, ManagerChannels) {
    let (event_tx, event_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();
    let (query_channels, respond_channels) = create_query_channels();
    let display_buffer: Arc<Mutex<UIElement>> =
        Arc::new(Mutex::new(UIElement::Container(Vec::new())));

    (
        TabChannels {
            event_tx,
            request_rx,
            respond_channels,
            ui_element: display_buffer.clone(),
        },
        ManagerChannels {
            event_rx,
            request_tx,
            query_channels,
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
    tab_data: TabData,

    /// REVIEW: idk if this will ever be used
    /// I realized I can't kill threads anyways
    _tab_thread: JoinHandle<()>,
}
impl TabHandler {
    /// TODO: allow setting focus
    pub fn new<F: 'static + TabCreator>(
        mut tab_creator: F,
        initial_tab_data: TabData,
        tab_area: DisplayArea,
    ) -> Self {
        let (tab_channels, manager_channels) = create_channels();

        // create tab thread with manager proxy
        let tab_thread = thread::spawn(move || {
            tab_creator.create_tab(ManagerHandler {
                manager_channels,
                inner_area: tab_area,
                // TODO
                focus: false,
            })
        });

        Self {
            tab_channels,
            _tab_thread: tab_thread,
            tab_name: String::new(),
            tab_area,
            tab_data: initial_tab_data,
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

    pub fn get_respond_channels(&self) -> &RespondChannels {
        &self.tab_channels.respond_channels
    }

    pub fn get_ui_element(&self) -> UIElement {
        self.tab_channels.ui_element.lock().unwrap().clone()
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

        self.send_event(Event::Resize(new_area));
    }

    pub fn get_tab_data(&self) -> &TabData {
        &self.tab_data
    }
}

/// Represents manager on tab side, is a wrapper for ManagerChannels
pub struct ManagerHandler {
    manager_channels: ManagerChannels,

    pub inner_area: DisplayArea,
    pub focus: bool,
}
impl ManagerHandler {
    pub fn send_request(&self, request: Request) {
        self.manager_channels
            .request_tx
            .send(request)
            .expect("failed to send request")
    }

    pub fn get_query_channels(&self) -> &QueryChannels {
        &self.manager_channels.query_channels
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
