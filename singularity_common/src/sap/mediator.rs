//! Look at 2026-07-20 DEVLOG entry for conception.
//! Issue [#40](https://github.com/mathkimchi/singularity/issues/40)
//! tracks the initial implementation of this.

use crate::sync::EncapsulatedLock;
use sonamu_ui::{ui_element::UIElement, ui_event::UIEvent};
use std::sync::{Arc, atomic::AtomicBool, mpsc};

struct MediatorSharedData {
    /// TODO: use double buffer?
    display_content: EncapsulatedLock<UIElement>,
    /// TODO: figure out Arc later
    display_damaged: AtomicBool,

    /// Called by the client notifying server that display damaged went from false to true
    display_server_callback: Box<dyn DamgeCallback>,
    display_client_callbacks: Box<dyn DisplayProtocolClientCallbacks>,

    event_sender: Box<dyn EventSender>,
}

/// ~~Made by server with default impls,
/// given to client initializer which can use and modify the mediator.~~
///
/// Just make the client initializer makes ts bruh.
/// (Duh doy)
/// I'm not going to update comments other than this,
/// TODO: update documentation
/// ^-- funny joke, what documentation?
///
/// I currerntly have each protocol as it's own component.
///
/// This really holds the server side (side called by server) of the protocols,
/// but client is given this so they can replace the default implementations.
///
/// Generally, the interfaces like `DisplayContentGetter` and `EventSender`
/// are for the server to call and client to implement (or leave as default).
/// The server doesn't know the impl beyond the interface
/// while the client should know the actual type of it
/// (either because it is kept the default impl struct or because the client re-implemented)
#[derive(Clone)]
pub struct SonamuMediator {
    inner: Arc<MediatorSharedData>,
}
impl SonamuMediator {
    pub fn new(
        display_server_callback: Box<dyn DamgeCallback>,
        display_client_callbacks: impl DisplayProtocolClientCallbacks + 'static,
        event_sender: impl EventSender + 'static,
    ) -> Self {
        Self {
            inner: Arc::new(MediatorSharedData {
                // just set initial content as nothing
                display_content: EncapsulatedLock::new(UIElement::Nothing),
                display_damaged: AtomicBool::new(false),
                display_server_callback,
                display_client_callbacks: Box::new(display_client_callbacks),
                event_sender: Box::new(event_sender),
            }),
        }
    }

    /// NOTE: Assumes whoever is calling this (server) will process it
    pub fn get_display_content(&self) -> UIElement {
        let content = self.inner.display_content.get();
        // self.content_processed();
        self.inner
            .display_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.inner.display_client_callbacks.content_processed();
        content
    }

    /// client should call this
    /// REVIEW: also have a force update?
    /// Returns whether or not display was actually updated
    pub fn try_update_display_content(&self, content_getter: impl FnOnce() -> UIElement) -> bool {
        if self
            .inner
            .display_damaged
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            // there are already unprocessed display updates, so don't update this
            false
        } else {
            // only update display if all previous updates have been processed by server
            self.inner.display_content.set(content_getter());

            self.inner
                .display_damaged
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.inner.display_server_callback.damage();

            true
        }
    }
}

// // Server Side Mediator is proxy pattern, is this bad?
// impl DisplayProtocolClientCallbacks for SonamuMediator {
//     fn content_processed(&self) {
//         self.display_damaged
//             .store(false, std::sync::atomic::Ordering::Relaxed);
//         self.display_client_callbacks.content_processed();
//     }
// }
impl EventSender for SonamuMediator {
    fn send_event(&self, event: UIEvent) {
        self.inner.event_sender.send_event(event);
    }
}

/// This interface is called by the server
/// (but the actual struct implementing this will probably let people use this)
pub trait DisplayProtocolClientCallbacks: Sync + Send {
    /// Called by the server when it reads the updated content
    /// and sets damaged to false
    fn content_processed(&self);
}

/// Called by client when `is_damaged` goes from false to true.
/// No need to call this when you update content and `is_damaged` is already true.
/// You're allowed to call it, it'll just be redundant.
pub trait DamgeCallback: Send + Sync {
    fn damage(&self);
}

pub struct NullDamageCallback;
impl DamgeCallback for NullDamageCallback {
    fn damage(&self) {}
}

// pub struct NullClientDisplayCallback;
// impl DisplayProtocolClientCallbacks for NullClientDisplayCallback {
//     fn content_processed(&self) {}
// }

// /// Default implementation of display getter that the server can give to client
// /// ^- not exactly anymore, but this is a simple one that the client initializer can make
// /// TODO: These should really go in client toolkit now
// #[derive(Clone)]
// pub struct CacheDisplayCommunicator {
//     is_damaged: Arc<AtomicBool>,
//     display_content: EncapsulatedLock<UIElement>,
// }

// impl Default for CacheDisplayCommunicator {
//     fn default() -> Self {
//         Self::new(UIElement::Nothing)
//     }
// }
// impl DisplayProtocolClientCallbacks for CacheDisplayCommunicator {
//     fn is_damaged(&self) -> bool {
//         self.is_damaged.load(std::sync::atomic::Ordering::Relaxed)
//     }

//     fn get_display_content(&self) -> UIElement {
//         // I already wrote the logic somewhere else of why we should set is_damaged to false and then get the content
//         self.is_damaged
//             .store(false, std::sync::atomic::Ordering::Relaxed);
//         self.display_content.get()
//     }
// }
// impl CacheDisplayCommunicator {
//     pub fn new(starting_content: UIElement) -> Self {
//         Self {
//             // REVIEW: should this be true, false, does it matter?
//             is_damaged: Arc::new(AtomicBool::new(true)),
//             display_content: EncapsulatedLock::new(starting_content),
//         }
//     }
//     pub fn set_display_content(&self, new_content: UIElement) {
//         // TODO: callback system for the parent/server
//         self.display_content.set(new_content);
//         self.is_damaged
//             .store(true, std::sync::atomic::Ordering::Relaxed);
//     }
// }

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
