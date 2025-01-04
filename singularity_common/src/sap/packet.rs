pub type IdType = u64;

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: std::marker::Sized {
    const PACKET_TYPE_ID: IdType;

    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: &[u8]) -> Option<Self>;
    // fn from_data(data: &[u8]) -> Self;
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

        impl $crate::sap::packet::PacketTrait for $new_name {
            const PACKET_TYPE_ID: IdType = $event_id;

            fn from_data(data: &[u8]) -> Option<Self> {
                let (id, data) = $crate::sap::packet::split_id(data);
                match id {
                    $($subevent::PACKET_TYPE_ID => Some(Self::$subevent($subevent::from_data(data)?)),)*
                    _ => None,
                }
            }

            fn to_data(&self) -> Vec<u8> {
                let (id, data) = match self {
                    $(Self::$subevent(subevent) => ($subevent::PACKET_TYPE_ID, subevent.to_data()),)*
                };

                $crate::sap::packet::join_id(id, &data)
            }
        }
    };
}

pub mod universal_client_socket {
    use super::PacketTrait;
    use crate::sap::byte_stream::{ByteReader, ByteWriter};
    use std::{marker::PhantomData, os::unix::net::UnixStream};

    /// To be used by the client.
    ///
    /// NOTE: technically, I could have the generics be per-function,
    /// but that might require more boilerplate for most cases
    pub struct UniversalClientSocket<Event: PacketTrait, Request: PacketTrait> {
        _r: PhantomData<Request>,

        event_queue: Vec<Event>,
        connection: UnixStream,
    }
    impl<Event: PacketTrait, Request: PacketTrait> UniversalClientSocket<Event, Request> {
        pub fn new(connection: UnixStream) -> Self {
            // now, we should actually expect some reads to cause errors
            // and the good error would be because of timeout/nonblocking
            connection
                .set_nonblocking(true)
                .expect("Couldn't set nonblocking");

            Self {
                _r: PhantomData,
                event_queue: Vec::new(),
                connection,
            }
        }

        fn update_event_queue(&mut self) {
            for raw_data in self.connection.try_iter_bytes() {
                // currently disregard parsing errors (from_data errors),
                // because it might just be an unsupported feature
                if let Some(event) = Event::from_data(&raw_data) {
                    self.event_queue.push(event);
                };
            }
        }

        /// Nonblocking
        pub fn read_events(&mut self) -> Vec<Event> {
            self.update_event_queue();

            std::mem::take(&mut self.event_queue)
        }

        pub fn send_request(&mut self, request: Request) {
            self.connection.write_bytes(&request.to_data());
        }
    }
}

pub mod universal_server_socket {
    use super::PacketTrait;
    use crate::sap::byte_stream::{ByteReader, ByteWriter};
    use std::{marker::PhantomData, os::unix::net::UnixStream};

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
    pub struct UniversalServerSocket<Event: PacketTrait, Request: PacketTrait> {
        _r: PhantomData<Event>,

        request_queue: Vec<Request>,
        connection: UnixStream,
    }
    impl<Event: PacketTrait, Request: PacketTrait> UniversalServerSocket<Event, Request> {
        pub fn new(connection: UnixStream) -> Self {
            // now, we should actually expect some reads to cause errors
            // and the good error would be because of timeout/nonblocking
            connection
                .set_nonblocking(true)
                .expect("Couldn't set nonblocking");

            Self {
                _r: PhantomData,
                request_queue: Vec::new(),
                connection,
            }
        }

        fn update_request_queue(&mut self) {
            for raw_data in self.connection.try_iter_bytes() {
                // currently disregard parsing errors (from_data errors),
                // because it might just be an unsupported feature
                if let Some(request) = Request::from_data(&raw_data) {
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
            self.connection.write_bytes(&event.to_data());
        }
    }
}
