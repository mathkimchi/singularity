//! Run: `cargo expand --manifest-path singularity_common/Cargo.toml --test sap_macro_test` to expand
//!
//! this file is a proof of concept and if successful,
//! many of the code should go inside the actual source code
//! TODO: make `#[derive(Event)]` macro for things like ClipboardEvent

#![allow(unused)]

use singularity_common::{
    packet_union,
    sap::packet::{IdType, PacketTrait},
};
use singularity_macros::PacketUnion;

#[derive(PartialEq, Eq, Debug)]
pub struct CopiedEvent;
impl PacketTrait for CopiedEvent {
    const PACKET_TYPE_ID: IdType = 98752896453;

    fn to_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_data(data: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

#[derive(PartialEq, Eq, Debug)]
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

packet_union!(pub ClipboardEvent => [CopiedEvent, PastedEvent], 23945086);

pub struct DraggedEvent;
impl PacketTrait for DraggedEvent {
    const PACKET_TYPE_ID: IdType = 17859015767526;

    fn to_data(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_data(data: &[u8]) -> Option<Self> {
        Some(Self)
    }
}

packet_union!(pub DragEvent => [DraggedEvent], 982734516872348);

packet_union!(pub MyEvent => [ClipboardEvent, DragEvent], 9000);

#[test]
fn split_and_join_id_test() {
    use singularity_common::sap::packet::{join_id, split_id};

    assert_eq!(split_id(&[0, 0, 0, 0, 0, 0, 0, 42]), (42u64, [].as_slice()));

    assert_eq!(
        split_id(&[0, 0, 0, 0, 0, 0, 0, 31, 41, 59, 26]),
        (31u64, [41, 59, 26].as_slice())
    );

    assert_eq!(
        split_id(&[0xab, 0xcd, 0, 0, 0, 0, 0, 0x2e, 41, 59, 26]),
        (0xabcd00000000002e, [41, 59, 26].as_slice())
    );

    assert_eq!(
        vec![0, 0, 0, 0, 0, 0, 0, 31, 41, 59, 26],
        join_id(31u64, [41, 59, 26].as_slice())
    );

    assert_eq!(
        vec![0xab, 0xcd, 0, 0, 0, 0, 0, 0x2e, 41, 59, 26],
        join_id(0xabcd00000000002e, [41, 59, 26].as_slice())
    );
}

#[test]
fn derive_packet_test() {
    #[derive(PacketUnion)]
    enum E {
        D(DragEvent),
        C(ClipboardEvent),
    }
}

#[test]
fn packet_test() {
    // TODO
}
