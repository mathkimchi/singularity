use singularity_common::sap::mediator::{
    CacheDisplayCommunicator, ClientInitializer, ClientSideMediator, DisplayContentGetter,
    SonamuMediator,
};
use singularity_ui::ui_element::UIElement;
use std::{
    thread::{self, sleep},
    time::{self, Duration},
};

/// Client that displays the current time when server asks for content
/// Will override the `get_display_content`
/// Different from a standard reactive client that only updates on events from server, but I haven't set that up
struct ReactiveClientInitializer;
impl ClientInitializer for ReactiveClientInitializer {
    fn initialize(
        self: Box<Self>,
        mediator: ClientSideMediator,
        _display_getter: CacheDisplayCommunicator,
    ) {
        // I am starting to worry that this new architecture might get ugly, but hopefully it scales well
        // hmm, I think it's fine if I just move the struct def outside for non-trivial applets
        struct TimeGetter;
        impl DisplayContentGetter for TimeGetter {
            fn is_damaged(&self) -> bool {
                // As I said already, this example is different from a standard reactive client because it always updates
                true
            }

            fn get_display_content(&self) -> singularity_ui::ui_element::UIElement {
                UIElement::from(format!("Time: {:?}", time::SystemTime::now()))
            }
        }

        mediator.set_display_getter(TimeGetter);
    }
}

/// Applet that spawns a seperate thread to update the clock every so often.
struct ActiveClientInitializer;
impl ClientInitializer for ActiveClientInitializer {
    fn initialize(
        self: Box<Self>,
        _mediator: ClientSideMediator,
        display_getter: CacheDisplayCommunicator,
    ) {
        // NOTE: `initialize` should be non-blocking; Sonamu doesn't enforce this but it assumes it
        // I mean, client should never write a blocking function the server calls, but wtv
        thread::spawn(move || {
            loop {
                display_getter.set_display_content(UIElement::from(format!(
                    "Time: {:?}",
                    time::SystemTime::now()
                )));
                // This should update slower than the reactive
                sleep(Duration::from_secs(1));
            }
        });
    }
}

// This is a simplified vesion of what you'd see in the server
pub fn sequential_run(client_initializer: impl ClientInitializer) {
    let cache_display = CacheDisplayCommunicator::default();
    let mediator = SonamuMediator::new(cache_display.clone());
    let (server_side_mediator, client_side_mediator) = mediator.split();

    Box::new(client_initializer).initialize(client_side_mediator, cache_display);

    loop {
        dbg!(server_side_mediator.get_display_content());
        sleep(Duration::from_millis(500));
    }
}

// This is a simplified vesion of what you'd see in the server
pub fn multi_sequential_run(client_initializers: Vec<Box<dyn ClientInitializer>>) {
    let server_side_mediators = client_initializers
        .into_iter()
        .map(|client_initializer| {
            // assume this segment is in the server code
            let cache_display = CacheDisplayCommunicator::default();
            let mediator = SonamuMediator::new(cache_display.clone());
            let (server_side_mediator, client_side_mediator) = mediator.split();

            client_initializer.initialize(client_side_mediator, cache_display);

            server_side_mediator
        })
        .collect::<Vec<_>>();

    loop {
        println!("New frame:");
        for server_side_mediator in &server_side_mediators {
            match server_side_mediator.get_display_content() {
                UIElement::Text(text) => {
                    println!("{}", text[0].0);
                }
                element => {
                    println!("{element:?}");
                }
            }
        }
        sleep(Duration::from_millis(500));
    }
}

fn main() {
    multi_sequential_run(vec![
        Box::new(ReactiveClientInitializer),
        Box::new(ActiveClientInitializer),
    ]);
}
