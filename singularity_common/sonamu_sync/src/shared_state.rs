use std::{
    ops::{Deref, DerefMut},
    ptr,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};

/// What you get when you lock the UI State
pub struct SharedStateGuard<'a, SharedState> {
    state: MutexGuard<'a, SharedState>,

    // /// Only needed for the condvar
    // shared_data: SharedData<SharedState>,
    cond: Arc<Condvar>,
}
impl<SharedState> SharedStateGuard<'_, SharedState> {
    /// Releases lock, waits until the state changes, and then returns the new state locked
    pub fn wait_for_update(&mut self) {
        // self.state = self.shared_data.inner.1.wait(self.state).unwrap();
        unsafe {
            // this is scary, and I don't even know if it's right
            // the idea is that ptr::read is like an unsafe copy kinda,
            // the point is that between the ptr::read and ptr::write, the self.state is super sus
            // REVIEW: And I think you need to worry about Drop with this, so I'm lowkey worried but idk
            let old_state = ptr::read(&raw const self.state);

            let new_state = self.cond.wait(old_state).unwrap();

            ptr::write(&raw mut self.state, new_state);
        }
    }
}
impl<SharedState> Deref for SharedStateGuard<'_, SharedState> {
    type Target = SharedState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl<SharedState> DerefMut for SharedStateGuard<'_, SharedState> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

/// REVIEW: rename this to like `NotifiableMutex`?
pub struct SharedData<SharedState> {
    /// I realized it looks nicer to use `inner` over a tuple struct of one
    inner: Arc<Mutex<SharedState>>,
    /// Since conditions might be shared, we can't just have a `Arc<(Mutex<SharedState>, Condvar)>` and assume they both have same lifetime
    cond: Arc<Condvar>,
}
// manual impl bc normal Clone adds a `where SharedState: Clone`
impl<SharedState> Clone for SharedData<SharedState> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cond: self.cond.clone(),
        }
    }
}
impl<SharedState> SharedData<SharedState> {
    pub fn new(initial_state: SharedState) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial_state)),
            cond: Arc::new(Condvar::new()),
        }
    }

    pub fn new_with_same_cond<OtherState>(
        initial_state: SharedState,
        other_cond: &SharedData<OtherState>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial_state)),
            cond: other_cond.cond.clone(),
        }
    }

    pub fn lock_state(&self) -> SharedStateGuard<'_, SharedState> {
        let state = self.inner.lock().unwrap();
        // // Since for this case, there's just one edge, notify_one should suffice but just have this to be safe
        // // The nice thing about condvar is that this will only wake threads up once the Lock is dropped
        // self.inner.1.notify_all();
        // ^^^ will cause the two threads to continuously update each other even if they don't change anything
        // Now that UIStateGuard is its own struct, I could also have implemented Drop, but I won't have to do either.
        SharedStateGuard {
            state,
            // REVIEW: Will this be a circular RC? I'm kinda turning off my brain (it's easy to write random stuff and hard to figure out if it should work)
            cond: self.cond.clone(),
        }
    }

    // pub fn set_ended(&self) {
    //     *self.inner.0.lock().unwrap() = UIState::Ended;

    //     self.notify();
    // }

    pub fn notify(&self) {
        // Since for this case, there's just one edge, notify_one should suffice but just have this to be safe
        // ^^^ no longer applicable now that ts is generalized
        self.cond.notify_all();
    }
}
