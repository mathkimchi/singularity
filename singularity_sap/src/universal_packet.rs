//! REVIEW: Parts of this might need to be put outside singularity common and into a sap_feature_tk

use super::{
    byte_stream::{ByteReader, ByteWriter},
    packet::IdType,
};

pub trait PacketConverter<InnerData, const ID: IdType> {
    // NOTE: I tried `... -> [u8]`, which compiled, but I couldn't impl it
    fn data_to_bytes(data: InnerData) -> Vec<u8>;
    fn data_from_bytes(raw_data: &[u8]) -> InnerData;

    fn to_universal_packet(data: InnerData) -> UniversalPacket {
        UniversalPacket {
            id: ID,
            raw_data: Self::data_to_bytes(data),
        }
    }

    fn try_from_universal_packet(universal_packet: UniversalPacket) -> Option<InnerData> {
        (universal_packet.id == ID).then(move || Self::data_from_bytes(&universal_packet.raw_data))
    }
}

pub struct UniversalPacket {
    id: IdType,
    raw_data: Vec<u8>,
}

pub trait UniversalPacketReader {
    fn read_universal_packet(&mut self) -> UniversalPacket;
}
impl<R: ByteReader> UniversalPacketReader for R {
    fn read_universal_packet(&mut self) -> UniversalPacket {
        let bytes = self.read_bytes();
        let (id, data) = bytes.split_at((IdType::BITS / 8) as usize);

        UniversalPacket {
            id: IdType::from_be_bytes(id.try_into().unwrap()),
            raw_data: data.into(),
        }
    }
}

pub trait UniversalPacketWriter {
    fn write_universal_packet(&mut self, universal_packet: UniversalPacket);
}
impl<W: ByteWriter> UniversalPacketWriter for W {
    fn write_universal_packet(&mut self, mut universal_packet: UniversalPacket) {
        let mut bytes = universal_packet.id.to_be_bytes().to_vec();
        bytes.append(&mut universal_packet.raw_data);
        self.write_bytes(&bytes);
    }
}
