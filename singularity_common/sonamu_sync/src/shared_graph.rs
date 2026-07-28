//! For the Node and edge idea I was talking about in DEVLOG (some day before 2026-07-27)
//!
//! Setup is that we have nodes that are threads and edges that are shared resources.
//! A node should be able to wait until any of it's neighbors update their shared edge.
//!
//! It is crucial that we don't miss a notification.
//!
//! My solution for this with Condvar (this won't work in tokio)
//! is to have a lock on each node and each edge.
//! There is a condvar for each node,
//! and each edge stores its endpoint's condvars.
//! When a node waits, it waits on its condvar and lock.
//! So, when notified, the node thread awakens holing its own lock.
//!
//! It will then want to acquire its edges,
//! which it can just do sequentially.
//! (This won't cause deadlock if graph is a tree.)
//!
//! The node thread should release all the edges before waiting.
//!
//! When one node updates an edge, it should notify the node on the other end.

use std::sync::{Arc, Condvar, Mutex};

/// Represents a thread.
pub struct SyncNode {
    lock: Mutex<()>,
    cond: Arc<Condvar>,
}

pub struct Edge<Data> {
    data: Mutex<Data>,
    condvars: [Arc<Condvar>; 2],
}
