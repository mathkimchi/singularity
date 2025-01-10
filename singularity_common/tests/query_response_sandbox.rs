//! This is the little sandbox file where I will try out query response
//! code and sift out ideas.
//!
//! Right now, I am just going to assume its only queries and responses
//! being sent.

use singularity_common::sap::{
    byte_stream::{ByteReader, ByteWriter},
    packet::{IdType, PacketTrait},
};
use std::os::unix::net::UnixStream;
use uuid::Uuid;

pub type PacketType = u8;
pub const EVENT_PACKET_TYPE: PacketType = 0;
pub const REQUEST_PACKET_TYPE: PacketType = 1;
pub const QUERY_PACKET_TYPE: PacketType = 2;
pub const RESPONSE_PACKET_TYPE: PacketType = 3;

pub type QueryInstanceId = Uuid;

pub struct UniversalQuerier {
    connection: UnixStream,

    queue: Vec<Vec<u8>>,
}
impl UniversalQuerier {
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
            const PACKET_TYPE: [u8; 1] = REQUEST_PACKET_TYPE.to_be_bytes();
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

        // FIXME: I really gotta do something standard about the nonblocking
        self.connection.set_nonblocking(false).unwrap();
        let response = loop {
            if let Some(incoming_packet_bytes) = self.connection.try_read_bytes() {
                if let Some((incoming_query_instance_id, response_type_id, inner_data)) =
                    Self::split_response_bytes(&incoming_packet_bytes)
                {
                    if incoming_query_instance_id == query_instance_id {
                        break if response_type_id == Q::ResponseType::PACKET_TYPE_ID {
                            Q::ResponseType::from_data(&inner_data)
                        } else {
                            // the instance id matches but the type doesn't
                            // just assume it is the null response
                            None
                        };
                    }
                }

                // didn't break, the incoming packet is for something else
                self.queue.push(incoming_packet_bytes);
            }
        };
        self.connection.set_nonblocking(true).unwrap();

        response
    }
}

pub trait UniversalQuery: PacketTrait {
    type ResponseType: PacketTrait;
}

pub struct AddQuery {
    lhs: f32,
    rhs: f32,
}
impl PacketTrait for AddQuery {
    const PACKET_TYPE_ID: u64 = 287561582341;

    fn to_data(&self) -> Vec<u8> {
        [self.lhs.to_be_bytes(), self.rhs.to_be_bytes()].concat()
    }

    fn from_data(data: &[u8]) -> Option<Self> {
        // TODO: tuple to and from bytes abstraction
        let lhs_bytes: [u8; 4] = data[0..4].try_into().unwrap();
        let rhs_bytes: [u8; 4] = data[4..8].try_into().unwrap();

        Some(Self {
            lhs: f32::from_be_bytes(lhs_bytes),
            rhs: f32::from_be_bytes(rhs_bytes),
        })
    }
}

pub struct AddResponse(f32);
impl PacketTrait for AddResponse {
    const PACKET_TYPE_ID: u64 = 1823795123589;

    fn to_data(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }

    fn from_data(data: &[u8]) -> Option<Self> {
        let bytes: [u8; 4] = data[0..4].try_into().unwrap();

        Some(Self(f32::from_be_bytes(bytes)))
    }
}

impl UniversalQuery for AddQuery {
    type ResponseType = AddResponse;
}

/**********************************
 * below should be macro generated
 */

pub enum MySupportedQuery {
    AddQuery(AddQuery),
}

pub struct MyQueryResponder {
    connection: UnixStream,
}

/*
 * above should be macro generated
 **********************************
 */
