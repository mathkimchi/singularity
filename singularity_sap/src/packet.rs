use super::datable::Datable;
use uuid::Uuid;

/// Previously IdType, but that is kind of confusing.
/// REVIEW: make `PacketId` a struct instead of a type alias?
pub type PacketId = u64;

pub type PacketType = u8;
pub const EVENT_PACKET_TYPE: PacketType = 0;
pub const REQUEST_PACKET_TYPE: PacketType = 1;
pub const QUERY_PACKET_TYPE: PacketType = 2;
pub const RESPONSE_PACKET_TYPE: PacketType = 3;

pub type QueryInstanceId = Uuid;

pub const UNKNOWN_RESPONSE_TYPE_ID: PacketId = 404;
pub const UNKNOWN_RESPONSE_TYPE_ID_BYTES: [u8; 8] = UNKNOWN_RESPONSE_TYPE_ID.to_be_bytes();

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: Datable {
    const PACKET_TYPE_ID: PacketId;
}

pub trait UniversalQueryTrait: PacketTrait {
    type ResponseType: PacketTrait;
}

/// REVIEW: make this just `PacketUnion` and just have a `EventPacketUnion: PacketUnion`?
///
/// [`EventPacketUnion`] is like [`Datable`], but since it has different uses, they are different traits
/// to reduce confusion.
pub trait EventPacketUnion: Sized {
    fn packet_to_data(&self) -> (PacketId, Vec<u8>);
    fn packet_try_from_data(packet_id: PacketId, packet_data: &[u8]) -> Option<Self>;
}

/// Just for safety
pub trait EventPacketTrait: PacketTrait {}

/// Just for safety
pub trait RequestPacketTrait: PacketTrait {}
