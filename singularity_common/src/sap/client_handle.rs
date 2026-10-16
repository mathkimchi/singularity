use crate::{
    sap::{
        packets::{StandardEvent, StateKind},
        raw_client_initializer::RawClientInitializer,
        shared_state::SharedState,
    },
    utils::tree::world_tree::WorldTree,
};
use calloop::LoopHandle;
use sonamu_ui::ui_element::{UIElement, UITree};
use std::thread;

/// ~~Implemented by Applet,~~ used by whoever holds a client (the sar server or an sttk node).
///
/// Dropping it closes the client's event queue, which should end the client.
pub struct ClientHandle {
    event_queue: calloop::channel::Sender<StandardEvent>,
    surface: SharedState<UITree>,
    treeview: SharedState<WorldTree<String>>,
}
impl ClientHandle {
    /// Runs the client on a new thread.
    /// Its requests are handled by `on_request` on `event_loop`.
    pub fn spawn_new_client<Request: Send + 'static, Data: 'static>(
        client_initializer: Box<dyn RawClientInitializer<Request>>,
        event_loop: &LoopHandle<'static, Data>,
        mut on_request: impl FnMut(Request, &mut Data) + 'static,
    ) -> Self {
        let surface = SharedState::new(StateKind::Surface, UITree::leaf(UIElement::Nothing));
        let treeview = SharedState::new(StateKind::Treeview, WorldTree::new_base(String::new()));
        let (event_tx, event_rx) = calloop::channel::channel();
        let (request_tx, request_rx) = calloop::channel::channel();

        // I don't think order matters,
        // but just in-case I should start listening before I make the client
        event_loop
            .insert_source(request_rx, move |request, &mut (), data| {
                // `Closed` means the client ended; calloop removes the source after that
                if let calloop::channel::Event::Msg(request) = request {
                    on_request(request, data);
                }
            })
            .unwrap();

        {
            let surface = surface.clone();
            let treeview = treeview.clone();
            thread::spawn(|| {
                client_initializer.init(surface, treeview, event_rx, request_tx);
            });
        }

        Self {
            event_queue: event_tx,
            surface,
            treeview,
        }
    }

    pub fn send_event(&self, event: StandardEvent) {
        // the client may have already ended (eg it closed itself)
        let _ = self.event_queue.send(event);
    }

    pub fn get_surface(&self) -> UITree {
        self.get_and_ack(&self.surface)
    }

    pub fn get_treeview(&self) -> WorldTree<String> {
        self.get_and_ack(&self.treeview)
    }

    fn get_and_ack<T: Clone>(&self, state: &SharedState<T>) -> T {
        // this order matters; you should generally do the action first then send notification that you did it
        let value = state.get();
        // TODO: just realized that I need to send this after the ui display confirms it too
        // TODO: make surface damage ack a UI Event so UI display can send it too
        self.send_event(StandardEvent::DamageAck(state.kind()));
        value
    }
}
