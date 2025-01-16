use events::{ClipboardEvent, CopiedEvent, DragEvent, DraggedEvent, MyEvent, PastedEvent};
use requests::MyRequest;
use singularity_common::{
    sap::packet::{
        universal_client_socket::UniversalClientSocket,
        universal_server_socket::UniversalServerSocket,
    },
    utils::usock_tools::{self, UnixServerHost},
};

pub mod events {
    use singularity_common::sap::{
        byte_stream::{ToData, TryFromData},
        packet::{IdType, PacketTrait},
    };
    use singularity_macros::PacketUnion;

    #[derive(Debug)]
    pub struct CopiedEvent;
    impl ToData for CopiedEvent {
        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }
    }
    impl TryFromData for CopiedEvent {
        fn try_from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }
    impl PacketTrait for CopiedEvent {
        const PACKET_TYPE_ID: IdType = 98752896453;
    }

    #[derive(Debug)]
    pub struct PastedEvent(pub String);
    impl ToData for PastedEvent {
        fn to_data(&self) -> Vec<u8> {
            // utf8
            self.0.as_bytes().to_vec()
        }
    }
    impl TryFromData for PastedEvent {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            Some(Self(String::from_utf8(data.to_vec()).unwrap()))
        }
    }
    impl PacketTrait for PastedEvent {
        const PACKET_TYPE_ID: IdType = 4325678983412657;
    }

    #[derive(Debug)]
    pub struct DraggedEvent;
    impl ToData for DraggedEvent {
        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }
    }
    impl TryFromData for DraggedEvent {
        fn try_from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }
    impl PacketTrait for DraggedEvent {
        const PACKET_TYPE_ID: IdType = 17859015767526;
    }

    #[derive(Debug, PacketUnion)]
    pub enum ClipboardEvent {
        CopiedEvent(CopiedEvent),
        PastedEvent(PastedEvent),
    }
    #[derive(Debug, PacketUnion)]
    pub enum DragEvent {
        DraggedEvent(DraggedEvent),
    }
    #[derive(Debug, PacketUnion)]
    pub enum MyEvent {
        ClipboardEvent(ClipboardEvent),
        DragEvent(DragEvent),
    }
}

/// NOTE: mostly testing events rn
pub mod requests {
    use singularity_common::sap::{
        byte_stream::{ToData, TryFromData},
        packet::{IdType, PacketTrait},
    };

    #[derive(Debug)]
    pub struct MyRequest;
    impl ToData for MyRequest {
        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }
    }
    impl TryFromData for MyRequest {
        fn try_from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }
    impl PacketTrait for MyRequest {
        const PACKET_TYPE_ID: IdType = 17859015767526;
    }
}

/// Currently testing w/ server and client on the same thread and process
#[test]
fn sap_connection_test() {
    let server = UnixServerHost::bind_new().unwrap();

    let client_side_conn = usock_tools::client_connect_from_env().unwrap();

    // blocks until connection (unless you set to non-blocking)
    let (server_side_conn, _address) = server.listener.accept().unwrap();

    let mut client_socket: UniversalClientSocket<MyEvent> =
        UniversalClientSocket::new(client_side_conn);

    let mut server_socket: UniversalServerSocket<MyEvent, MyRequest> =
        UniversalServerSocket::new(server_side_conn);

    println!("Connected on both ends.");
    println!("Starting basic tests:");

    assert!(dbg!(client_socket.try_read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.try_read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `CopiedEvent`...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::CopiedEvent(
        CopiedEvent,
    )));

    dbg!(client_socket.try_read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.try_read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `PastedEvent` with \"Hello World!\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("Hello World!".to_string()),
    )));

    dbg!(client_socket.try_read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.try_read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `PastedEvent` with \"Good Morning!\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("Good Morning!".to_string()),
    )));

    dbg!(client_socket.try_read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.try_read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Testing multiple sends:");

    println!("Sending `PastedEvent` with \"bonjour\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("bonjour".to_string()),
    )));
    println!("Sending `DraggedEvent`...");
    server_socket.send_event(MyEvent::DragEvent(DragEvent::DraggedEvent(DraggedEvent)));

    dbg!(client_socket.try_read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    dbg!(client_socket.try_read_events());
    dbg!(server_socket.read_requests());
}
