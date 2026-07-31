use std::thread;

use calloop::LoopHandle;
use singularity_common::sap::{
    packets::StandardEvent, raw_client_initializer::RawClientInitializer,
};
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::ui_element::UIElement;

use crate::runner::AppletRunner;

/// ~~Implemented by Applet,~~ used by SDE/server.
pub(crate) struct ClientHandle {
    event_queue: calloop::channel::Sender<StandardEvent>,
    surface: EncapsulatedLock<UIElement>,
}
impl ClientHandle {
    pub fn spawn_new_client(
        client_initializer: Box<dyn RawClientInitializer>,
        initial_content: UIElement,
        runner_event_loop: &LoopHandle<'static, AppletRunner>,
    ) -> Self {
        let surface = EncapsulatedLock::new(initial_content);
        let (event_tx, event_rx) = calloop::channel::channel();
        let (request_tx, request_rx) = calloop::channel::channel();

        // I don't think order matters,
        // but just in-case I should start listening before I make the client
        runner_event_loop
            .insert_source(request_rx, |request, &mut (), runner| {
                let calloop::channel::Event::Msg(request) = request else {
                    // Means the UI closed; haven't thought abt what to do in this case
                    panic!()
                };
                runner.handle_client_request(request);
            })
            .unwrap();

        {
            let surface = surface.clone();
            thread::spawn(|| {
                client_initializer.init(surface, event_rx, request_tx);
            });
        }

        Self {
            event_queue: event_tx,
            surface,
        }
    }

    pub fn send_event(&self, event: StandardEvent) {
        self.event_queue.send(event).unwrap();
    }

    pub fn get_surface(&self) -> UIElement {
        // this order matters; you should generally do the action first then send notification that you did it
        let surface = self.surface.get();
        self.send_event(StandardEvent::SurfaceDamageAck);
        surface
    }
}
