//! Tests for the node-and-edge sync primitives.
//!
//! A lost update shows up as a thread that sleeps forever, so nearly every test
//! here runs on its own thread behind a deadline: "hung" is the failure mode we
//! are actually hunting.

use sonamu_sync::node_edge::Node;
use std::{
    ops::ControlFlow,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_secs(10);

/// Runs `body` on another thread and fails if it does not finish in time.
fn before_deadline<Out: Send + 'static>(body: impl FnOnce() -> Out + Send + 'static) -> Out {
    let (finished, result) = mpsc::channel();

    thread::spawn(move || finished.send(body()));

    result
        .recv_timeout(DEADLINE)
        .expect("test never finished: an update was lost (or a thread panicked)")
}

/// The exact race the whole design exists for: the peer writes in the window
/// between "I have handled everything" and "I am asleep".
#[test]
fn update_landing_before_the_sleep_is_not_lost() {
    before_deadline(|| {
        let node = Node::new();
        let peer = Node::new();
        let (mine, theirs) = node.connect_to(&peer, 0_u32);

        // I look at my edges and find nothing...
        let watch = node.watch();
        assert_eq!(*mine.lock(), 0);

        // ...and *right then*, before I get to sleep, the peer writes.
        thread::spawn(move || *theirs.lock() = 1).join().unwrap();

        // So this must not sleep.
        let _ = node.wait(watch);
        assert_eq!(*mine.lock(), 1);
    });
}

/// Same race, but hammered from several edges at once, which is where sloppy
/// flag-clearing schemes fall over.
#[test]
fn no_update_is_ever_lost_under_load() {
    const WRITERS: u32 = 4;
    const PER_WRITER: u32 = 500;

    before_deadline(|| {
        let node = Node::new();

        let mut ends = Vec::new();
        for writer in 0..WRITERS {
            let peer = Node::new();
            let (mine, theirs) = node.connect_to(&peer, Vec::<u32>::new());
            ends.push(mine);

            thread::spawn(move || {
                // vary the timing so writes land all over the reader's loop,
                // including inside the check-then-sleep window
                let mut jitter = writer + 1;
                for message in 0..PER_WRITER {
                    theirs.lock().push(message);

                    jitter ^= jitter << 13;
                    jitter ^= jitter >> 17;
                    jitter ^= jitter << 5;
                    for _ in 0..(jitter % 8) {
                        thread::yield_now();
                    }
                }
            });
        }

        let mut received = 0;
        node.run(|| {
            for end in &ends {
                // draining my own inbox is not news to the writer
                received += end.lock().quiet_mut().drain(..).count() as u32;
            }

            if received == WRITERS * PER_WRITER {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
    });
}

/// Every edge of a node has to be able to wake it, not just the last one touched.
#[test]
fn any_edge_wakes_the_node() {
    before_deadline(|| {
        let node = Node::new();

        let ends: Vec<_> = (0..3)
            .map(|_| {
                let peer = Node::new();
                node.connect_to(&peer, false)
            })
            .collect();

        for (_, theirs) in &ends {
            let theirs = theirs.clone();
            // stagger them so the node genuinely sleeps between updates
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(20));
                *theirs.lock() = true;
            });
        }

        let mut watch = node.watch();
        while !ends.iter().all(|(mine, _)| *mine.lock()) {
            watch = node.wait(watch);
        }
    });
}

#[test]
fn reading_does_not_wake_the_peer() {
    let node = Node::new();
    let peer = Node::new();
    let (mine, theirs) = node.connect_to(&peer, 7_u32);

    let peer_watch = peer.watch();
    let node_watch = node.watch();

    assert_eq!(*mine.lock(), 7);
    assert_eq!(*theirs.lock(), 7);

    assert!(!peer.has_update(peer_watch), "a read woke the peer");
    assert!(!node.has_update(node_watch), "a read woke the node");
}

#[test]
fn writing_wakes_only_the_peer() {
    let node = Node::new();
    let peer = Node::new();
    let (mine, _theirs) = node.connect_to(&peer, 0_u32);

    let peer_watch = peer.watch();
    let node_watch = node.watch();

    *mine.lock() = 1;

    assert!(peer.has_update(peer_watch), "the peer was not woken");
    assert!(
        !node.has_update(node_watch),
        "a node woke itself up with its own write"
    );
}

