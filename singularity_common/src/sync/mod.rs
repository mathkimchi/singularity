use std::{
    ops::Deref,
    sync::{Arc, atomic::AtomicUsize},
};

/// The data in a clam, just not wrapped in Arc.
struct ClamFields<T, CleanUpHook: Fn()> {
    inner: T,
    open_counter: AtomicUsize,
    cleanup_hook: CleanUpHook,
}

#[derive(Clone)]
pub struct Clam<T, CleanUpHook: Fn()> {
    fields: Arc<ClamFields<T, CleanUpHook>>,
}
impl<T, CleanUpHook: Fn()> Clam<T, CleanUpHook> {
    pub fn new(inner: T, cleanup_hook: CleanUpHook) -> Self {
        Self {
            fields: Arc::new(ClamFields {
                inner,
                open_counter: AtomicUsize::new(0),
                cleanup_hook,
            }),
        }
    }

    pub fn get_pearl(&self) -> Pearl<T, CleanUpHook> {
        Pearl::from_clam(self)
    }
}

pub struct Pearl<T, CleanUpHook: Fn()> {
    fields: Arc<ClamFields<T, CleanUpHook>>,
}
impl<T, CleanUpHook: Fn()> Pearl<T, CleanUpHook> {
    pub fn from_clam(clam: &Clam<T, CleanUpHook>) -> Self {
        clam.fields
            .open_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            fields: clam.fields.clone(),
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
            .open_counter
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
            == 1
        {
            (self.fields.cleanup_hook)();
        }
    }
}
