use std::{
    ops::Deref,
    sync::{Arc, RwLock, atomic::AtomicUsize},
};

pub mod shared_graph;
// pub mod shared_state; // shared_graph mogs

/// Lock (RwLock) but you can only call getter and setter,
/// so this is guranteed to prevent deadlocks.
/// (Don't hold me liable for the above statement.)
///
/// NOTE: technically, I don't need the Arc here bc I could use
/// `Arc<EncapsulatedLock<T>>` whenever.
/// And if I was writing a library,
/// coupling EncapsulatedLock with Arc is bad practice,
/// but I am fine with this.
///
/// TODO: decouple `EncapsulatedLock` and `Arc`.
#[derive(Clone)]
pub struct EncapsulatedLock<T: Clone> {
    inner: Arc<RwLock<T>>,
}
impl<T: Clone> EncapsulatedLock<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    /// Returns a clone of the held object.
    #[must_use]
    pub fn get(&self) -> T {
        self.inner.read().unwrap().clone()
    }

    pub fn set(&self, object: T) {
        *self.inner.write().unwrap() = object;
    }
}

/// The data in a clam, just not wrapped in Arc.
struct ClamFields<T, CleanUpHook: Fn()> {
    inner: T,
    pearl_counter: AtomicUsize,
    cleanup_hook: CleanUpHook,
}

/// Allows for setting up calling a clean-up hook.
///
/// Think of Clam as the dormant state
/// (you might access the inner data later but not now)
/// while Pearl means you are actively accessing it.
/// When all Pearls are gone, the cleanup function is called.
///
/// NOTE: Look at DEVLOG 2026-01-21 for a better explanation.
///
/// NOTE: When you wrap data in a Clam, you don't need Arc.
#[derive(Clone)]
pub struct Clam<T, CleanUpHook: Fn()> {
    fields: Arc<ClamFields<T, CleanUpHook>>,
}
impl<T, CleanUpHook: Fn()> Clam<T, CleanUpHook> {
    pub fn new(inner: T, cleanup_hook: CleanUpHook) -> Self {
        Self {
            fields: Arc::new(ClamFields {
                inner,
                pearl_counter: AtomicUsize::new(0),
                cleanup_hook,
            }),
        }
    }

    #[must_use]
    pub fn get_pearl(&self) -> Pearl<T, CleanUpHook> {
        Pearl::from_clam(self)
    }

    /// Returns whether or not clam is currently dormant.
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        self.fields
            .pearl_counter
            .load(std::sync::atomic::Ordering::Relaxed)
            == 0
    }
}

/// Pearl means you are actively accessing the inner data.
pub struct Pearl<T, CleanUpHook: Fn()> {
    fields: Arc<ClamFields<T, CleanUpHook>>,
}
impl<T, CleanUpHook: Fn()> Clone for Pearl<T, CleanUpHook> {
    fn clone(&self) -> Self {
        self.fields
            .pearl_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            fields: self.fields.clone(),
        }
    }
}
impl<T, CleanUpHook: Fn()> Pearl<T, CleanUpHook> {
    #[must_use]
    pub fn from_clam(clam: &Clam<T, CleanUpHook>) -> Self {
        clam.fields
            .pearl_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            fields: clam.fields.clone(),
        }
    }

    #[must_use]
    pub fn get_clam(&self) -> Clam<T, CleanUpHook> {
        Clam {
            fields: self.fields.clone(),
        }
    }
}
impl<T, CleanUpHook: Fn()> Deref for Pearl<T, CleanUpHook> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.fields.inner
    }
}
impl<T, CleanUpHook: Fn()> Drop for Pearl<T, CleanUpHook> {
    fn drop(&mut self) {
        // if it was previously 1, it is now 0
        if self
            .fields
            .pearl_counter
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
            == 1
        {
            (self.fields.cleanup_hook)();
        }
    }
}
