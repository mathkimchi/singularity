//! NOTE: this whole bytes thing is a naming nightmare,
//! especially since everything is just bytes, so the types are all the same

use singularity_common::sap::{
    byte_stream::{ByteStream, ToData, TryFromData},
    packet::{
        IdType, PacketTrait, PacketType, QueryInstanceId, UniversalQuery, EVENT_PACKET_TYPE,
        QUERY_PACKET_TYPE, REQUEST_PACKET_TYPE, RESPONSE_PACKET_TYPE, UNKNOWN_RESPONSE_TYPE_ID,
        UNKNOWN_RESPONSE_TYPE_ID_BYTES,
    },
};
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
