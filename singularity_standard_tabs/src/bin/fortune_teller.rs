use singularity_sap::{
    byte_stream::{ByteReaderWrapper, CombinedByteStream},
    standard_packets::display_packets::{DisplayEvent, RequestChangeName, RequestUpdateWindow},
    universal_stream::universal_client_stream::UniversalClientStream,
};
use singularity_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
};
use std::{
    io::Stdout,
    thread::sleep,
    time::{self, Duration, UNIX_EPOCH},
};

pub const FORTUNES: [&str; 4] = [
    "An idiot admires complexity, a genius admires simplicity.",
    "You'll never know if you don't go\nYou'll never shine if you don't glow",
    "To play a wrong note is insignificant; to play without passion is inexcusable.",
    "A monad is just a monoid in the category of endofunctors, what's the problem?",
];

fn main() {
    let mut client_stream: UniversalClientStream<
        CombinedByteStream<ByteReaderWrapper, Stdout>,
        DisplayEvent,
    > = UniversalClientStream::new(CombinedByteStream::take_from_stdio());

    let now = time::SystemTime::now();
    let seed = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let pseudo_rand = seed.count_ones() as usize;
    let fortune_str = FORTUNES[pseudo_rand % (FORTUNES.len())];

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
    client_stream.send_request(RequestChangeName {
        new_name: "Fortuna".to_string(),
    });

    loop {
        sleep(Duration::from_secs(1));
    }
}