#[test]
fn quiet_mut_does_not_wake_the_peer() {
    let node = Node::new();
    let peer = Node::new();
    let (mine, _theirs) = node.connect_to(&peer, 0_u32);

    let watch = peer.watch();
    *mine.lock().quiet_mut() = 1;
    assert!(!peer.has_update(watch), "`quiet_mut` woke the peer");

    // ...unless you ask for it after the fact
    let mut guard = mine.lock();
    *guard.quiet_mut() = 2;
    guard.poke_peer();
    drop(guard);
    assert!(peer.has_update(watch), "`poke_peer` did not wake the peer");
}

/// An update that arrives *while* the node is busy is remembered, not dropped.
#[test]
fn updates_during_handling_are_remembered() {
    let node = Node::new();
    let peer = Node::new();
    let (mine, theirs) = node.connect_to(&peer, 0_u32);

    let watch = node.watch(); // taken before handling, as the API insists

    // handling the first update...
    assert_eq!(*mine.lock(), 0);
    // ...during which a second one arrives
    *theirs.lock() = 1;

    assert!(
        node.has_update(watch),
        "an update that arrived mid-handling was forgotten"
    );
}

#[test]
fn wait_times_out_when_nothing_happens() {
    before_deadline(|| {
        let node = Node::new();
        let _ends = node.connect_to(&Node::new(), ());

        let started = Instant::now();
        let (watch, timed_out) = node.wait_timeout(node.watch(), Duration::from_millis(50));

        assert!(timed_out, "claimed an update that never happened");
        assert!(started.elapsed() >= Duration::from_millis(50));

        // and it still works afterwards
        let (_, timed_out) = node.wait_timeout(watch, Duration::from_millis(10));
        assert!(timed_out);
    });
}

#[test]
fn a_node_can_wake_itself() {
    before_deadline(|| {
        let node = Node::new();
        let poker = node.poker();

        let watch = node.watch();
        poker.poke();

        let _ = node.wait(watch); // must not sleep
    });
}

/// Both directions of one edge, plus a shutdown handshake: the shape real code
/// will use.
#[test]
fn parent_and_child_can_talk_both_ways() {
    #[derive(Default)]
    struct Link {
        to_parent: Vec<u32>,
        stop: bool,
    }

    before_deadline(|| {
        let parent = Node::new();
        let child = Node::new();
        let (parent_end, child_end) = parent.connect_to(&child, Link::default());

        let child_thread = thread::spawn(move || {
            let mut sent = 0;
            child.run(|| {
                let mut link = child_end.lock();

                if link.stop {
                    return ControlFlow::Break(sent);
                }

                if sent < 5 {
                    sent += 1;
                    link.to_parent.push(sent);
                }

                ControlFlow::Continue(())
            })
        });

        let mut received = Vec::new();
        parent.run(|| {
            let mut link = parent_end.lock();
            received.append(&mut link.quiet_mut().to_parent);

            if received.len() < 5 {
                // reading the outbox is not an update, but "your outbox is empty
                // again" is: the child only sends once the parent has caught up
                link.poke_peer();
                return ControlFlow::Continue(());
            }

            link.stop = true; // `DerefMut`, so the child is woken on unlock
            ControlFlow::Break(())
        });

        assert_eq!(received, vec![1, 2, 3, 4, 5]);
        assert_eq!(child_thread.join().unwrap(), 5);
    });
}

/// Sleeping while holding an edge is a guaranteed hang, so it is a panic instead.
#[test]
#[should_panic(expected = "tried to sleep while holding 1 edge(s)")]
fn sleeping_while_holding_an_edge_panics() {
    let node = Node::new();
    let (mine, _theirs) = node.connect_to(&Node::new(), ());

    let watch = node.watch();
    let _guard = mine.lock();

    let _ = node.wait(watch);
}

#[test]
fn guards_stop_being_held_once_dropped() {
    let node = Node::new();
    let (mine, _theirs) = node.connect_to(&Node::new(), ());

    for _ in 0..3 {
        let guard = mine.lock();
        drop(guard);
    }

    // would panic if the bookkeeping leaked
    let _ = node.wait_timeout(node.watch(), Duration::from_millis(1));
}
