use singularity_common::utils::usock_tools::{client_connect_from_env, UnixServerHost};
use singularity_macros::{Packet, PacketUnion, Request};
use singularity_sap::{
    datable::{ToData, TryFromData},
    packet::{PacketId, PacketTrait, RequestPacketTrait},
    standard_packets::display_packets::{DisplayEvent, RequestChangeName},
    universal_stream::{
        universal_client_stream::UniversalClientStream,
        universal_server_stream::UniversalServerStream,
    },
};
use std::{
    thread::{self, spawn},
    time::Duration,
};

#[derive(PacketUnion, Packet, Request)]
enum MyRequest {
    RequestChangeName(RequestChangeName),
}

#[test]
fn test_request() {
    let unix_host = UnixServerHost::bind_new().unwrap();
    let client_handle = spawn(|| {
        let mut client_stream: UniversalClientStream<std::os::unix::net::UnixStream, DisplayEvent> =
            UniversalClientStream::new(client_connect_from_env().unwrap());
        client_stream.send_request(RequestChangeName {
            new_name: "Hello?".to_string(),
        });
    });
    let mut server_stream = UniversalServerStream::new(unix_host.listener.accept().unwrap().0);

    loop {
        let requests: Vec<MyRequest> = server_stream.handle_incoming(&mut vec![]);
        if requests.is_empty() {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        println!(
            "Change name request to: {}",
            match requests[0] {
                MyRequest::RequestChangeName(RequestChangeName { ref new_name }) => new_name,
            }
        );
        break;
    }

    client_handle.join().unwrap();
}
