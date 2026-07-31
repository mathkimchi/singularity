//! REVIEW: I am not sure where this belongs
//! it is implemenented in SDE but used in Ratk of STTK

use sonamu_sync::EncapsulatedLock;
use sonamu_ui::ui_element::UIElement;

use crate::sap::packets::StandardRequest;

/// This represents the server on the applet (client) side.
pub struct ServerHandle {
    request_queue: calloop::channel::Sender<StandardRequest>,
    surface: EncapsulatedLock<UIElement>,
}

impl ServerHandle {
    // /// Called by Applet, implemented by server (SDE).
    // fn query(&mut self, query: Todo) -> Todo;

    /// Called by Applet, implemented by server (SDE).
    pub fn send_request(&self, request: StandardRequest) {
        self.request_queue.send(request).unwrap();
    }

    pub fn set_surface(&self, surface: UIElement) {
        self.surface.set(surface);
        self.send_request(StandardRequest::DamageSurface);
    }
}
