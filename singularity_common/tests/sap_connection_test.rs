use events::{ClipboardEvent, CopiedEvent, DragEvent, DraggedEvent, MyEvent, PastedEvent};
use requests::MyRequest;
use singularity_common::sap::packet::{
    universal_client_socket::UniversalClientSocket, universal_server_socket::UniversalServerSocket,
};
use unix_tools::{ServerHandle, ServerHost};

pub mod events {
    use singularity_common::sap::packet::{IdType, PacketTrait};
    use singularity_macros::PacketUnion;

    #[derive(Debug)]
    pub struct CopiedEvent;
    impl PacketTrait for CopiedEvent {
        const PACKET_TYPE_ID: IdType = 98752896453;

        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }

        fn from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }

    #[derive(Debug)]
    pub struct PastedEvent(pub String);
    impl PacketTrait for PastedEvent {
        const PACKET_TYPE_ID: IdType = 4325678983412657;

        fn to_data(&self) -> Vec<u8> {
            // utf8
            self.0.as_bytes().to_vec()
        }

        fn from_data(data: &[u8]) -> Option<Self> {
            Some(Self(String::from_utf8(data.to_vec()).unwrap()))
        }
    }

    #[derive(Debug)]
    pub struct DraggedEvent;
    impl PacketTrait for DraggedEvent {
        const PACKET_TYPE_ID: IdType = 17859015767526;

        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }

        fn from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
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
    use singularity_common::sap::packet::{IdType, PacketTrait};

    #[derive(Debug)]
    pub struct MyRequest;
    impl PacketTrait for MyRequest {
        const PACKET_TYPE_ID: IdType = 17859015767526;

        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }

        fn from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }
}

/// from usock_demo::connection
/// TODO: put this in singularity_common
pub mod unix_tools {
    use std::os::unix::net::{UnixListener, UnixStream};

    const PATH_PREFIX_ENV_KEY: &str = "XDG_RUNTIME_DIR";
    const PATH_SUFFIX_ENV_KEY: &str = "SINGULARITY_SERVER";

    /// Server side
    pub struct ServerHost {
        path: String,
        pub listener: UnixListener,
    }
    impl ServerHost {
        pub fn bind_new() -> Option<Self> {
            let path_prefix = std::env::var(PATH_PREFIX_ENV_KEY).ok()?;

            // TODO
            let path_suffix = "singularity-0";

            // FIXME, I think this only applies to children processes
            std::env::set_var(PATH_SUFFIX_ENV_KEY, path_suffix);

            let path = format!("{}/{}", path_prefix, path_suffix);
            Some(Self {
                listener: UnixListener::bind(&path).ok()?,
                path,
            })
        }
    }
    impl Drop for ServerHost {
        fn drop(&mut self) {
            // smh, rust should have some temp_set_env_var function which returns an empty object so it auto removes on drop
            // std::env::remove_var(PATH_SUFFIX_ENV_KEY);
            // ^ actually, processes might make this unnecessary

            // unix listener doesn't remove the file on drop
            if let Err(e) = std::fs::remove_file(&self.path) {
                dbg!(e);
            }
        }
    }

    /// Client-side
    pub struct ServerHandle {
        pub stream: UnixStream,
    }
    impl ServerHandle {
        /// Connect to unix socket at `$XDG_RUNTIME_DIR/$SINGULARITY_SERVER`
        pub fn connect_from_env() -> Option<Self> {
            let socket_path = format!(
                "{}/{}",
                std::env::var(PATH_PREFIX_ENV_KEY).ok()?,
                std::env::var(PATH_SUFFIX_ENV_KEY).unwrap_or("singularity-0".to_string())
            );

            Some(Self {
                stream: UnixStream::connect(socket_path).ok()?,
            })
        }
    }
}

/// Currently testing w/ server and client on the same thread and process
#[test]
fn sap_connection_test() {
    let server = ServerHost::bind_new().unwrap();

    let client_side_conn = ServerHandle::connect_from_env().unwrap().stream;

    // blocks until connection (unless you set to non-blocking)
    let (server_side_conn, _address) = server.listener.accept().unwrap();

    let mut client_socket: UniversalClientSocket<MyEvent, MyRequest> =
        UniversalClientSocket::new(client_side_conn);

    let mut server_socket: UniversalServerSocket<MyEvent, MyRequest> =
        UniversalServerSocket::new(server_side_conn);

    println!("Connected on both ends.");
    println!("Starting basic tests:");

    assert!(dbg!(client_socket.read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `CopiedEvent`...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::CopiedEvent(
        CopiedEvent,
    )));

    dbg!(client_socket.read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `PastedEvent` with \"Hello World!\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("Hello World!".to_string()),
    )));

    dbg!(client_socket.read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Sending `PastedEvent` with \"Good Morning!\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("Good Morning!".to_string()),
    )));

    dbg!(client_socket.read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    assert!(dbg!(client_socket.read_events()).is_empty());
    assert!(dbg!(server_socket.read_requests()).is_empty());

    println!("Testing multiple sends:");

    println!("Sending `PastedEvent` with \"bonjour\"...");
    server_socket.send_event(MyEvent::ClipboardEvent(ClipboardEvent::PastedEvent(
        PastedEvent("bonjour".to_string()),
    )));
    println!("Sending `DraggedEvent`...");
    server_socket.send_event(MyEvent::DragEvent(DragEvent::DraggedEvent(DraggedEvent)));

    dbg!(client_socket.read_events());
    dbg!(server_socket.read_requests());
    // test reading twice just to be safe
    dbg!(client_socket.read_events());
    dbg!(server_socket.read_requests());
}
