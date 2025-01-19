//! NOTE: this whole bytes thing is a naming nightmare,
//! especially since everything is just bytes, so the types are all the same

use std::{
    marker::PhantomData,
    os::unix::net::UnixStream,
    thread,
    time::{self, Duration},
};

use add_query::{AddQuery, AddResponse};
use events::MyEvent;
use secret_query::{SecretQuery0, SecretQuery1};
use singularity_common::{
    sap::{
        byte_stream::{ByteStream, ToData, TryFromData},
        packet::{
            IdType, PacketTrait, PacketType, QueryInstanceId, UniversalQuery, EVENT_PACKET_TYPE,
            QUERY_PACKET_TYPE, REQUEST_PACKET_TYPE, RESPONSE_PACKET_TYPE, UNKNOWN_RESPONSE_TYPE_ID,
            UNKNOWN_RESPONSE_TYPE_ID_BYTES,
        },
    },
    utils::usock_tools::{self, UnixServerHost},
};
use time_query::{TimeQuery, TimeResponse};
use uuid::Uuid;

/// To be used by the client.
/// REVIEW: always knowing the event type is slightly more efficient
/// because we wouldn't need to make a slice into a vec,
/// I don't necessarily think that it is better though.
///
/// TODO: I am using `try` and `wait` prefix to specify nonblocking vs blocking,
/// which is based off the [`std::sync::mpsc`], but it is kind of confusing because
/// try is more commonly used as the prefix when the output is [`Option`]/[`Result`]
pub struct UniversalClientStream<Stream: ByteStream, Event: PacketTrait> {
    stream: Stream,

    event_queue: Vec<Event>,
    response_data_queue: Vec<Vec<u8>>,
}
impl<Stream: ByteStream, Event: PacketTrait> UniversalClientStream<Stream, Event> {
    pub fn new(stream: Stream) -> Self {
        Self {
            stream,
            event_queue: Vec::new(),
            response_data_queue: Vec::new(),
        }
    }

    /// Returns: (Packet type, packet data)
    /// For client recieving, packet type ids should be event and response
    fn split_packet_type(data: &[u8]) -> (PacketType, &[u8]) {
        (data[0], &data[1..])
    }

    /// NOTE: annoyingly had to slightly change arguments because of borrow checker
    fn add_data(event_queue: &mut Vec<Event>, response_data_queue: &mut Vec<Vec<u8>>, data: &[u8]) {
        let (packet_type, packet_data) = Self::split_packet_type(data);
        match packet_type {
            EVENT_PACKET_TYPE => {
                if let Some(event) = Event::try_from_data(packet_data) {
                    event_queue.push(event);
                } else {
                    // event not known
                    eprintln!("Warning: Event {:?} could not be parsed", packet_data);
                }
            }
            RESPONSE_PACKET_TYPE => {
                response_data_queue.push(packet_data.to_vec());
            }
            other => {
                eprintln!("Warning: Client stream recieved a packet type {other} that is not an event or response with data {:?}", packet_data)
            }
        }
    }

    /// Non-blocking
    fn try_update_data_queues(&mut self) {
        for incoming_data in self.stream.try_iter_bytes() {
            Self::add_data(
                &mut self.event_queue,
                &mut self.response_data_queue,
                &incoming_data,
            );
        }
    }

    /// Like [`try_update_data_queues`] but blocking and only recieves exactly one.
    fn wait_recieve_packet(&mut self) {
        let incoming_data = self.stream.wait_read_bytes();
        Self::add_data(
            &mut self.event_queue,
            &mut self.response_data_queue,
            &incoming_data,
        );
    }

    /// Non-blocking
    pub fn try_read_events(&mut self) -> Vec<Event> {
        self.try_update_data_queues();

        std::mem::take(&mut self.event_queue)
    }

    /// NOTE: assumes that we already checked the head and know it is a response packet, and sliced off that information
    /// TODO: should be a way to make the indexing more easily maintainable
    fn split_response_bytes(response_bytes: &[u8]) -> (QueryInstanceId, IdType, &[u8]) {
        let query_instance_id = Uuid::from_bytes_le(response_bytes[0..16].try_into().unwrap());
        let response_type_id = IdType::from_be_bytes(
            response_bytes[16..(16 + (IdType::BITS as usize) / 8)]
                .try_into()
                .unwrap(),
        );
        let inner_data = &response_bytes[(16 + (IdType::BITS as usize) / 8)..];

        (query_instance_id, response_type_id, inner_data)
    }

