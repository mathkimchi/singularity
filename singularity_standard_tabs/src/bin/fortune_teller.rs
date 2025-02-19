use singularity_common::utils::usock_tools::client_connect_from_env;
use singularity_sap::{
    standard_packets::display_packets::{DisplayEvent, RequestUpdateWindow},
    universal_stream::universal_client_stream::UniversalClientStream,
};
use singularity_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
};
use std::{
    os::unix::net::UnixStream,
    time::{self, UNIX_EPOCH},
};

pub const FORTUNES: [&str; 4] = [
    "An idiot admires complexity, a genius admires simplicity.",
    "You'll never know if you don't go\nYou'll never shine if you don't glow",
    "To play a wrong note is insignificant; to play without passion is inexcusable.",
    "A monad is just a monoid in the category of endofunctors, what's the problem?",
];

fn main() {
    let mut client_stream: UniversalClientStream<UnixStream, DisplayEvent> =
        UniversalClientStream::new(client_connect_from_env().unwrap());

    let now = time::SystemTime::now();
    let seed = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let pseudo_rand = seed.count_ones() as usize;
    let fortune_str = FORTUNES[pseudo_rand % (FORTUNES.len())];

    println!("Fortune: {fortune_str}");

    let fortune_ui = UIElement::CharGrid(CharGrid::new_monostyled(
        fortune_str.into(),
        Color::LIGHT_GREEN,
        Color::TRANSPARENT,
    ));

    client_stream.send_request(RequestUpdateWindow {
        contents: fortune_ui,
    });
}
