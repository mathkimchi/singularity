//! Look at 2026-07-20 DEVLOG entry for conception.
//! Issue [#40](https://github.com/mathkimchi/singularity/issues/40)
//! tracks the initial implementation of this.

use crate::sync::EncapsulatedLock;
use singularity_ui::ui_element::UIElement;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

/// Made by server with default impls,
/// given to client initializer which can use and modify the mediator.
///
/// I currerntly have each protocol as it's own component.
pub struct SonamuMediator {
    display_getter: Box<dyn DisplayContentGetter>,
}
impl SonamuMediator {
    pub fn new(display_getter: impl DisplayContentGetter + 'static) -> Self {
        Self {
            display_getter: Box::new(display_getter),
        }
    }

    pub fn split(self) -> (ServerSideMediator, ClientSideMediator) {
        let shared_self = Arc::new(Mutex::new(self));

        (
            ServerSideMediator(shared_self.clone()),
            ClientSideMediator(shared_self),
        )
    }
}

/// Used by server
pub struct ServerSideMediator(Arc<Mutex<SonamuMediator>>);
// Server Side Mediator is a proxy, is this bad?
impl DisplayContentGetter for ServerSideMediator {
    fn is_damaged(&self) -> bool {
        self.0.lock().unwrap().display_getter.is_damaged()
    }

    fn get_display_content(&self) -> UIElement {
        self.0.lock().unwrap().display_getter.get_display_content()
    }
}

/// Used by client
pub struct ClientSideMediator(Arc<Mutex<SonamuMediator>>);
impl ClientSideMediator {
    pub fn set_display_getter(&self, display_getter: impl DisplayContentGetter + 'static) {
        self.0.lock().unwrap().display_getter = Box::new(display_getter);
    }
}

/// This interface is called by the server
/// (but the actual struct implementing this will probably let people use this)
pub trait DisplayContentGetter: Sync + Send {
    fn is_damaged(&self) -> bool;

    /// Should set `is_damaged` to false
    fn get_display_content(&self) -> UIElement;
}

/// Default implementation of display getter that the Sonamu Mediator will use
#[derive(Clone)]
pub struct CacheDisplay {
    is_damaged: Arc<AtomicBool>,
    display_content: EncapsulatedLock<UIElement>,
}
impl Default for CacheDisplay {
    fn default() -> Self {
        Self {
            is_damaged: Arc::new(AtomicBool::new(false)),
            display_content: EncapsulatedLock::new(UIElement::Nothing),
        }
    }
}
impl DisplayContentGetter for CacheDisplay {
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
impl CacheDisplay {
    pub fn set_display_content(&self, new_content: UIElement) {
        // TODO: callback system for the parent/server
        self.display_content.set(new_content);
        self.is_damaged
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Hooks up the mediator and client
/// This *could* just be a FnOnce, but wtv
pub trait ClientInitializer {
    /// TODO: not really sure how to deal with the generics and stuff
    /// the ownership and synchronization isn't a problem, but this just feels really suboptimal
    /// but maybe it is also just a matter of framing what each of the types are
    fn initialize(self: Box<Self>, mediator: ClientSideMediator, display_getter: CacheDisplay);
}

/*
TODO: currently give client and server access to everything and just expect them to call the right things, but I should split them and only open up necessary interface like mpsc, maybe this is the facade pattern

/// Mediator used by the client
pub struct ClientSideMediator {
    inner: Arc<Mutex<SonamuMediator>>,
}
impl ClientSideMediator {
    pub fn set_display_content_getter(&self, new_getter: ) {}
}

/// Mediator used by the server
pub struct ServerSideMediator {
    inner: Arc<Mutex<SonamuMediator>>,
}
impl ServerSideMediator {
    pub fn get_display_content(&self) -> UIElement {
        self.inner
            .lock()
            .unwrap()
            .display_getter
            .get_display_content()
    }
}
*/
