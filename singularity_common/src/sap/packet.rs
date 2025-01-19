use super::byte_stream::Datable;
use uuid::Uuid;

pub type IdType = u64;

pub type PacketType = u8;
pub const EVENT_PACKET_TYPE: PacketType = 0;
pub const REQUEST_PACKET_TYPE: PacketType = 1;
pub const QUERY_PACKET_TYPE: PacketType = 2;
pub const RESPONSE_PACKET_TYPE: PacketType = 3;

pub type QueryInstanceId = Uuid;

pub const UNKNOWN_RESPONSE_TYPE_ID: IdType = 404;
pub const UNKNOWN_RESPONSE_TYPE_ID_BYTES: [u8; 8] = UNKNOWN_RESPONSE_TYPE_ID.to_be_bytes();

/// Like a more specific version of serde's serialize and deserialize
pub trait PacketTrait: Datable {
    const PACKET_TYPE_ID: IdType;
}

pub trait UniversalQuery: PacketTrait {
    type ResponseType: PacketTrait;
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
