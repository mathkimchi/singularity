//! REVIEW: Move

pub mod display_packets {
    use crate::{
        byte_stream::{ToData, TryFromData},
        packet::{IdType, PacketTrait},
    };
    use singularity_macros::{Datable, Packet, PacketUnion};
    use singularity_ui::{display_units::DisplayArea, ui_element::UIElement, ui_event::UIEvent};

    #[derive(Debug, Datable, Packet)]
    pub struct ResizeEvent(DisplayArea);
    #[derive(Debug, Datable, Packet)]
    pub struct FocusedEvent;
    #[derive(Debug, Datable, Packet)]
    pub struct UnfocusedEvent;
    #[derive(Debug, Datable, Packet)]
    pub struct CloseWarningEvent;

    #[derive(Debug, PacketUnion, Packet)]
    pub enum DisplayEvent {
        UIEvent(UIEvent),
        Resize(ResizeEvent),
        Focused(FocusedEvent),
        Unfocused(UnfocusedEvent),
        Close(CloseWarningEvent),
    }

    #[derive(Debug, Datable, Packet)]
    pub struct RequestUpdateWindow {
        contents: UIElement,
    }
}
