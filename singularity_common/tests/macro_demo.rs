//! Run: `cargo expand --manifest-path singularity_common/Cargo.toml --test macro_demo` to expand
//!
//! this file is a proof of concept and if successful,
//! many of the code should go inside the actual source code

#![allow(unused)]

use singularity_common::{
    packet_union,
    sap::packet::{IdType, PacketTrait},
};

enum ClipboardEvent {
    Copied,
    Pasted(String),
}
enum DragEvent {
    Dragged,
}

// of course, the impls should be derive later
impl PacketTrait for ClipboardEvent {
    const PACKET_TYPE_ID: IdType = 42;
    fn to_data(&self) -> Vec<u8> {
        todo!()
    }

    fn from_data(_data: &[u8]) -> Option<Self> {
        todo!()
    }
}
impl PacketTrait for DragEvent {
    const PACKET_TYPE_ID: IdType = 12345;
    fn to_data(&self) -> Vec<u8> {
        todo!()
    }

    fn from_data(_data: &[u8]) -> Option<Self> {
        todo!()
    }
}

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
