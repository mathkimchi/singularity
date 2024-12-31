//! Run: `cargo expand --manifest-path singularity_common/Cargo.toml --test macro_demo` to expand
//!
//! this file is a proof of concept and if successful,
//! many of the code should go inside the actual source code

// pub type Data = Vec<u8>;

use singularity_common::sap::universal_packet::IdType;

/// Like a more specific version of serde's serialize and deserialize
trait PacketTrait: std::marker::Sized {
    fn to_data(&self) -> Vec<u8>;
    fn from_data(data: &[u8]) -> Option<Self>;
    // fn from_data(data: &[u8]) -> Self;
}

trait EventTrait: PacketTrait {
    const EVENT_TYPE_ID: IdType;
}

enum ClipboardEvent {
    Copied,
    Pasted(String),
}
enum DragEvent {
    Dragged,
}

// of course, the impls should be derive later
impl PacketTrait for ClipboardEvent {
    fn to_data(&self) -> Vec<u8> {
        todo!()
    }

    fn from_data(_data: &[u8]) -> Option<Self> {
        todo!()
    }
}
impl EventTrait for ClipboardEvent {
    const EVENT_TYPE_ID: IdType = 42;
}
impl PacketTrait for DragEvent {
    fn to_data(&self) -> Vec<u8> {
        todo!()
    }

    fn from_data(_data: &[u8]) -> Option<Self> {
        todo!()
    }
}
impl EventTrait for DragEvent {
    const EVENT_TYPE_ID: IdType = 12345;
}

/// returns the id (from the beginning) and the rest of the data
fn seperate_id(data: &[u8]) -> (IdType, &[u8]) {
    todo!()
}
fn add_id(id: IdType, data: &[u8]) -> Vec<u8> {
    todo!()
}

enum MyEvent {
    ClipboardEvent(ClipboardEvent),
    DragEvent(DragEvent),
}
impl PacketTrait for MyEvent {
    fn from_data(data: &[u8]) -> Option<Self> {
        let (id, data) = seperate_id(data);
        match id {
            ClipboardEvent::EVENT_TYPE_ID => {
                Some(Self::ClipboardEvent(ClipboardEvent::from_data(data)?))
            }
            DragEvent::EVENT_TYPE_ID => Some(Self::DragEvent(DragEvent::from_data(data)?)),
            _ => None,
        }
    }

    fn to_data(&self) -> Vec<u8> {
        let (id, data) = match self {
            MyEvent::ClipboardEvent(clipboard_event) => {
                (ClipboardEvent::EVENT_TYPE_ID, clipboard_event.to_data())
            }
            MyEvent::DragEvent(drag_event) => (DragEvent::EVENT_TYPE_ID, drag_event.to_data()),
        };

        add_id(id, &data)
    }
}
