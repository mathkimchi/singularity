use singularity_common::sap::{
    byte_stream::{ByteStream, TryFromData},
    packet::{
        IdType, PacketTrait, PacketType, QueryInstanceId, UniversalQuery, EVENT_PACKET_TYPE,
        QUERY_PACKET_TYPE, REQUEST_PACKET_TYPE, RESPONSE_PACKET_TYPE, UNKNOWN_RESPONSE_TYPE_ID,
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

// /// To be used by the server.
// ///
// /// Represents connection to one client.
// ///
// /// REVIEW: naming, should I name this: `ClientHandler`?
// ///
// /// Right now, server and client socket are literally just the same with event and request switched.
// /// I could abstract to just `UniversalStream<SendPacket, RecvPacket>`,
// /// but I am preparing for queries and responses.
// /// TODO: I could still abstract though.
// ///
// /// TODO: I have an idea: I don't think the server side actually requires a manual queue storage implementation
// pub struct UniversalServerStream<Stream: ByteStream, Request: PacketTrait> {
//     stream: Stream,

//     request_queue: Vec<Request>,
//     // TODO
//     query_data_queue: Vec<(QueryInstanceId, Vec<u8>)>,
// }
// impl<Stream: ByteStream, Request: PacketTrait> UniversalServerStream<Stream, Request> {
//     pub fn new(stream: Stream) -> Self {
//         Self {
//             stream,
//             request_queue: Vec::new(),
//             query_data_queue: Vec::new(),
//         }
//     }

//     /// Returns: (Packet type, packet data)
//     /// For server recieving, packet type ids should be request and query
//     fn split_packet_type(data: &[u8]) -> (PacketType, &[u8]) {
//         (data[0], &data[1..])
//     }

//     fn try_update_queues(&mut self) {
//         for incoming_data in self.stream.try_iter_bytes() {
//             let (packet_type, packet_data) = Self::split_packet_type(&data);
//             match packet_type {
//                 REQUEST_PACKET_TYPE => {
//                     if let Some(event) = Event::try_from_data(packet_data) {
//                         event_queue.push(event);
//                     } else {
//                         // event not known
//                         eprintln!("Warning: Event {:?} could not be parsed", packet_data);
//                     }
//                 }
//                 QUERY_PACKET_TYPE => {
//                     response_data_queue.push(packet_data.to_vec());
//                 }
//                 other => {
//                     eprintln!("Warning: Client stream recieved a packet type {other} that is not an event or response with data {:?}", packet_data)
//                 }
//             }
//         }
//     }

//     /// Nonblocking
//     pub fn read_requests(&mut self) -> Vec<Request> {
//         self.try_update_queues();

//         std::mem::take(&mut self.request_queue)
//     }

//     pub fn send_event<Event: PacketTrait>(&mut self, event: Event) {
//         self.stream.write_bytes(&event.to_data());
//     }
// }
