use crate::packet::{PacketCategory, PacketTypeId};

/// Returns: (Packet category, packet type id, packet inner data)
///
/// For client recieving, packet type ids should be event and response
/// For server recieving, packet type ids should be request and query
///
/// REVIEW: make packet type an enum?
#[deprecated(note = "use packet_to_typed_data and packet_try_from_bytes")]
fn split_packet_category(data: &[u8]) -> (PacketCategory, PacketTypeId, &[u8]) {
    (
        data[0], // like data[0..1]
        PacketTypeId::from_be_bytes(
            data[1..(1 + (PacketTypeId::BITS as usize) / 8)]
                .try_into()
                .unwrap(),
        ),
        &data[(1 + (PacketTypeId::BITS as usize) / 8)..],
    )
}

/// TODO: use cfg attributes for client and server, or just make new crates
pub mod universal_client_stream {
    use crate::{
        byte_stream::ByteStream,
        datable::TryFromData,
        packet::{
            EventPacketUnion, PacketTrait as _, PacketTypeId, QueryInstanceId, RequestPacketTrait,
            UniversalQueryTrait, EVENT_PACKET_CATEGORY, QUERY_PACKET_CATEGORY,
            REQUEST_PACKET_CATEGORY, RESPONSE_PACKET_CATEGORY, UNKNOWN_RESPONSE_TYPE_ID,
        },
    };
    use uuid::Uuid;

    use super::split_packet_category;

    /// To be used by the client.
    /// REVIEW: always knowing the event type is slightly more efficient
    /// because we wouldn't need to make a slice into a vec,
    /// I don't necessarily think that it is better though.
    ///
    /// TODO: I am using `try` and `wait` prefix to specify nonblocking vs blocking,
    /// which is based off the [`std::sync::mpsc`], but it is kind of confusing because
    /// try is more commonly used as the prefix when the output is [`Option`]/[`Result`]
    pub struct UniversalClientStream<Stream: ByteStream, Event: EventPacketUnion> {
        stream: Stream,

        /// REVIEW: slightly less performant, but philosophically better way might be to store `Vec<(PacketType, Vec<u8>)>` bc then, UClientStream wouldn't need Event generic.
        event_queue: Vec<Event>,
        response_data_queue: Vec<(PacketTypeId, Vec<u8>)>,
    }
    impl<Stream: ByteStream, Event: EventPacketUnion> UniversalClientStream<Stream, Event> {
        pub fn new(stream: Stream) -> Self {
            Self {
                stream,
                event_queue: Vec::new(),
                response_data_queue: Vec::new(),
            }
        }

