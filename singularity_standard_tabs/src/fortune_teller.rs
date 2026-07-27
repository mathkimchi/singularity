use singularity_macros::EventPacketUnion;
use singularity_sap::{
    byte_stream::{ByteReaderWrapper, CombinedByteStream},
    packet::{EventPacketUnion, PacketTypeId, PacketUnion},
    standard_packets::display_packets::{DisplayEvent, RequestChangeName, RequestUpdateWindow},
    universal_stream::universal_client_stream::UniversalClientStream,
};
use sonamu_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};
use std::{
    io::Stdout,
    thread::sleep,
    time::{self, Duration, UNIX_EPOCH},
};

pub const FORTUNES: [&str; 7] = [
    "An idiot admires complexity, a genius admires simplicity.",
    "You'll never know if you don't go\nYou'll never shine if you don't glow",
    "To play a wrong note is insignificant; to play without passion is inexcusable.",
    "A monad is just a monoid in the category of endofunctors, what's the problem?",
    "And so it must be, for so it is written
On the doorway to paradise
That those who falter and those who fall
Must pay the price!",
    "Act only according to that maxim by which you can at the same time will that it should become a universal law.",
    "When the odds are saying you'll never win, that's when the grin should start!",
];

#[derive(EventPacketUnion)]
enum MyEvent {
    #[sub_union]
    DisplayEvent(DisplayEvent),
}

fn main() {
    let mut client_stream: UniversalClientStream<
        CombinedByteStream<ByteReaderWrapper, Stdout>,
        MyEvent,
    > = UniversalClientStream::new(CombinedByteStream::take_from_stdio());

    client_stream.send_request(RequestChangeName::new(&"Fortuna"));

    // ik, semantic types are bad, whatever
    let mut past_index = usize::MAX;

    loop {
        let now = time::SystemTime::now();
        let seed = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let pseudo_rand = seed.count_ones() as usize;
        let mut fortune_index = pseudo_rand % (FORTUNES.len());
        if fortune_index == past_index {
            fortune_index += 1;
            fortune_index %= FORTUNES.len();
        }
        past_index = fortune_index;
        let fortune_str = FORTUNES[fortune_index];

        // // Jank way of temporarily debugging
        // eprintln!("Fortune: {fortune_str}");

        let fortune_ui = UIElement::CharGrid(CharGrid::new_monostyled(
            fortune_str.into(),
            Color::LIGHT_GREEN,
            Color::TRANSPARENT,
        ));

        client_stream.send_request(RequestUpdateWindow {
            contents: fortune_ui,
        });

        'recv_loop: loop {
            let events = client_stream.try_read_events();
            for MyEvent::DisplayEvent(event) in events {
                match event {
                    DisplayEvent::UIEvent(UIEvent::KeyPress(key, KeyModifiers::NONE))
                        if key.to_char() == Some(' ') =>
                    {
                        // generate new fortune
                        break 'recv_loop;
                    }
                    _ => {}
                }
            }
            sleep(Duration::from_millis(1));
        }
    }
}
