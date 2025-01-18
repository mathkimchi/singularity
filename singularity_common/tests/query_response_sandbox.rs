//! This is the little sandbox file where I will try out query response
//! code and sift out ideas.
//!
//! Right now, I am just going to assume its only queries and responses
//! being sent.

use secret_query::{SecretQuery0, SecretQuery1};
use singularity_common::{
    sap::{
        byte_stream::{ByteReader, ByteWriter, TryFromData},
        packet::{
            IdType, PacketTrait, PacketType, QueryInstanceId, UniversalQuery, QUERY_PACKET_TYPE,
            RESPONSE_PACKET_TYPE, UNKNOWN_RESPONSE_TYPE_ID, UNKNOWN_RESPONSE_TYPE_ID_BYTES,
        },
    },
    utils::usock_tools::{self, UnixServerHost},
};
use std::{
    io::Write,
    os::unix::net::UnixStream,
    thread,
    time::{self, Duration},
};
use uuid::Uuid;

pub struct UniversalQuerier {
    connection: UnixStream,

    queue: Vec<Vec<u8>>,
}
impl UniversalQuerier {
    pub fn new(connection: UnixStream) -> Self {
        Self {
            connection,
            queue: Vec::new(),
        }
    }

    /// Returns [`None`] if the byte is not a response.
    /// Still returns [`Some`] if it is a unknown response or a response that is for a different query
    fn split_response_bytes(response_bytes: &[u8]) -> Option<(QueryInstanceId, IdType, Vec<u8>)> {
        let packet_type = response_bytes[0];
        if packet_type != RESPONSE_PACKET_TYPE {
            return None;
        }

        let query_instance_id = Uuid::from_bytes_le(response_bytes[1..(1 + 16)].try_into().ok()?);
        let response_type_id = IdType::from_be_bytes(
            response_bytes[(1 + 16)..(1 + 16 + (IdType::BITS as usize) / 8)]
                .try_into()
                .ok()?,
        );
        let inner_data = response_bytes[(1 + 16 + (IdType::BITS as usize) / 8)..].to_vec();

        Some((query_instance_id, response_type_id, inner_data))
    }

