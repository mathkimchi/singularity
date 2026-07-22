use singularity_common::sap::mediator::{
    CacheDisplay, ClientInitializer, ClientSideMediator, DisplayContentGetter, SonamuMediator,
};
use singularity_ui::ui_element::{CharGrid, UIElement};
use std::{
    thread::sleep,
    time::{self, Duration},
};

/// Client that displays the current time when server asks for content
/// Will override the `get_display_content`
/// Different from a standard reactive client that only updates on events from server, but I haven't set that up
struct ReactiveClientInitializer;
impl ClientInitializer for ReactiveClientInitializer {
    fn initialize(self, mediator: ClientSideMediator, _display_getter: CacheDisplay) {
        // I am starting to worry that this new architecture might get ugly, but hopefully it scales well
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

// /// Client that increases counter and updates display on user input
// struct ActiveClientInitializer {}
// impl ClientInitializer for ActiveClientInitializer {
//     fn initialize(self, mediator: SonamuMediator, display_getter: CacheDisplay) {
//         todo!()
//     }
// }

fn sequential_run(client_initializer: impl ClientInitializer) {
    // assume this segment is in the server code
    let cache_display = CacheDisplay::default();
    let mediator = SonamuMediator::new(cache_display.clone());
    let (server_side_mediator, client_side_mediator) = mediator.split();

    client_initializer.initialize(client_side_mediator, cache_display);

    loop {
        dbg!(server_side_mediator.get_display_content());
        sleep(Duration::from_millis(500));
    }
}

fn main() {
    sequential_run(ReactiveClientInitializer);
}
