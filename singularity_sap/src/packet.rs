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

    /// Is useless on its own, but useful for packet unions,
    /// which is why this function's inverse `packet_try_from_typed_data` is not needed (probably).
    /// Contains both the packet type id and packet inner data.
    /// `PacketUnion` also has this.
    fn packet_to_typed_data(&self) -> Vec<u8> {
        let packet_inner_data = self.to_data();

        [
            Self::PACKET_TYPE_ID.to_be_bytes().as_slice(),
            &packet_inner_data,
        ]
        .concat()
    }
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
    /// TODO: documentation
    /// REVIEW: rename
    fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>);
    /// TODO: documentation
    /// REVIEW: rename
    /// TODO: output `Option` -> `Result`
    fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self>;

    /// `packet_to_typed_data` has the same signature as `ToData`,
    /// but `ToData` is deliberately not implemented (by default) to
    /// eliminate potential misuse.
    ///
    /// The output contains data for both the PacketTypeId,
    /// as well as the packet inner data.
    ///
    /// TODO: search `to_be_bytes` and `as_slice` for places to upgrade with this function.
    fn packet_to_typed_data(&self) -> Vec<u8> {
        // NOTE: I am not using `self.packet_to_data().to_data()`
        // even though it is elegant and conforms with DRY,
        // because tuple's ToData has extra bytes and the efficiency isn't the problem,
        // and I want this serialization to be standard. (like in universal_stream)

        // TODO: later, maybe if I make Datable for tuples better, just use:
        // self.packet_to_data().to_data()

        let (packet_type_id, packet_inner_data) = self.packet_to_data();

        [packet_type_id.to_be_bytes().as_slice(), &packet_inner_data].concat()
    }
    /// `packet_try_from_typed_data` has the same signature as `TryFromData`,
    /// but `TryFromData` is deliberately not implemented (by default) to
    /// eliminate potential misuse.
    ///
    /// REVIEW: rename
    ///
    /// TODO: search `from_be_bytes` for places to upgrade with this function.
    fn packet_try_from_typed_data(packet_typed_data: &[u8]) -> Option<Self> {
        let packet_id = PacketTypeId::from_be_bytes(
            packet_typed_data[0..((PacketTypeId::BITS as usize) / 8)]
                .try_into()
                .unwrap(),
        );
        let packet_inner_data = &packet_typed_data[((PacketTypeId::BITS as usize) / 8)..];
        Self::packet_try_from_data(packet_id, packet_inner_data)
    }
}

/// Just for safety
pub trait EventPacketUnion: PacketUnion {}

/// Just for safety
pub trait RequestPacketUnion: PacketUnion {}

/// Just for safety
pub trait EventPacketTrait: PacketTrait {}

/// Just for safety
pub trait RequestPacketTrait: PacketTrait {}
