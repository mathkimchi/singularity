//! NOTE: this whole bytes thing is a naming nightmare,
//! especially since everything is just bytes, so the types are all the same

use add_query::{AddQuery, AddResponse};
use events::{CopiedEvent, MyEvent};
use secret_query::{SecretQuery0, SecretQuery1};
use singularity_common::{
    sap::{
        byte_stream::{ToData, TryFromData},
        packet::{PacketTrait, UniversalQuery},
        universal_stream::{
            universal_client_stream::UniversalClientStream,
            universal_server_stream::{as_query_data_responder, UniversalServerStream},
        },
    },
    utils::usock_tools::{self, UnixServerHost},
};
use std::{
    os::unix::net::UnixStream,
    process::Command,
    thread,
    time::{self, Duration},
};
use time_query::{TimeQuery, TimeResponse};

pub mod testing_packets {
    use singularity_macros::PacketUnion;

    use super::*;
    pub mod add_query {
        use singularity_common::sap::byte_stream::ToData;

        use super::*;

        #[derive(Debug)]
        pub struct AddQuery {
            pub lhs: f32,
            pub rhs: f32,
        }
        impl ToData for AddQuery {
            fn to_data(&self) -> Vec<u8> {
                [self.lhs.to_be_bytes(), self.rhs.to_be_bytes()].concat()
            }
        }
        impl TryFromData for AddQuery {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                // TODO: tuple to and from bytes abstraction
                let lhs_bytes: [u8; 4] = data[0..4].try_into().unwrap();
                let rhs_bytes: [u8; 4] = data[4..8].try_into().unwrap();

                Some(Self {
                    lhs: f32::from_be_bytes(lhs_bytes),
                    rhs: f32::from_be_bytes(rhs_bytes),
                })
            }
        }
        impl PacketTrait for AddQuery {
            const PACKET_TYPE_ID: u64 = 287561582341;
        }

        #[derive(Debug)]
        pub struct AddResponse(pub f32);
        impl ToData for AddResponse {
            fn to_data(&self) -> Vec<u8> {
                self.0.to_be_bytes().to_vec()
            }
        }
        impl TryFromData for AddResponse {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                let bytes: [u8; 4] = data[0..4].try_into().unwrap();

                Some(Self(f32::from_be_bytes(bytes)))
            }
        }
        impl PacketTrait for AddResponse {
            const PACKET_TYPE_ID: u64 = 1823795123589;
        }

        impl UniversalQuery for AddQuery {
            type ResponseType = AddResponse;
        }
    }

    pub mod time_query {
        use singularity_common::sap::byte_stream::ToData;

        use super::*;

        #[derive(Debug)]
        pub struct TimeQuery;
        impl ToData for TimeQuery {
            fn to_data(&self) -> Vec<u8> {
                Vec::new()
            }
        }
        impl TryFromData for TimeQuery {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                if !data.is_empty() {
                    return None;
                }

                Some(Self)
            }
        }
        impl PacketTrait for TimeQuery {
            const PACKET_TYPE_ID: u64 = 9857791231234;
        }

        #[derive(Debug)]
        pub struct TimeResponse(pub String);
        impl ToData for TimeResponse {
            fn to_data(&self) -> Vec<u8> {
                self.0.as_bytes().to_vec()
            }
        }
        impl TryFromData for TimeResponse {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                Some(Self(String::from_utf8(data.to_vec()).ok()?))
            }
        }
        impl PacketTrait for TimeResponse {
            const PACKET_TYPE_ID: u64 = 867532867123;
        }

        impl UniversalQuery for TimeQuery {
            type ResponseType = TimeResponse;
        }
    }

    /// This simulates the query that the querier knows about but is unknown to the request
    pub mod secret_query {
        use crate::UniversalQuery;
        use singularity_common::sap::{
            byte_stream::{ToData, TryFromData},
            packet::PacketTrait,
        };

        pub struct SecretQuery0(pub String);
        impl ToData for SecretQuery0 {
            fn to_data(&self) -> Vec<u8> {
                self.0.as_bytes().to_vec()
            }
        }
        impl TryFromData for SecretQuery0 {
            fn try_from_data(_data: &[u8]) -> Option<Self> {
                unimplemented!("This shouldn't be called in the testing code")
            }
        }
        impl PacketTrait for SecretQuery0 {
            const PACKET_TYPE_ID: u64 = 435671234;
        }
        #[derive(Debug)]
        pub struct SecretResponse0;
        impl ToData for SecretResponse0 {
            fn to_data(&self) -> Vec<u8> {
                Vec::new()
            }
        }
        impl TryFromData for SecretResponse0 {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                if !data.is_empty() {
                    return None;
                }

                Some(Self)
            }
        }
        impl PacketTrait for SecretResponse0 {
            const PACKET_TYPE_ID: u64 = 89273456234;
        }

        impl UniversalQuery for SecretQuery0 {
            type ResponseType = SecretResponse0;
        }

        pub struct SecretQuery1;
        impl ToData for SecretQuery1 {
            fn to_data(&self) -> Vec<u8> {
                Vec::new()
            }
        }
        impl TryFromData for SecretQuery1 {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                if !data.is_empty() {
                    return None;
                }

                Some(Self)
            }
        }
        impl PacketTrait for SecretQuery1 {
            const PACKET_TYPE_ID: u64 = 356473546542;
        }
        #[derive(Debug)]
        pub struct SecretResponse1;
        impl ToData for SecretResponse1 {
            fn to_data(&self) -> Vec<u8> {
                Vec::new()
            }
        }
        impl TryFromData for SecretResponse1 {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                if !data.is_empty() {
                    return None;
                }

                Some(Self)
            }
        }
        impl PacketTrait for SecretResponse1 {
            const PACKET_TYPE_ID: u64 = 345346755674257;
        }

        impl UniversalQuery for SecretQuery1 {
            type ResponseType = SecretResponse1;
        }
    }

    #[derive(Debug)]
    pub struct PrintRequest {
        pub msg: String,
    }
    impl ToData for PrintRequest {
        fn to_data(&self) -> Vec<u8> {
            self.msg.as_bytes().to_vec()
        }
    }
    impl TryFromData for PrintRequest {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            Some(Self {
                msg: String::from_utf8(data.to_vec()).ok()?,
            })
        }
    }
    impl PacketTrait for PrintRequest {
        const PACKET_TYPE_ID: u64 = 78543290324;
    }

    #[derive(Debug)]
    pub struct QuitRequest;
    impl ToData for QuitRequest {
        fn to_data(&self) -> Vec<u8> {
            Vec::new()
        }
    }
    impl TryFromData for QuitRequest {
        fn try_from_data(_: &[u8]) -> Option<Self> {
            Some(Self)
        }
    }
    impl PacketTrait for QuitRequest {
        const PACKET_TYPE_ID: u64 = 751529872348;
    }

    #[derive(Debug, PacketUnion)]
    pub enum MyRequest {
        PrintRequest(PrintRequest),
        QuitRequest(QuitRequest),
    }

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
}
use testing_packets::*;

