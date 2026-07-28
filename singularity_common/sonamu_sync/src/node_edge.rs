//! Sleeping on a tree of threads without ever missing an update.
//!
//! # The model
//!
//! - A **node** is a thread. It owns exactly one [`Node`].
//! - An **edge** is state shared by exactly two nodes. Access to it is mutually
//!   exclusive, and each of the two nodes holds its own [`EdgeEnd`] of it.
//! - A node may sleep until *any* of its edges is written to by a peer.
//!
//! Since the graph is a tree, locking a single edge can never deadlock,
//! and this module never asks you to hold two locks at once.
//!
//! # The bug this exists to kill
//!
//! The obvious implementation (one mutex + one condvar per edge, wait on the
//! condvar when there is nothing to do) drops updates:
//!
//! ```text
//!  node A                              node B
//!  ------                              ------
//!  lock edge, handle everything
//!  unlock edge
//!                                      lock edge, write, unlock edge
//!                                      notify   <-- nobody is listening yet, so this is lost
//!  sleep
//!  ...zzz (forever, even though there is work waiting)
//! ```
//!
//! The window is between "I decided there was nothing left to do" and "I am
//! actually asleep", so no amount of re-checking the edges closes it: the check
//! and the sleep have to be *one atomic step*.
//!
//! # The fix
//!
//! Every node gets a **second, tiny lock of its very own**, holding nothing but a
//! counter of how many times peers have poked this node. Peers bump that counter
//! (never the sleeper), and it is the *only* thing the condvar is attached to.
//!
//! You take a [`Watch`] (a snapshot of the counter) **before** you look at your
//! edges, and hand it back to [`Node::wait`] **after**:
//!
//! ```text
//!  node A                              node B
//!  ------                              ------
//!  watch = 7
//!  lock edge, handle everything
//!  unlock edge
//!                                      lock edge, write, unlock edge
//!                                      counter := 8
//!  wait(7): counter is 8, so do not sleep -- go around again
//! ```
//!
//! and if B is a moment later instead, it has to take A's node lock to bump the
//! counter, which A holds until the instant it falls asleep -- so the notify
//! lands on a node that is provably already listening. There is no third case:
//! either the counter moved before A slept (A does not sleep) or after (A is
//! asleep and gets woken). **A cannot miss an update.**
//!
//! The rest is bookkeeping:
//!
//! - Writes notify the peer *automatically*, when the [`EdgeGuard`] is dropped
//!   (see [`EdgeGuard::quiet_mut`] for the "I am consuming, not producing" case),
//!   so you cannot forget to.
//! - The notify happens *after* the write is published and the edge is unlocked.
//! - A node lock is a **leaf**: no lock is ever taken while one is held, so
//!   adding these locks cannot introduce a deadlock cycle.
//! - Falling asleep while holding an edge would wedge the peer forever, so
//!   [`Node::wait`] panics instead of hanging.
//!
//! # Example
//!
//! ```
//! use sonamu_sync::node_edge::Node;
//! use std::ops::ControlFlow;
//!
//! let parent = Node::new();
//! let child = Node::new();
//! // each side gets its own handle to the same state
//! let (parent_end, child_end) = parent.connect_to(&child, Vec::<u32>::new());
//!
//! std::thread::spawn(move || {
//!     for i in 0..10 {
//!         child_end.lock().push(i); // dropping the guard wakes the parent
//!     }
//! });
//!
//! let mut seen = 0;
//! parent.run(|| {
//!     // `quiet_mut`: draining my inbox is not an update *for the child*
//!     seen += parent_end.lock().quiet_mut().drain(..).count();
//!     if seen == 10 { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
//! });
//! ```