    pub fn query<Q: UniversalQuery>(&mut self, query: Q) -> Option<Q::ResponseType> {
        let query_instance_id = Uuid::new_v4();

        // send query

        let query_bytes = {
            const PACKET_TYPE: [u8; 1] = QUERY_PACKET_TYPE.to_be_bytes();
            let query_instance_id_bytes = query_instance_id.to_bytes_le();
            let query_type_id = Q::PACKET_TYPE_ID.to_be_bytes();
            let query_inner_data = query.to_data();

            [
                &PACKET_TYPE[..],
                &query_instance_id_bytes[..],
                &query_type_id[..],
                &query_inner_data[..],
            ]
            .concat()
        };

        self.stream.write_bytes(&query_bytes);

        // recieve response

        'recieve_loop: loop {
            for incoming_response_data in &self.response_data_queue {
                let (incoming_query_instance_id, response_type_id, inner_data) =
                    Self::split_response_bytes(incoming_response_data);

                if incoming_query_instance_id == query_instance_id {
                    // FIXME: IMPORTANT: should remove from the queue

                    break 'recieve_loop if response_type_id == Q::ResponseType::PACKET_TYPE_ID {
                        Q::ResponseType::try_from_data(inner_data)
                    } else {
                        if response_type_id != UNKNOWN_RESPONSE_TYPE_ID {
                            // instance id matches but type isn't match or the standard unknown,
                            // just debug

                            // (likely two different versions)
                            dbg!(response_type_id);
                        }
                        // the instance id matches but the type doesn't
                        // just assume it is the null response
                        None
                    };
                }
            }

            // didn't break, the incoming packet was for something else

            // update the response data queue
            // REVIEW: I have not thoroughly thought through how this would work multi-threaded,
            // even though a lot of my choices were made to try to support multiple queries sent
            // before gettign responses
            // REVIEW: a way to do this better might be like a `Future` type
            self.wait_recieve_packet();
        }
    }
}

/// To be used by the server.
///
/// Represents connection to one client.
///
/// REVIEW: naming, should I name this: `ClientHandler`?
///
/// Right now, server and client socket are literally just the same with event and request switched.
/// I could abstract to just `UniversalStream<SendPacket, RecvPacket>`,
/// but I am preparing for queries and responses.
/// TODO: I could still abstract though.
///
/// TODO: I have an idea: I don't think the server side actually requires a manual queue storage implementation
pub struct UniversalServerStream<Stream: ByteStream> {
    stream: Stream,
}
impl<Stream: ByteStream> UniversalServerStream<Stream> {
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }

    /// Returns: (Packet type, packet data)
    /// For server recieving, packet type ids should be request and query
    ///
    /// REVIEW: this function is also in the universal_client_stream
    ///
    /// REVIEW: make packet type an enum?
    fn split_packet_type(data: &[u8]) -> (PacketType, &[u8]) {
        (data[0], &data[1..])
    }

    /// Returns (query instance id, query id type, query inner data)
    fn split_query_data(query_packet_data: &[u8]) -> (QueryInstanceId, IdType, &[u8]) {
        let query_instance_id =
            QueryInstanceId::from_bytes_le(query_packet_data[0..16].try_into().unwrap());

        let query_type_id = IdType::from_be_bytes(
            query_packet_data[16..(16 + (IdType::BITS as usize) / 8)]
                .try_into()
                .unwrap(),
        );

        let query_inner_data = &query_packet_data[(16 + (IdType::BITS as usize) / 8)..];

        (query_instance_id, query_type_id, query_inner_data)
    }

    /// responds to all incoming queries and returns a vec of all incoming requests
    pub fn handle_incoming<Request: PacketTrait>(
        &mut self,
        query_responders: &mut Vec<&mut dyn QueryDataResponder>,
    ) -> Vec<Request> {
        let mut requests = Vec::new();

        for incoming_data in self.stream.collect_try_iter_bytes() {
            let (packet_type, packet_data) = Self::split_packet_type(&incoming_data);

            match packet_type {
                REQUEST_PACKET_TYPE => {
                    if let Some(request) = Request::try_from_data(packet_data) {
                        requests.push(request);
                    } else {
                        // request not known
                        eprintln!("Warning: Request {:?} could not be parsed", packet_data);
                    }
                }
                QUERY_PACKET_TYPE => {
                    let (query_instance_id, query_id_type, query_inner_data) =
                        Self::split_query_data(packet_data);

                    let response_packet_data = query_responders.iter_mut().find_map(|query_responder| {
                        if query_id_type != query_responder.__get_query_type_id() {
                            return None;
                        }

                        query_responder
                            .__generate_response_packet_data(query_inner_data, query_instance_id)
                    }).unwrap_or_else(|| {
                        eprintln!("Warning: Query of packet data: `{:?}` could not be parsed or responded to", packet_data);

                        /// TODO: duplicate in `____generate_response_packet_data`
                        const RESPONSE_PACKET_TYPE_BYTES: [u8; 1] = RESPONSE_PACKET_TYPE.to_be_bytes();

                        [
                            RESPONSE_PACKET_TYPE_BYTES.as_slice(),
                            query_instance_id.to_bytes_le().as_slice(),
                            UNKNOWN_RESPONSE_TYPE_ID_BYTES.as_slice(),
                            &[],
                        ]
                        .concat()
                    });

                    self.stream.write_bytes(&response_packet_data);
                }
                _ => {
                    eprintln!("Warning: Server stream recieved a packet type {packet_type} that is not a request or query with data {:?}", packet_data)
                }
            }
        }

        requests
    }

    pub fn send_event<Event: PacketTrait>(&mut self, event: Event) {
        self.stream.write_bytes(&event.to_data());
    }
}