/// NOTE: This should be ran before the client side test
#[test]
fn test_server_side() {
    let server = UnixServerHost::bind_new().unwrap();
    println!("server side: server created");

    // blocks until connection (unless you set to non-blocking)
    let (server_side_conn, _address) = server.listener.accept().unwrap();
    let mut universal_server_stream = UniversalServerStream::new(server_side_conn);

    thread::sleep(Duration::from_nanos(1));

    let mut num_time_queries = 0;
    let mut to_continue = true;

    universal_server_stream.send_event(MyEvent::ClipboardEvent(
        events::ClipboardEvent::CopiedEvent(CopiedEvent),
    ));

    while to_continue {
        // println!("Started a loop of respond all.");
        // std::io::stdout().flush().unwrap();

        let requests: Vec<MyRequest> = universal_server_stream.handle_incoming(&mut vec![
            &mut as_query_data_responder(|AddQuery { lhs, rhs }| Some(AddResponse(lhs + rhs))),
            &mut as_query_data_responder(|TimeQuery| {
                num_time_queries += 1;
                Some(TimeResponse(format!(
                    "The time is: {:?}. This is my {}th time responding to a time query.",
                    time::Instant::now(),
                    num_time_queries
                )))
            }),
        ]);
        // println!("Finished a loop of respond all.");

        println!("Requests: {:?}", requests);

        for request in requests {
            match request {
                MyRequest::PrintRequest(print_request) => {
                    println!("Got print request: {}", print_request.msg)
                }
                MyRequest::QuitRequest(QuitRequest) => to_continue = false,
            }
        }

        // thread::sleep(Duration::from_secs(1));
        thread::sleep(Duration::from_nanos(1));
    }
}

#[test]
fn test_client_side() {
    println!("Hello from client test");

    let client_side_conn = usock_tools::client_connect_from_env().unwrap();
    println!("client side: connected");

    thread::sleep(Duration::from_nanos(1));

    let mut client_stream: UniversalClientStream<UnixStream, MyEvent> =
        UniversalClientStream::new(client_side_conn);

    dbg!(client_stream.try_read_events());

    dbg!(client_stream.query(AddQuery {
        lhs: 1200.,
        rhs: 34.,
    }));
    dbg!(client_stream.query(AddQuery { lhs: 10., rhs: 10. }));
    dbg!(client_stream.query(TimeQuery));
    dbg!(client_stream.query(AddQuery { lhs: 1.0, rhs: 2.0 }));
    dbg!(client_stream.query(AddQuery { lhs: 2.0, rhs: 2.0 }));
    dbg!(client_stream.query(SecretQuery0("Hello".to_string())));
    dbg!(client_stream.query(TimeQuery));
    dbg!(client_stream.query(TimeQuery));
    dbg!(client_stream.query(SecretQuery0("Goodmorning".to_string())));
    dbg!(client_stream.query(SecretQuery1));
    dbg!(
        client_stream.send_request(MyRequest::PrintRequest(PrintRequest {
            msg: "Request to print".to_string(),
        }))
    );
    dbg!(client_stream.query(TimeQuery));
    dbg!(client_stream.query(TimeQuery));
    dbg!(client_stream.query(AddQuery { lhs: 1., rhs: -1. }));

    dbg!(client_stream.try_read_events());

    client_stream.send_request(MyRequest::QuitRequest(QuitRequest));
}

/// Tests both sides on one process, two different thread
#[test]
fn test_same_process() {
    let server_handle = thread::spawn(test_server_side);

    thread::sleep(Duration::from_secs_f32(0.1));
    let client_handle = thread::spawn(test_client_side);

    server_handle.join().unwrap();
    client_handle.join().unwrap();
}

#[test]
fn test_multi_process() {
    let server_handle = Command::new("cargo")
        .args([
            "test",
            "--package",
            "singularity_common",
            "--test",
            "all_packets_sandbox",
            "--",
            "test_server_side",
            "--exact",
            "--show-output",
            "--nocapture",
        ])
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_secs_f32(0.1));

    let client_handle = Command::new("cargo")
        .args([
            "test",
            "--package",
            "singularity_common",
            "--test",
            "all_packets_sandbox",
            "--",
            "test_client_side",
            "--exact",
            "--show-output",
            "--nocapture",
        ])
        .spawn()
        .unwrap();

    dbg!(server_handle.wait_with_output().unwrap());
    dbg!(client_handle.wait_with_output().unwrap());
}
