use singularity_common::{
    packet_union,
    sap::packet::{IdType, PacketTrait},
};

pub struct CopiedEvent;
impl PacketTrait for CopiedEvent {
    const PACKET_TYPE_ID: IdType = 98752896453;

    fn to_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_data(_: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

pub struct PastedEvent(String);
impl PacketTrait for PastedEvent {
    const PACKET_TYPE_ID: IdType = 4325678983412657;

    fn to_data(&self) -> Vec<u8> {
        // utf8
        self.0.as_bytes().to_vec()
    }

    fn from_data(data: &[u8]) -> Option<Self> {
        Some(Self(String::from_utf8(data.to_vec()).unwrap()))
    }
}

pub struct DraggedEvent;
impl PacketTrait for DraggedEvent {
    const PACKET_TYPE_ID: IdType = 17859015767526;

    fn to_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_data(_: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

packet_union!(pub ClipboardEvent => [CopiedEvent, PastedEvent], 23945086);
packet_union!(pub DragEvent => [DraggedEvent], 982734516872348);
packet_union!(pub MyEvent => [ClipboardEvent, DragEvent], 9000);

pub struct Request;
impl PacketTrait for Request {
    const PACKET_TYPE_ID: IdType = 17859015767526;

    fn to_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_data(_: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

#[test]
fn sap_connection_test() {
    // let server = ServerHost;
}
