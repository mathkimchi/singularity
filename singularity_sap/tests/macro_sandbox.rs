use singularity_macros::{Datable, Packet};
use singularity_sap::{
    byte_stream::{ToData, TryFromData},
    packet::{IdType, PacketTrait},
};
use singularity_ui::display_units::DisplayArea;

/// Not how I would actually do paste, but I just want to test with multiple unknown size fields.
pub struct PastePacket {
    clean_content: String,
    raw_content: String,
}
impl ToData for PastePacket {
    fn to_data(&self) -> Vec<u8> {
        // setup field data
        let clean_content_bytes = self.clean_content.to_data();
        let clean_content_len = clean_content_bytes.len().to_be_bytes();
        let raw_content_bytes = self.raw_content.to_data();
        let raw_content_len = raw_content_bytes.len().to_be_bytes();

        // combine field data
        [
            clean_content_len.as_slice(),
            clean_content_bytes.as_slice(),
            raw_content_len.as_slice(),
            raw_content_bytes.as_slice(),
        ]
        .concat()
    }
}
impl TryFromData for PastePacket {
    fn try_from_data(data: &[u8]) -> Option<Self> {
        let mut index = 0;
        let s = Self {
            clean_content: {
                let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                index += 8;

                let inner_data = &data[index..(index + len)];
                index += len;

                String::try_from_data(inner_data)?
            },
            raw_content: {
                let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                index += 8;

                let inner_data = &data[index..(index + len)];
                index += len;

                String::try_from_data(inner_data)?
            },
        };

        if index != data.len() {
            return None;
        }

        Some(s)
    }
}

#[derive(Debug, Datable, Packet)]
pub struct AEvent {
    d: DisplayArea,
}
#[derive(Debug, Datable, Packet)]
pub struct ResizeEvent(DisplayArea);
#[derive(Debug, Datable, Packet)]
pub struct FocusedEvent;
