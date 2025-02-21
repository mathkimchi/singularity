use singularity_common::utils::usock_tools::client_connect_from_env;
use singularity_sap::{
    standard_packets::display_packets::DisplayEvent,
    universal_stream::universal_client_stream::UniversalClientStream,
};
use std::{os::unix::net::UnixStream, thread, time::Duration};

fn main() {
    println!("Hi from print_test_tab");

    let mut client_stream: UniversalClientStream<UnixStream, DisplayEvent> =
        UniversalClientStream::new(client_connect_from_env().unwrap());
    println!("Client side: connected");
    thread::sleep(Duration::from_secs_f32(0.5));
    dbg!(client_stream.try_read_events());
    println!("Client side: finished");
}