use std::{
    cell::Cell,
    fmt,
    ops::{ControlFlow, Deref, DerefMut},
    sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

thread_local! {
    /// How many edge guards *this thread* is currently holding.
    ///
    /// Purely a bug detector, see [`Node::wait`].
    static HELD_EDGES: Cell<usize> = const { Cell::new(0) };
}

/// A thread, in the node-and-edge model. One per thread; it is what you sleep on.
///
/// Create the nodes first, [`connect_to`](Node::connect_to) them into a tree,
/// then send each `Node` off to its thread.
#[derive(Debug)]
pub struct Node {
    inner: Arc<NodeInner>,
}

/// The bit of a node that peers are allowed to touch.
#[derive(Debug)]
struct NodeInner {
    /// A node's *own* lock, separate from any edge lock.
    ///
    /// This is a **leaf lock**: nothing else is ever locked while it is held
    /// (waiting releases it), so it cannot take part in a deadlock cycle.
    state: Mutex<NodeState>,
    /// One condvar per *node*, not per edge: a node sleeps once and wakes for
    /// whichever of its edges moved.
    poked: Condvar,
}

#[derive(Debug)]
struct NodeState {
    /// How many times peers have poked this node, ever.
    ///
    /// A counter rather than a `bool` so that a node can snapshot it *before*
    /// handling its edges and still detect a poke that arrived *during* the
    /// handling. Clearing a flag afterwards would throw that poke away.
    pokes: u64,
}

impl Node {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NodeInner {
                state: Mutex::new(NodeState { pokes: 0 }),
                poked: Condvar::new(),
            }),
        }
    }

    /// Shares some state between this node and `other`, and hands each of them
    /// their own end of it.
    ///
    /// Returns `(my end, their end)`. Writing through an end wakes the node at
    /// the *other* end, which is why the two ends are not interchangeable.
    #[must_use]
    pub fn connect_to<Shared>(
        &self,
        other: &Self,
        shared: Shared,
    ) -> (EdgeEnd<Shared>, EdgeEnd<Shared>) {
        let edge = Arc::new(Mutex::new(shared));

        (
            EdgeEnd {
                edge: edge.clone(),
                peer: other.poker(),
            },
            EdgeEnd {
                edge,
                peer: self.poker(),
            },
        )
    }

    /// A handle other threads can use to wake this node up, with no state attached.
    ///
    /// [`connect_to`](Node::connect_to) already wires these up for you; reach for
    /// this when you want a bare "something happened" signal, or to wake
    /// *yourself* later (a self-poke just means the next [`wait`](Node::wait)
    /// returns immediately).
    #[must_use]
    pub fn poker(&self) -> Poker {
        Poker {
            node: self.inner.clone(),
        }
    }

    /// A snapshot of "everything I have been told about so far".
    ///
    /// **Take this before you look at your edges.** Anything that happens after
    /// this call makes the watch stale, and [`wait`](Node::wait) refuses to sleep
    /// on a stale watch -- which is exactly what stops updates from being missed.
    pub fn watch(&self) -> Watch {
        Watch {
            pokes: self.lock_state().pokes,
            node: self.id(),
        }
    }

    /// Whether a peer has poked this node since `watch` was taken.
    ///
    /// The non-blocking version of [`wait`](Node::wait), for a node that has its
    /// own reasons to keep spinning (a render loop, say).
    #[must_use]
    pub fn has_update(&self, watch: Watch) -> bool {
        debug_assert_eq!(watch.node, self.id(), "that `Watch` is another node's");

        self.lock_state().pokes != watch.pokes
    }

    /// Sleeps until a peer touches one of this node's edges, and returns a fresh
    /// [`Watch`] for the next round.
    ///
    /// Returns immediately if a poke landed after `watch` was taken -- so the
    /// "handle everything, then sleep" race cannot lose an update.
    ///
    /// # Panics
    ///
    /// If the calling thread is still holding an [`EdgeGuard`]. That would block
    /// the peer from ever writing to that edge, and therefore from ever waking
    /// you up: a guaranteed hang, reported as a panic instead.
    pub fn wait(&self, watch: Watch) -> Watch {
        assert_can_sleep();
        debug_assert_eq!(watch.node, self.id(), "that `Watch` is another node's");

        let state = self
            .inner
            .poked
            .wait_while(self.lock_state(), |state| state.pokes == watch.pokes)
            .unwrap_or_else(PoisonError::into_inner);

        Watch {
            pokes: state.pokes,
            node: self.id(),
        }
    }

    /// [`wait`](Node::wait), but gives up after `timeout`.
    ///
    /// Returns the fresh watch, and whether it timed out (ie: nothing arrived).
    ///
    /// # Panics
    ///
    /// Same as [`wait`](Node::wait).
    pub fn wait_timeout(&self, watch: Watch, timeout: Duration) -> (Watch, bool) {
        assert_can_sleep();
        debug_assert_eq!(watch.node, self.id(), "that `Watch` is another node's");

        let (state, timed_out) = self
            .inner
            .poked
            .wait_timeout_while(self.lock_state(), timeout, |state| {
                state.pokes == watch.pokes
            })
            .unwrap_or_else(PoisonError::into_inner);

        (
            Watch {
                pokes: state.pokes,
                node: self.id(),
            },
            timed_out.timed_out(),
        )
    }

    /// The whole loop, written correctly once so you do not have to.
    ///
    /// `handle_updates` runs immediately, and then again every time a peer
    /// touches one of your edges. Handle *everything* that changed in there:
    /// anything that arrives while it runs is remembered, not lost.
    ///
    /// Returns whatever `handle_updates` breaks with.
    pub fn run<Out>(&self, mut handle_updates: impl FnMut() -> ControlFlow<Out>) -> Out {
        // NOTE: the watch is deliberately taken *before* the first call, and
        // `wait` hands back a watch taken at the instant it woke up, so every
        // call is covered by a watch that predates it.
        let mut watch = self.watch();

        loop {
            match handle_updates() {
                ControlFlow::Break(out) => return out,
                ControlFlow::Continue(()) => watch = self.wait(watch),
            }
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, NodeState> {
        // The state is one counter, so a peer that panicked mid-poke can not have
        // left it in a nonsense state. Ignoring the poison keeps one crashed
        // thread from taking down the whole tree.
        self.inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Identifies the node, so a [`Watch`] can check it came from the right one.
    fn id(&self) -> usize {
        Arc::as_ptr(&self.inner) as usize
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

/// Panics if the calling thread is about to fall asleep holding an edge, which
/// would stop its peer from ever writing to it, and therefore from ever waking
/// this thread up.
fn assert_can_sleep() {
    let held = HELD_EDGES.with(Cell::get);

    assert!(
        held == 0,
        "tried to sleep while holding {held} edge(s): \
         the peer can not write to an edge you are holding, \
         so it could never wake you up. \
         Drop the guard(s) first (or use `quiet_mut` if you meant to consume)."
    );
}

/// A snapshot of how much a node has been told, taken *before* it handles its
/// edges and handed to [`Node::wait`] after.
///
/// Passing it around by value is what makes "check, then sleep" a single step:
/// there is no way to ask to sleep without saying what you already knew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Watch {
    pokes: u64,
    /// Which node this came from, so mixing them up is caught in debug builds.
    node: usize,
}

/// A handle for waking a node up. Cheap to clone, and safe to hold across threads.
#[derive(Debug, Clone)]
pub struct Poker {
    node: Arc<NodeInner>,
}
impl Poker {
    /// Tells the node that something it cares about changed.
    ///
    /// Never blocks on anything but the node's own leaf lock, so it is always
    /// safe to call, even from inside the peer's own critical sections.
    pub fn poke(&self) {
        let mut state = self
            .node
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);

        // (u64 does not realistically wrap, but there is nothing to gain by
        // panicking if it somehow did: only *changes* to this number matter.)
        state.pokes = state.pokes.wrapping_add(1);

        // Notifying while still holding the lock is the version that is easiest
        // to convince yourself is correct: the sleeper cannot be between its
        // check and its sleep, because that check happens under this very lock.
        //
        // `notify_all` because nothing stops two threads from driving one node.
        self.node.poked.notify_all();
    }
}

/// One node's end of an edge: state shared with exactly one other node.
///
/// Writing through it wakes the node at the *other* end, so keep hold of your
/// own end and hand the other one to your peer's thread. Cloning gives another
/// handle to the *same* end (useful for callbacks that run on your thread).
pub struct EdgeEnd<Shared> {
    edge: Arc<Mutex<Shared>>,
    peer: Poker,
}
// manual impls: deriving would demand `Shared: Clone`/`Shared: Debug`
impl<Shared> Clone for EdgeEnd<Shared> {
    fn clone(&self) -> Self {
        Self {
            edge: self.edge.clone(),
            peer: self.peer.clone(),
        }
    }
}
impl<Shared> fmt::Debug for EdgeEnd<Shared> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgeEnd").finish_non_exhaustive()
    }
}
impl<Shared> EdgeEnd<Shared> {
    /// Takes exclusive access to the shared state, blocking until it is free.
    ///
    /// Mutating what you get back (through [`DerefMut`]) wakes the peer when the
    /// guard is dropped. Only *reading* wakes nobody.
    ///
    /// Never hold two of these at once if you can help it, and never hold one
    /// while sleeping ([`Node::wait`] will catch you).
    pub fn lock(&self) -> EdgeGuard<'_, Shared> {
        let shared = self.edge.lock().unwrap_or_else(PoisonError::into_inner);