        /// NOTE: annoyingly had to slightly change arguments because of borrow checker
        fn add_data(
            event_queue: &mut Vec<Event>,
            response_data_queue: &mut Vec<(PacketTypeId, Vec<u8>)>,
            data: &[u8],
        ) {
            let (packet_category, packet_type_id, packet_inner_data) = split_packet_category(data);
            match packet_category {
                EVENT_PACKET_CATEGORY => {
                    if let Some(event) =
                        Event::packet_try_from_data(packet_type_id, packet_inner_data)
                    {
                        event_queue.push(event);
                    } else {
                        // event not known
                        eprintln!("Warning: Event with type id {packet_type_id} and inner data {:?} could not be parsed", packet_inner_data);
                    }
                }
                RESPONSE_PACKET_CATEGORY => {
                    response_data_queue.push((packet_type_id, packet_inner_data.to_vec()));
                }
                other => {
                    eprintln!("Warning: Client stream recieved a packet category {other} and type id {packet_type_id} that is not an event or response with data {:?}", packet_inner_data)
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

        /// Blocks until there is an event.
        /// If there are already events, then returns instantly.
        /// If there are no events, then waits.
        pub fn wait_read_events(&mut self) -> Vec<Event> {
            let events = self.try_read_events();
            if !events.is_empty() {
                return events;
            }

            self.wait_recieve_packet();
            self.try_read_events()
        }

        /// Non-blocking
        pub fn try_read_events(&mut self) -> Vec<Event> {
            self.try_update_data_queues();

            std::mem::take(&mut self.event_queue)
        }

        /// NOTE: assumes that we already checked the head and know it is a response packet, and sliced off that information
        /// TODO: should be a way to make the indexing more easily maintainable
        fn split_response_bytes(response_bytes: &[u8]) -> (QueryInstanceId, &[u8]) {
            let query_instance_id = Uuid::from_bytes_le(response_bytes[0..16].try_into().unwrap());
            let inner_data = &response_bytes[16..];

            (query_instance_id, inner_data)
        }

        pub fn send_request<R: RequestPacketTrait>(&mut self, request: R) {
            let request_bytes = {
                const REQUEST_PACKET_CATEGORY_DATA: [u8; 1] = REQUEST_PACKET_CATEGORY.to_be_bytes();
                let request_data = &request.to_data();

                [
                    &REQUEST_PACKET_CATEGORY_DATA[..],
                    &R::PACKET_TYPE_ID.to_be_bytes(),
                    request_data,
                ]
                .concat()
            };

            self.stream.write_bytes(&request_bytes);
        }

        pub fn query<Q: UniversalQueryTrait>(&mut self, query: Q) -> Option<Q::ResponseType> {
            let query_instance_id = Uuid::new_v4();

            // send query

            let query_bytes = {
                const PACKET_TYPE: [u8; 1] = QUERY_PACKET_CATEGORY.to_be_bytes();
                let query_type_id = Q::PACKET_TYPE_ID.to_be_bytes();
                let query_instance_id_bytes = query_instance_id.to_bytes_le();
                let query_inner_data = query.to_data();

                [
                    &PACKET_TYPE[..],
                    &query_type_id[..],
                    &query_instance_id_bytes[..],
                    &query_inner_data[..],
                ]
                .concat()
            };

            self.stream.write_bytes(&query_bytes);

            // recieve response

            'recieve_loop: loop {
                for (response_type_id, incoming_response_data) in &self.response_data_queue {
                    let (incoming_query_instance_id, inner_data) =
                        Self::split_response_bytes(incoming_response_data);

                    if incoming_query_instance_id == query_instance_id {
                        // FIXME: IMPORTANT: should remove from the queue

                        break 'recieve_loop if *response_type_id == Q::ResponseType::PACKET_TYPE_ID
                        {
                            Q::ResponseType::try_from_data(inner_data)
                        } else {
                            if *response_type_id != UNKNOWN_RESPONSE_TYPE_ID {
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
}

pub mod universal_server_stream {
    use crate::{
        byte_stream::ByteStream,
        datable::{ToData, TryFromData},
        packet::{
            EventPacketTrait, EventPacketUnion, PacketTrait as _, PacketTypeId, QueryInstanceId,
            RequestPacketUnion, UniversalQueryTrait, EVENT_PACKET_CATEGORY, QUERY_PACKET_CATEGORY,
            REQUEST_PACKET_CATEGORY, RESPONSE_PACKET_CATEGORY, UNKNOWN_RESPONSE_TYPE_ID_BYTES,
        },
    };
    use std::marker::PhantomData;

    use super::split_packet_category;

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

        /// Returns (query instance id, query id type, query inner data)
        fn split_query_data(query_packet_data: &[u8]) -> (QueryInstanceId, &[u8]) {
            let query_instance_id =
                QueryInstanceId::from_bytes_le(query_packet_data[0..16].try_into().unwrap());

            let query_inner_data = &query_packet_data[16..];

            (query_instance_id, query_inner_data)
        }

        /// responds to all incoming queries and returns a vec of all incoming requests
        pub fn handle_incoming<Request: RequestPacketUnion>(
            &mut self,
            query_responders: &mut Vec<&mut dyn QueryDataResponder>,
        ) -> Vec<Request> {
            let mut requests = Vec::new();

            for incoming_data in self.stream.collect_try_iter_bytes() {
                let (packet_category, packet_type_id, packet_inner_data) =
                    split_packet_category(&incoming_data);

                match packet_category {
                    REQUEST_PACKET_CATEGORY => {
                        if let Some(request) =
                            Request::packet_try_from_data(packet_type_id, packet_inner_data)
                        {
                            requests.push(request);
                        } else {
                            // request not known
                            eprintln!("Warning: Request type id {packet_type_id} with data {:?} could not be parsed", packet_inner_data);
                        }
                    }
                    QUERY_PACKET_CATEGORY => {
                        let (query_instance_id, query_inner_data) =
                            Self::split_query_data(packet_inner_data);

                        let response_packet_data = query_responders.iter_mut().find_map(|query_responder| {
                                if packet_type_id != query_responder.__get_query_type_id() {
                                    return None;
                                }

                                query_responder
                                    .__generate_response_packet_data(query_inner_data, query_instance_id)
                            }).unwrap_or_else(|| {
                                eprintln!("Warning: Query of packet data: `{:?}` and type id `{packet_type_id}` could not be parsed or responded to", packet_inner_data);

                                /// TODO: duplicate in `____generate_response_packet_data`
                                const RESPONSE_PACKET_CATEGORY_BYTES: [u8; 1] = RESPONSE_PACKET_CATEGORY.to_be_bytes();

                                [
                                    RESPONSE_PACKET_CATEGORY_BYTES.as_slice(),
                                    UNKNOWN_RESPONSE_TYPE_ID_BYTES.as_slice(),
                                    query_instance_id.to_bytes_le().as_slice(),
                                    &[],
                                ]
                                .concat()
                            });

                        self.stream.write_bytes(&response_packet_data);
                    }
                    _ => {
                        eprintln!("Warning: Server stream recieved a packet category {packet_category} with type id {packet_type_id} that is not a request or query with data {:?}", packet_inner_data)
                    }
                }
            }

            requests
        }

        pub fn send_event<Event: EventPacketTrait>(&mut self, event: Event) {
            const EVENT_PACKET_CATEGORY_BYTES: [u8; 1] = EVENT_PACKET_CATEGORY.to_be_bytes();

            self.stream.write_bytes(
                &[
                    EVENT_PACKET_CATEGORY_BYTES.as_slice(),
                    Event::PACKET_TYPE_ID.to_be_bytes().as_slice(),
                    &event.to_data(),
                ]
                .concat(),
            );
        }

        pub fn send_event_union(&mut self, event: impl EventPacketUnion) {
            const EVENT_PACKET_CATEGORY_BYTES: [u8; 1] = EVENT_PACKET_CATEGORY.to_be_bytes();

            let (packet_type_id, packet_inner_data) = event.packet_to_data();

            self.stream.write_bytes(
                &[
                    EVENT_PACKET_CATEGORY_BYTES.as_slice(),
                    packet_type_id.to_be_bytes().as_slice(),
                    &packet_inner_data,
                ]
                .concat(),
            );
        }
    }

    pub trait QueryResponder {
        type Query: UniversalQueryTrait;

        fn respond(
            &mut self,
            query: Self::Query,
            query_instance_id: QueryInstanceId,
        ) -> Option<<Self::Query as UniversalQueryTrait>::ResponseType>;
    }
    /// NOTE: don't override this
    /// TODO: make this somehow public but not overridable
    pub trait QueryDataResponder {
        fn __get_query_type_id(&self) -> PacketTypeId;
        fn __generate_response_packet_data(
            &mut self,
            query_data: &[u8],
            query_instance_id: QueryInstanceId,
        ) -> Option<Vec<u8>>;
    }
    impl<R: QueryResponder> QueryDataResponder for R {
        fn __get_query_type_id(&self) -> PacketTypeId {
            R::Query::PACKET_TYPE_ID
        }

        fn __generate_response_packet_data(
            &mut self,
            query_data: &[u8],
            query_instance_id: QueryInstanceId,
        ) -> Option<Vec<u8>> {
            let response_object: <R::Query as UniversalQueryTrait>::ResponseType =
                self.respond(R::Query::try_from_data(query_data)?, query_instance_id)?;

            const RESPONSE_PACKET_CATEGORY_BYTES: [u8; 1] = RESPONSE_PACKET_CATEGORY.to_be_bytes();

            Some(
                [
                    RESPONSE_PACKET_CATEGORY_BYTES.as_slice(),
                    <R::Query as UniversalQueryTrait>::ResponseType::PACKET_TYPE_ID
                        .to_be_bytes()
                        .as_slice(),
                    query_instance_id.to_bytes_le().as_slice(),
                    response_object.to_data().as_slice(),
                ]
                .concat(),
            )
        }
    }

    pub fn as_query_data_responder<
        Q: UniversalQueryTrait,
        F: FnMut(Q) -> Option<Q::ResponseType>,
    >(
        f: F,
    ) -> impl QueryDataResponder {
        struct FnWrapper<Q: UniversalQueryTrait, F: FnMut(Q) -> Option<Q::ResponseType>>(
            F,
            PhantomData<Q>,
        );

        impl<Q: UniversalQueryTrait, F: FnMut(Q) -> Option<Q::ResponseType>> QueryResponder
            for FnWrapper<Q, F>
        {
            type Query = Q;

            fn respond(
                &mut self,
                query: Self::Query,
                _query_instance_id: QueryInstanceId,
            ) -> Option<<Self::Query as UniversalQueryTrait>::ResponseType> {
                self.0(query)
            }
        }

        FnWrapper(f, PhantomData)
    }
}
