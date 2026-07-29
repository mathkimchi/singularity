//! For the Node and edge idea I was talking about in DEVLOG (some day before 2026-07-27)
//!
//! Setup is that we have nodes that are threads and edges that are shared resources.
//! A node should be able to wait until any of it's neighbors update their shared edge.
//!
//! It is crucial that we don't miss a notification.
//!
//! Look at 2026-07-28 DEVLOG for algorithm

//! For the Node and edge idea I was talking about in DEVLOG (some day before 2026-07-27)
//!
//! Setup is that we have nodes that are threads and edges that are shared resources.
//! A node should be able to wait until any of it's neighbors update their shared edge.
//!
//! It is crucial that we don't miss a notification.
//!
//! Look at 2026-07-28 DEVLOG for algorithm

use std::{
    ops::{Deref, DerefMut},
    ptr,
    sync::{Arc, Condvar, Mutex, MutexGuard},
};
// use uuid::Uuid;

struct SyncNodeInner {
    // /// I just use this to see which endpoint is calling
    // id: Uuid,
    is_updated: Mutex<bool>,
    cond: Condvar,
}

/// Represents a thread.
pub struct SyncNode {
    inner: Arc<SyncNodeInner>,
}
impl Default for SyncNode {
    fn default() -> Self {
        Self {
            inner: Arc::new(SyncNodeInner {
                is_updated: Mutex::new(false),
                cond: Condvar::new(),
            }),
        }
    }
}
impl SyncNode {
    pub fn lock(&self) -> SyncNodeGuard<'_> {
        SyncNodeGuard {
            is_updated: self.inner.is_updated.lock().unwrap(),
            node: self.inner.clone(),
        }
    }
    // pub fn lock_on_update(&self) -> SyncNodeGuard<'_> {
    //     let mut guard = self.lock();

    //     guard
    // }
}

pub struct SyncNodeGuard<'a> {
    is_updated: MutexGuard<'a, bool>,
    /// Needed for the condvar
    node: Arc<SyncNodeInner>,
}
impl SyncNodeGuard<'_> {
    /// call this before re-attempting a grab
    pub fn wait_for_notif(&mut self) {
        unsafe {
            // this is scary, and I don't even know if it's right
            // the idea is that ptr::read is like an unsafe copy kinda,
            // the point is that between the ptr::read and ptr::write, the self.state is super sus
            // REVIEW: And I think you need to worry about Drop with this, so I'm lowkey worried but idk
            let old_is_updated = ptr::read(&raw const self.is_updated);

            // During the wait, the guard is released
            // The condvar does this temp drop thing with internal functions
            let new_is_updated = self.node.cond.wait(old_is_updated).unwrap();

            ptr::write(&raw mut self.is_updated, new_is_updated);
        }
    }

    pub fn wait_for_update(&mut self) {
        while !*self.is_updated {
            unsafe {
                // this is scary, and I don't even know if it's right
                // the idea is that ptr::read is like an unsafe copy kinda,
                // the point is that between the ptr::read and ptr::write, the self.state is super sus
                // REVIEW: And I think you need to worry about Drop with this, so I'm lowkey worried but idk
                let old_is_updated = ptr::read(&raw const self.is_updated);

                let new_is_updated = self.node.cond.wait(old_is_updated).unwrap();

                ptr::write(&raw mut self.is_updated, new_is_updated);
            }
        }
    }

    pub fn mark_updates_processed(&mut self) {
        *self.is_updated = false;
    }
}

// struct SyncEdgeShared<Data> {
//     data: Mutex<Data>,
//     /// REVIEW: do I even need this and the index? Can I just store the other guy?
//     endpoints: [Arc<SyncNodeInner>; 2],
// }

/// An edge represents data shared between two nodes.
/// This struct represents one node's handle of the edge.
pub struct SyncEdge<Data> {
    // inner: Arc<SyncEdgeShared<Data>>,
    // /// 0 or 1 based on the endpoints array of 2
    // self_index: usize,
    data: Arc<Mutex<Data>>,
    other_endpoint: Arc<SyncNodeInner>,
}
impl<Data> SyncEdge<Data> {
    pub fn new_edge(initial_data: Data, endpoints: [&SyncNode; 2]) -> [Self; 2] {
        let data = Arc::new(Mutex::new(initial_data));

        [
            Self {
                data: data.clone(),
                other_endpoint: endpoints[1].inner.clone(),
            },
            Self {
                data: data.clone(),
                other_endpoint: endpoints[0].inner.clone(),
            },
        ]
    }

    pub fn try_lock(&self) -> Option<SyncEdgeGuard<'_, Data>> {
        let guard = self.data.try_lock().ok()?;

        Some(SyncEdgeGuard {
            data: guard,
            other_endpoint: self.other_endpoint.clone(),
        })
    }

pub struct SyncEdgeGuard<'a, Data> {
    data_guard: MutexGuard<'a, Data>,
    other_endpoint: Arc<SyncNodeInner>,
}
impl<Data> Deref for SyncEdgeGuard<'_, Data> {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.data_guard
    }
}
impl<Data> DerefMut for SyncEdgeGuard<'_, Data> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // This time, I'm auto doing the update via if ref or mut deref is used
        *self.other_endpoint.is_updated.lock().unwrap() = true;
        // notify is done anyways on drop so no need to do it again here
        &mut self.data_guard
    }
}