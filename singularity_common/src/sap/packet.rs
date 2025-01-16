use uuid::Uuid;

use super::byte_stream::Datable;

pub type PacketType = u8;
pub const EVENT_PACKET_TYPE: PacketType = 0;
pub const REQUEST_PACKET_TYPE: PacketType = 1;
pub const QUERY_PACKET_TYPE: PacketType = 2;
pub const RESPONSE_PACKET_TYPE: PacketType = 3;

pub type QueryInstanceId = Uuid;

pub const UNKNOWN_RESPONSE_TYPE_ID: IdType = 404;
pub const UNKNOWN_RESPONSE_TYPE_ID_BYTES: [u8; 8] = UNKNOWN_RESPONSE_TYPE_ID.to_be_bytes();

pub type IdType = u64;

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: Datable {
    const PACKET_TYPE_ID: IdType;
}

/// returns the id (from the beginning) and the rest of the data
pub fn split_id(data: &[u8]) -> (IdType, &[u8]) {
    let (id_bytes, inner_data) = data.split_at((IdType::BITS / 8) as usize);

    let id = IdType::from_be_bytes(id_bytes.try_into().unwrap());

    (id, inner_data)
}
pub fn join_id(id: IdType, inner_data: &[u8]) -> Vec<u8> {
    let id_bytes: &[u8] = &id.to_be_bytes();

    [id_bytes, inner_data].concat()
}

/// NOTE: The subevents are actually both idents and types.
/// Idents can be types, but types can't be idents (easily),
/// which is why I told the macro subevents are idents.
#[deprecated = "use PacketUnion from singularity_macros instead"]
#[macro_export]
macro_rules! packet_union {
    // ($($v:vis)? $new_name:ident => [$($subevent:ty),*]) => {
    //     enum $new_name {}
    // };

    // I guess vis is special, so no need for the optional with ?
    ($v:vis $new_name:ident => [$($subevent:ident),*], $event_id:expr) => {
        $v enum $new_name {
            $($subevent($subevent),)*
        }

        impl $crate::sap::byte_stream::ToData for $new_name {
            fn to_data(&self) -> Vec<u8> {
                let (id, data) = match self {
                    $(Self::$subevent(subevent) => ($subevent::PACKET_TYPE_ID, subevent.to_data()),)*
                };

                $crate::sap::packet::join_id(id, &data)
            }
        }

        impl $crate::sap::byte_stream::TryFromData for $new_name {
            fn try_from_data(data: &[u8]) -> Option<Self> {
                let (id, data) = $crate::sap::packet::split_id(data);
                match id {
                    $($subevent::PACKET_TYPE_ID => Some(Self::$subevent($subevent::try_from_data(data)?)),)*
                    _ => None,
                }
            }
        }

        impl $crate::sap::packet::PacketTrait for $new_name {
            const PACKET_TYPE_ID: IdType = $event_id;
        }
    };
}

pub mod universal_client_stream {
    use super::PacketTrait;
    use crate::sap::byte_stream::ByteStream;

    /// To be used by the client.
    ///
    /// NOTE: technically, I could have the generics be per-function,
    /// but that might require more boilerplate for most cases
    pub struct UniversalClientStream<Stream: ByteStream, Event: PacketTrait> {
        stream: Stream,

        event_queue: Vec<Event>,
    }
    impl<Stream: ByteStream, Event: PacketTrait> UniversalClientStream<Stream, Event> {
        pub fn new(stream: Stream) -> Self {
            Self {
                stream,
                event_queue: Vec::new(),
            }
        }

        fn update_event_queue(&mut self) {
            for raw_data in self.stream.try_iter_bytes() {
                // currently disregard parsing errors (try_from_data errors),
                // because it might just be an unsupported feature
                if let Some(event) = Event::try_from_data(&raw_data) {
                    self.event_queue.push(event);
                };
            }
        }

        /// Nonblocking
        pub fn try_read_events(&mut self) -> Vec<Event> {
            self.update_event_queue();

            std::mem::take(&mut self.event_queue)
        }

        pub fn send_request<Request: PacketTrait>(&mut self, request: Request) {
            self.stream.write_bytes(&request.to_data());
        }
    }
}

pub mod universal_server_stream {
    use super::PacketTrait;
    use crate::sap::byte_stream::ByteStream;
    use std::marker::PhantomData;

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
    pub struct UniversalServerStream<Stream: ByteStream, Event: PacketTrait, Request: PacketTrait> {
        stream: Stream,

        request_queue: Vec<Request>,

        _r: PhantomData<Event>,
    }
    impl<Stream: ByteStream, Event: PacketTrait, Request: PacketTrait>
        UniversalServerStream<Stream, Event, Request>
    {
        pub fn new(stream: Stream) -> Self {
            Self {
                stream,
                request_queue: Vec::new(),
                _r: PhantomData,
            }
        }

        fn update_request_queue(&mut self) {
            for raw_data in self.stream.try_iter_bytes() {
                // currently disregard parsing errors (try_from_data errors),
                // because it might just be an unsupported feature
                if let Some(request) = Request::try_from_data(&raw_data) {
                    self.request_queue.push(request);
                };
            }
        }

        /// Nonblocking
        pub fn read_requests(&mut self) -> Vec<Request> {
            self.update_request_queue();

            std::mem::take(&mut self.request_queue)
        }

        pub fn send_event(&mut self, event: Event) {
            self.stream.write_bytes(&event.to_data());
        }
    }
}
