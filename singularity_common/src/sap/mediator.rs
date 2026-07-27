//! Look at 2026-07-20 DEVLOG entry for conception.
//! Issue [#40](https://github.com/mathkimchi/singularity/issues/40)
//! tracks the initial implementation of this.

use crate::sync::EncapsulatedLock;
use sonamu_ui::{ui_element::UIElement, ui_event::UIEvent};
use std::sync::{atomic::AtomicBool, mpsc, Arc};

/// Created by the client initializer; each protocol is its own component.
///
/// This holds the server side (the side called by the server) of the protocols,
/// but the client is given this so they can replace the default implementations.
///
/// Generally, the interfaces like `DisplayContentGetter` and `EventSender`
/// are for the server to call and client to implement (or leave as default).
/// The server doesn't know the impl beyond the interface
/// while the client should know the actual type of it
/// (either because it is kept the default impl struct or because the client re-implemented)
pub struct SonamuMediator {
    display_getter: Box<dyn DisplayContentGetter>,

    event_sender: Box<dyn EventSender>,
}
impl SonamuMediator {
    pub fn new(
        display_getter: impl DisplayContentGetter + 'static,
        event_sender: impl EventSender + 'static,
    ) -> Self {
        Self {
            display_getter: Box::new(display_getter),
            event_sender: Box::new(event_sender),
        }
    }
}

// Server Side Mediator is proxy pattern, is this bad?
impl DisplayContentGetter for SonamuMediator {
    fn is_damaged(&self) -> bool {
        self.display_getter.is_damaged()
    }

    fn get_display_content(&self) -> UIElement {
        self.display_getter.get_display_content()
    }
}
impl EventSender for SonamuMediator {
    fn send_event(&self, event: UIEvent) {
        self.event_sender.send_event(event);
    }
}

/// This interface is called by the server
/// (but the actual struct implementing this will probably let people use this)
pub trait DisplayContentGetter: Sync + Send {
    fn is_damaged(&self) -> bool;

    /// Should set `is_damaged` to false
    fn get_display_content(&self) -> UIElement;
}

/// Called by client when `is_damaged` goes from false to true.
/// No need to call this when you update content and `is_damaged` is already true.
/// You're allowed to call it, it'll just be redundant.
pub trait DamgeCallback: Send + Sync {
    fn damage(&self);
}

/// Default implementation of display getter that the server can give to client
/// ^- not exactly anymore, but this is a simple one that the client initializer can make
/// TODO: These should really go in client toolkit now
#[derive(Clone)]
pub struct CacheDisplayCommunicator {
    is_damaged: Arc<AtomicBool>,
    display_content: EncapsulatedLock<UIElement>,
}

impl Default for CacheDisplayCommunicator {
    fn default() -> Self {
        Self::new(UIElement::Nothing)
    }
}
impl DisplayContentGetter for CacheDisplayCommunicator {
    fn is_damaged(&self) -> bool {
        self.is_damaged.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn get_display_content(&self) -> UIElement {
        // I already wrote the logic somewhere else of why we should set is_damaged to false and then get the content
        self.is_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.display_content.get()
    }
}
impl CacheDisplayCommunicator {
    pub fn new(starting_content: UIElement) -> Self {
        Self {
            // REVIEW: should this be true, false, does it matter?
            is_damaged: Arc::new(AtomicBool::new(true)),
            display_content: EncapsulatedLock::new(starting_content),
        }
    }
    pub fn set_display_content(&self, new_content: UIElement) {
        // TODO: callback system for the parent/server
        self.display_content.set(new_content);
        self.is_damaged
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub trait EventSender: Sync + Send {
    fn send_event(&self, event: UIEvent);
}

/// Just throws away the events
pub struct NullEventCommunicator;
impl EventSender for NullEventCommunicator {
    fn send_event(&self, _: UIEvent) {}
}

/// Default implementation of display getter that the server can give to client
#[derive(Clone)]
pub struct QueuedEventCommunicator {
    event_queue: mpsc::Sender<UIEvent>,
}
impl QueuedEventCommunicator {
    pub fn new() -> (Self, mpsc::Receiver<UIEvent>) {
        let (tx, rx) = mpsc::channel();

        (Self { event_queue: tx }, rx)
    }
}
impl EventSender for QueuedEventCommunicator {
    fn send_event(&self, event: UIEvent) {
        self.event_queue.send(event).unwrap();
    }
}

/// Hooks up the mediator and client
/// This *could* just be a FnOnce, but wtv
pub trait ClientInitializer {
    /// TODO: not really sure how to deal with the generics and stuff
    /// the ownership and synchronization isn't a problem, but this just feels really suboptimal
    /// but maybe it is also just a matter of framing what each of the types are
    fn initialize(
        self: Box<Self>,
        content_damage_callback: Box<dyn DamgeCallback>,
    ) -> SonamuMediator;
}
