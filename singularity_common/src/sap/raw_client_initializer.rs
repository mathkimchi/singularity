use calloop::channel::{Channel, Sender};
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::ui_element::UIElement;

use crate::sap::packets::{StandardEvent, StandardRequest};

/// Look at 2026-07-30 DEVLOG for initial creation
///
/// Called "Raw" because expectation is to make a wrapper in sttk for less boilerplate
pub trait RawClientInitializer: Send {
    fn init(
        // smth smth box needs to know size
        self: Box<Self>,
        content: EncapsulatedLock<UIElement>,
        event_queue: Channel<StandardEvent>,
        // yeah, ik the naming is inconsistent bc I'm not saying "event_receiver" or "event_rx", but it's calm
        // (I am really trying to convince myself this is fine, I am the strawman)
        request_sender: Sender<StandardRequest>,
    );
}
