//! Run: `cargo expand --manifest-path singularity_common/Cargo.toml --test macro_demo` to expand
//!
//! this file is a proof of concept and if successful,
//! many of the code should go inside the actual source code

// pub type Data = Vec<u8>;

use singularity_common::{
    packet_union,
    sap::packet::{IdType, PacketTrait},
};

// SECTION - should be in shared 3rd party crate or singularity standard

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

/// returns the id (from the beginning) and the rest of the data
fn seperate_id(data: &[u8]) -> (IdType, &[u8]) {
    todo!()
}
fn add_id(id: IdType, data: &[u8]) -> Vec<u8> {
    todo!()
}

packet_union!(pub MyEvent => [ClipboardEvent, DragEvent], 9000);