pub trait QueryResponder {
    type Query: UniversalQuery;

    fn respond(
        &mut self,
        query: Self::Query,
        query_instance_id: QueryInstanceId,
    ) -> Option<<Self::Query as UniversalQuery>::ResponseType>;
}
/// NOTE: don't override this
/// TODO: make this somehow public but not overridable
pub trait QueryDataResponder {
    fn __get_query_type_id(&self) -> IdType;
    fn __generate_response_packet_data(
        &mut self,
        query_data: &[u8],
        query_instance_id: QueryInstanceId,
    ) -> Option<Vec<u8>>;
}
impl<R: QueryResponder> QueryDataResponder for R {
    fn __get_query_type_id(&self) -> IdType {
        R::Query::PACKET_TYPE_ID
    }

    fn __generate_response_packet_data(
        &mut self,
        query_data: &[u8],
        query_instance_id: QueryInstanceId,
    ) -> Option<Vec<u8>> {
        let response_object: <R::Query as UniversalQuery>::ResponseType =
            self.respond(R::Query::try_from_data(query_data)?, query_instance_id)?;

        const RESPONSE_PACKET_TYPE_BYTES: [u8; 1] = RESPONSE_PACKET_TYPE.to_be_bytes();

        Some(
            [
                RESPONSE_PACKET_TYPE_BYTES.as_slice(),
                query_instance_id.to_bytes_le().as_slice(),
                <R::Query as UniversalQuery>::ResponseType::PACKET_TYPE_ID
                    .to_be_bytes()
                    .as_slice(),
                response_object.to_data().as_slice(),
            ]
            .concat(),
        )
    }
}

pub fn as_query_data_responder<Q: UniversalQuery, F: FnMut(Q) -> Option<Q::ResponseType>>(
    f: F,
) -> impl QueryDataResponder {
    struct FnWrapper<Q: UniversalQuery, F: FnMut(Q) -> Option<Q::ResponseType>>(F, PhantomData<Q>);

    impl<Q: UniversalQuery, F: FnMut(Q) -> Option<Q::ResponseType>> QueryResponder for FnWrapper<Q, F> {
        type Query = Q;

        fn respond(
            &mut self,
            query: Self::Query,
            _query_instance_id: QueryInstanceId,
        ) -> Option<<Self::Query as UniversalQuery>::ResponseType> {
            self.0(query)
        }
    }

    FnWrapper(f, PhantomData)
}

pub mod testing_packets {
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
        msg: String,
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
///
/// FIXME: this and client side test will ruin a total test, if I ever did one. Make this like a main runner, not a test
#[test]
fn test_server_side() {
    let server = UnixServerHost::bind_new().unwrap();
    println!("server side: server created");

    // blocks until connection (unless you set to non-blocking)
    let (server_side_conn, _address) = server.listener.accept().unwrap();
    let mut universal_server_stream = UniversalServerStream {
        stream: server_side_conn,
    };

    thread::sleep(Duration::from_nanos(1));

    let mut num_time_queries = 0;
    let mut to_continue = true;

    while to_continue {
        // println!("Started a loop of respond all.");
        // std::io::stdout().flush().unwrap();

        let requests: Vec<PrintRequest> = universal_server_stream.handle_incoming(&mut vec![
            &mut as_query_data_responder(|AddQuery { lhs, rhs }| {
                if lhs == 666.0 && rhs == 666.0 {
                    to_continue = false;
                }
                Some(AddResponse(lhs + rhs))
            }),
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

    let mut querier: UniversalClientStream<UnixStream, MyEvent> =
        UniversalClientStream::new(client_side_conn);
    dbg!(querier.query(AddQuery {
        lhs: 1200.,
        rhs: 34.,
    }));
    dbg!(querier.query(AddQuery { lhs: 10., rhs: 10. }));
    dbg!(querier.query(TimeQuery));
    dbg!(querier.query(AddQuery { lhs: 1.0, rhs: 2.0 }));
    dbg!(querier.query(AddQuery { lhs: 2.0, rhs: 2.0 }));
    dbg!(querier.query(SecretQuery0("Hello".to_string())));
    dbg!(querier.query(TimeQuery));
    dbg!(querier.query(TimeQuery));
    dbg!(querier.query(SecretQuery0("Goodmorning".to_string())));
    dbg!(querier.query(SecretQuery1));
    dbg!(querier.query(TimeQuery));
    dbg!(querier.query(TimeQuery));
    dbg!(querier.query(AddQuery { lhs: 1., rhs: -1. }));
    // this is the temporary, jank quitting
    dbg!(querier.query(AddQuery {
        lhs: 666.,
        rhs: 666.
    }));
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
