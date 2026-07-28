//! A middle node with two edges: the shape `singularity_sar`'s main loop has.
//!
//! ```text
//!     ui  <--(events, frame)-->  runner  <--(events, content)-->  applet
//! ```
//!
//! The runner sleeps once and wakes for whichever side moved, and neither side
//! can lose an update by writing while the runner is between "handled
//! everything" and "asleep".
//!
//! Note that the runner never holds both edges at once, and that "is it still
//! running" is duplicated per edge rather than shared: the ui and the applet are
//! not neighbours, so they must never be able to lock each other.
//!
//! Run with: `cargo run -p sonamu_sync --example tree`

use sonamu_sync::node_edge::{EdgeEnd, Node};
use std::{ops::ControlFlow, thread, time::Duration};

const EVENTS: usize = 5;

/// The state shared by the ui thread and the runner.
#[derive(Default)]
struct UiEdge {
    /// ui -> runner
    events: Vec<String>,
    /// runner -> ui
    frame: Option<String>,
    /// runner -> ui
    quit: bool,
}

/// The state shared by the runner and the applet.
#[derive(Default)]
struct AppletEdge {
    /// runner -> applet
    events: Vec<String>,
    /// applet -> runner
    content: Option<String>,
    /// applet -> runner: the applet asked to be closed
    closed: bool,
}

fn main() {
    let ui = Node::new();
    let runner = Node::new();
    let applet = Node::new();

    // the tree: ui --- runner --- applet
    let (runner_to_ui, ui_end) = runner.connect_to(&ui, UiEdge::default());
    let (runner_to_applet, applet_end) = runner.connect_to(&applet, AppletEdge::default());

    let ui_thread = thread::spawn(move || run_ui(&ui, &ui_end));
    let applet_thread = thread::spawn(move || run_applet(&applet, &applet_end));

    run_runner(&runner, &runner_to_ui, &runner_to_applet);

    ui_thread.join().unwrap();
    applet_thread.join().unwrap();
    println!("[runner] everyone is home");
}

/// Pretends to be a window: makes up an event every so often, draws frames.
fn run_ui(ui: &Node, edge: &EdgeEnd<UiEdge>) {
    {
        // a real ui thread has its own event loop, so it just spawns a poker
        let edge = edge.clone();
        thread::spawn(move || {
            for event in 0..EVENTS {
                thread::sleep(Duration::from_millis(40));
                edge.lock().events.push(format!("key #{event}"));
            }
        });
    }

    ui.run(|| {
        let mut ui_edge = edge.lock();

        if ui_edge.quit {
            println!("[ui] closing");
            return ControlFlow::Break(());
        }

        // taking the frame is consuming, not producing: `quiet_mut` so the
        // runner is not woken up by us reading its mail
        if let Some(frame) = ui_edge.quiet_mut().frame.take() {
            println!("[ui] drawing {frame:?}");
        }

        ControlFlow::Continue(())
    });
}

/// Pretends to be an applet: turns events into content, and closes itself once
/// it has had enough.
fn run_applet(applet: &Node, edge: &EdgeEnd<AppletEdge>) {
    let mut handled = 0;

    applet.run(|| {
        let mut applet_edge = edge.lock();

        for event in applet_edge.quiet_mut().events.drain(..) {
            handled += 1;
            println!("[applet] handling {event:?}");
        }

        if handled == 0 {
            return ControlFlow::Continue(());
        }

        // `DerefMut`, so dropping the guard wakes the runner up for us
        applet_edge.content = Some(format!("{handled} event(s) so far"));

        if handled == EVENTS {
            println!("[applet] that is enough, closing");
            applet_edge.closed = true;
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    });
}

/// The middle node: it has two edges and no idea which one will move next.
fn run_runner(runner: &Node, to_ui: &EdgeEnd<UiEdge>, to_applet: &EdgeEnd<AppletEdge>) {
    runner.run(|| {
        // Handle *everything* that could have changed, one edge at a time.
        // Anything that lands while we are in here bumps our poke count, so the
        // `wait` at the bottom of `run` will not sleep on it.

        let events = std::mem::take(&mut to_ui.lock().quiet_mut().events);
        if !events.is_empty() {
            to_applet.lock().events.extend(events);
        }

        let (content, closed) = {
            let mut applet_edge = to_applet.lock();
            let applet_edge = applet_edge.quiet_mut();

            (applet_edge.content.take(), applet_edge.closed)
        };

        if let Some(content) = content {
            to_ui.lock().frame = Some(content);
        }

        if closed {
            println!("[runner] applet closed, telling the ui to quit");
            to_ui.lock().quit = true;
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    });
}
