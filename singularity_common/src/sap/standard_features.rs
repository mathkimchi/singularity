//! REVIEW: Move

pub mod display {
    use crate::sap::packet::PacketConverter;
    use serde::{Deserialize, Serialize};
    use singularity_ui::display_units::DisplayArea;

    #[derive(Serialize, Deserialize)]
    pub enum DisplayEvent {
        // UIEvent(UIEvent),
        Resize(DisplayArea),
        Focused,
        Unfocused,
        /// TODO: close forcibly
        Close,
    }

    pub struct DisplayEventConverter;

    impl PacketConverter<DisplayEvent, 123456> for DisplayEventConverter {
        fn data_to_bytes(data: DisplayEvent) -> Vec<u8> {
            serde_json::to_vec(&data).unwrap()
        }

        fn data_from_bytes(raw_data: &[u8]) -> DisplayEvent {
            serde_json::from_slice(raw_data).unwrap()
        }
    }
}