    pub fn query<Q: UniversalQuery>(&mut self, query: Q) -> Option<Q::ResponseType> {
        let query_instance_id = Uuid::new_v4();

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

        self.connection.write_bytes(&query_bytes);

        loop {
            // need to wait for response packet anyways, so use `wait_read_bytes`
            let incoming_packet_bytes = self.connection.wait_read_bytes();

            if let Some((incoming_query_instance_id, response_type_id, inner_data)) =
                Self::split_response_bytes(&incoming_packet_bytes)
            {
                if incoming_query_instance_id == query_instance_id {
                    break if response_type_id == Q::ResponseType::PACKET_TYPE_ID {
                        Q::ResponseType::try_from_data(&inner_data)
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

            // didn't break, the incoming packet is for something else
            self.queue.push(incoming_packet_bytes);
        }
    }
}

mod add_query {
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
use add_query::*;

mod time_query {
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
use time_query::*;

/// This simulates the query that the querier knows about but is unknown to the request
mod secret_query {
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

/**********************************
 * below should be macro generated
 */

#[derive(Debug)]
pub enum MySupportedQuery {
    AddQuery(AddQuery),
    TimeQuery(TimeQuery),
    /// Special query type
    __UnsupportedQuery,
}

pub struct MyQueryResponder {
    connection: UnixStream,
}
impl MyQueryResponder {
    /// Returns [`None`] if the packet was not a query
    /// TODO: in the actual thing, make this return an enum of Query or Request
    fn bytes_to_query(query_bytes: &[u8]) -> Option<(QueryInstanceId, MySupportedQuery)> {
        let packet_type = PacketType::from_be_bytes(query_bytes[0..1].try_into().unwrap());
        if packet_type != QUERY_PACKET_TYPE {
            return None;
        }

        let query_instance_id =
            QueryInstanceId::from_bytes_le(query_bytes[1..(1 + 16)].try_into().ok()?);

        let query_type_id = IdType::from_be_bytes(
            query_bytes[(1 + 16)..(1 + 16 + (IdType::BITS as usize) / 8)]
                .try_into()
                .ok()?,
        );

        let query_enum = match query_type_id {
            AddQuery::PACKET_TYPE_ID => {
                AddQuery::try_from_data(&query_bytes[(1 + 16 + (IdType::BITS as usize) / 8)..])
                    .map_or(MySupportedQuery::__UnsupportedQuery, |q| {
                        MySupportedQuery::AddQuery(q)
                    })
            }
            TimeQuery::PACKET_TYPE_ID => {
                TimeQuery::try_from_data(&query_bytes[(1 + 16 + (IdType::BITS as usize) / 8)..])
                    .map_or(MySupportedQuery::__UnsupportedQuery, |q| {
                        MySupportedQuery::TimeQuery(q)
                    })
            }
            _ => MySupportedQuery::__UnsupportedQuery,
        };

        Some((query_instance_id, query_enum))
    }

    /// Doesn't wait
    ///
    /// TODO: right now, just assumes query. later, should handle events as well
    fn try_get_queries(&mut self) -> Vec<(QueryInstanceId, MySupportedQuery)> {
        let mut queries = Vec::new();

        while let Some(query_bytes) = self.connection.try_read_bytes() {
            if let Some(q) = Self::bytes_to_query(&query_bytes) {
                queries.push(q);
            }
        }

        queries
    }

    fn send_response_bytes(
        &mut self,
        query_instance_id: QueryInstanceId,
        response_type_id: [u8; 8],
        response_inner_data: Vec<u8>,
    ) {
        let request_bytes = {
            const PACKET_TYPE: [u8; 1] = RESPONSE_PACKET_TYPE.to_be_bytes();
            let query_instance_id_bytes = query_instance_id.to_bytes_le();

            [
                &PACKET_TYPE[..],
                &query_instance_id_bytes[..],
                &response_type_id[..],
                &response_inner_data[..],
            ]
            .concat()
        };

        self.connection.write_bytes(&request_bytes);
    }

    fn send_response_packet<R: PacketTrait>(
        &mut self,
        query_instance_id: QueryInstanceId,
        response: R,
    ) {
        self.send_response_bytes(
            query_instance_id,
            R::PACKET_TYPE_ID.to_be_bytes(),
            response.to_data(),
        );
    }

    /// TODO: make the responders output Option?
    pub fn respond_all<
        AddResponder: FnMut(AddQuery) -> <AddQuery as UniversalQuery>::ResponseType,
        TimeResponder: FnMut(TimeQuery) -> <TimeQuery as UniversalQuery>::ResponseType,
    >(
        &mut self,
        mut add_responder: AddResponder,
        mut time_responder: TimeResponder,
    ) {
        for (query_instance_id, query_enum) in self.try_get_queries() {
            match query_enum {
                MySupportedQuery::AddQuery(add_query) => {
                    self.send_response_packet(query_instance_id, add_responder(add_query));
                }
                MySupportedQuery::TimeQuery(time_query) => {
                    self.send_response_packet(query_instance_id, time_responder(time_query));
                }
                MySupportedQuery::__UnsupportedQuery => self.send_response_bytes(
                    query_instance_id,
                    UNKNOWN_RESPONSE_TYPE_ID_BYTES,
                    Vec::new(),
                ),
            }
        }
    }
}

/*
 * above should be macro generated
 **********************************
 */

#[test]
fn test() {
    println!("Hi!");
    std::io::stdout().flush().unwrap();

    let server = UnixServerHost::bind_new().unwrap();
    println!("server side: server created");

    let server_thread = thread::spawn(move || {
        // blocks until connection (unless you set to non-blocking)
        let (server_side_conn, _address) = server.listener.accept().unwrap();
        let mut responder = MyQueryResponder {
            connection: server_side_conn,
        };

        thread::sleep(Duration::from_nanos(1));

        let mut num_time_queries = 0;
        let mut to_continue = true;

        while to_continue {
            // println!("Started a loop of respond all.");
            // std::io::stdout().flush().unwrap();

            responder.respond_all(
                |AddQuery { lhs, rhs }| {
                    if lhs == 666.0 && rhs == 666.0 {
                        to_continue = false;
                    }
                    AddResponse(lhs + rhs)
                },
                |TimeQuery| {
                    num_time_queries += 1;
                    TimeResponse(format!(
                        "The time is: {:?}. This is my {}th time responding to a time query.",
                        time::Instant::now(),
                        num_time_queries
                    ))
                },
            );
            // println!("Finished a loop of respond all.");

            // thread::sleep(Duration::from_secs(1));
            thread::sleep(Duration::from_nanos(1));
        }
    });

    let client_thread = thread::spawn(|| {
        println!("Hello from client thread");

        let client_side_conn = usock_tools::client_connect_from_env().unwrap();
        println!("client side: connected");

        thread::sleep(Duration::from_nanos(1));

        let mut querier = UniversalQuerier::new(client_side_conn);
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
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
