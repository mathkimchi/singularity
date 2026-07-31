use crate::sap::packets::StandardEvent;
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::ui_element::UIElement;

/// ~~Implemented by Applet,~~ used by SDE/server.
pub struct ClientHandle {
    event_queue: calloop::channel::Sender<StandardEvent>,
    surface: EncapsulatedLock<UIElement>,
}
impl ClientHandle {
    // TODO: new

    pub fn send_event(&self, event: StandardEvent) {
        self.event_queue.send(event).unwrap();
    }

    pub fn get_surface(&self) -> UIElement {
        let surface = self.surface.get();
        self.send_event(StandardEvent::SurfaceDamageAck);
        surface
    }
}
