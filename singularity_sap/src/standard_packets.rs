//! REVIEW: Move

pub mod display_packets {
    use crate::{
        byte_stream::{ToData, TryFromData},
        packet::{self, IdType, PacketTrait},
    };
    use singularity_macros::Packet;
    use singularity_ui::display_units::DisplayArea;

    #[derive(Debug, Packet)]
    pub struct ResizeEvent(DisplayArea);
    #[derive(Debug, Packet)]
    pub struct FocusedEvent;
    #[derive(Debug, Packet)]
    pub struct UnfocusedEvent;
    #[derive(Debug, Packet)]
    pub struct CloseWarningEvent;

    #[derive(Debug, Packet)]
    pub enum DisplayEvent {
        // UIEvent(UIEvent),
        Resize(ResizeEvent),
        Focused(FocusedEvent),
        Unfocused(UnfocusedEvent),
        /// TODO: close forcibly
        Close(CloseWarningEvent),
    }
}