        EdgeGuard::new(shared, &self.peer)
    }

    /// [`lock`](EdgeEnd::lock), but gives up instead of blocking if the peer is
    /// mid-write. For loops that would rather do something else than wait.
    pub fn try_lock(&self) -> Option<EdgeGuard<'_, Shared>> {
        let shared = match self.edge.try_lock() {
            Ok(shared) => shared,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return None,
        };

        Some(EdgeGuard::new(shared, &self.peer))
    }

    /// Wakes the peer without touching the shared state.
    ///
    /// For "look again" signals that are not really a change to this edge.
    pub fn poke_peer(&self) {
        self.peer.poke();
    }
}

/// Exclusive access to an edge's state.
///
/// Reading is free; the moment you take a `&mut` out of it, the peer gets woken
/// when this is dropped. If you are *consuming* rather than producing (draining
/// your inbox), use [`quiet_mut`](EdgeGuard::quiet_mut) so the peer keeps sleeping.
pub struct EdgeGuard<'edge, Shared> {
    /// `Option` so [`Drop`] can release the edge *before* waking the peer;
    /// otherwise the peer wakes up only to block on the lock we still hold.
    shared: Option<MutexGuard<'edge, Shared>>,
    peer: &'edge Poker,
    poke_on_unlock: bool,
}
impl<'edge, Shared> EdgeGuard<'edge, Shared> {
    fn new(shared: MutexGuard<'edge, Shared>, peer: &'edge Poker) -> Self {
        HELD_EDGES.with(|held| held.set(held.get() + 1));

        Self {
            shared: Some(shared),
            peer,
            poke_on_unlock: false,
        }
    }

