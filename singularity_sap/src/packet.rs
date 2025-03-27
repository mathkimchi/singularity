use super::datable::Datable;
use uuid::Uuid;

/// Previously IdType, but that is kind of confusing.
/// TODO: make this a struct instead of a type alias?
pub type PacketTypeId = u64;

// REVIEW: I actually hate this, and would prefer to do a new channel per packet type, but that uses unnecessary resources.
// TODO: a way to make this less horrible would be to have it an enum
pub type PacketCategory = u8;
pub const EVENT_PACKET_CATEGORY: PacketCategory = 0;
pub const REQUEST_PACKET_CATEGORY: PacketCategory = 1;
pub const QUERY_PACKET_CATEGORY: PacketCategory = 2;
pub const RESPONSE_PACKET_CATEGORY: PacketCategory = 3;

pub type QueryInstanceId = Uuid;

pub const UNKNOWN_RESPONSE_TYPE_ID: PacketTypeId = 404;
pub const UNKNOWN_RESPONSE_TYPE_ID_BYTES: [u8; 8] = UNKNOWN_RESPONSE_TYPE_ID.to_be_bytes();

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: Datable {
    const PACKET_TYPE_ID: PacketTypeId;
}

pub trait UniversalQueryTrait: PacketTrait {
    type ResponseType: PacketTrait;
}

/// REVIEW: make this just `PacketUnion` and just have a `EventPacketUnion: PacketUnion`?
///
/// [`EventPacketUnion`] is like [`Datable`], but since it has different uses, they are different traits
/// to reduce confusion.
// pub trait EventPacketUnion: Sized {
pub trait PacketUnion: Sized {
    fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>);
    fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self>;
}

/// Just for safety
pub trait EventPacketUnion: PacketUnion {}

/// Just for safety
pub trait RequestPacketUnion: PacketUnion {}

/// Just for safety
pub trait EventPacketTrait: PacketTrait {}

/// Just for safety
pub trait RequestPacketTrait: PacketTrait {}