    /// Mutable access that does **not** wake the peer.
    ///
    /// Use it when the change is not news to them: draining a queue they wrote,
    /// clearing a damage flag, marking their message as read. Waking them for
    /// that is how two nodes end up notifying each other in a loop forever.
    pub fn quiet_mut(&mut self) -> &mut Shared {
        self.shared.as_mut().expect("guard is alive")
    }

    /// Wakes the peer when this guard is dropped, even if you only used
    /// [`quiet_mut`](EdgeGuard::quiet_mut).
    ///
    /// For deciding *after* the fact that a change was worth reporting.
    pub fn poke_peer(&mut self) {
        self.poke_on_unlock = true;
    }
}
impl<Shared> Deref for EdgeGuard<'_, Shared> {
    type Target = Shared;

    fn deref(&self) -> &Shared {
        self.shared.as_ref().expect("guard is alive")
    }
}
impl<Shared> DerefMut for EdgeGuard<'_, Shared> {
    /// Taking a `&mut` counts as an update, so the peer is woken on unlock.
    /// That is deliberate: forgetting to notify is the bug that started all this.
    fn deref_mut(&mut self) -> &mut Shared {
        self.poke_on_unlock = true;

        self.shared.as_mut().expect("guard is alive")
    }
}
impl<Shared> Drop for EdgeGuard<'_, Shared> {
    fn drop(&mut self) {
        // release the edge *first*, so the peer does not wake up just to block on it
        drop(self.shared.take());

        // `try_with` because a guard could outlive TLS during thread teardown
        let _ = HELD_EDGES.try_with(|held| held.set(held.get() - 1));

        if self.poke_on_unlock {
            // The write is published (the lock above is released) *before* the
            // poke lands, so a peer that misses the write is guaranteed to see
            // the poke and come back for it.
            self.peer.poke();
        }
    }
}
