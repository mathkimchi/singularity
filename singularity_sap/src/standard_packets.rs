//! REVIEW: Move

pub mod display_packets {
    use crate::{
        datable::{ToData, TryFromData},
        packet::{IdType, PacketTrait, UniversalQuery},
    };
    use singularity_common::utils::tree::tree_node_path::TreeNodePath;
    use singularity_macros::{Datable, Packet, PacketUnion};
    use singularity_ui::{display_units::DisplayArea, ui_element::UIElement, ui_event::UIEvent};

    #[derive(Debug, Datable, Packet)]
    pub struct ResizeEvent(pub DisplayArea);
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

    /// TODO: use shared mem instead of requests for display
    #[derive(Debug, Datable, Packet)]
    pub struct RequestUpdateWindow {
        pub contents: UIElement,
    }

    #[derive(Debug, Datable, Packet)]
    pub struct RequestChangeName {
        pub new_name: String,
    }
    impl RequestChangeName {
        pub fn new<S: ToString>(new_name: &S) -> Self {
            Self {
                new_name: new_name.to_string(),
            }
        }
    }

    #[derive(Debug, Datable, Packet)]
    pub struct PathQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct PathResponse(pub TreeNodePath);
    impl UniversalQuery for PathQuery {
        type ResponseType = PathResponse;
    }

    #[derive(Debug, Datable, Packet)]
    pub struct NameQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct NameResponse(pub String);
    impl UniversalQuery for NameQuery {
        type ResponseType = NameResponse;
    }

    // #[derive(Debug, Datable, Packet)]
    // pub struct TabDataQuery;
    // #[derive(Debug, Datable, Packet)]
    // pub struct TabDataResponse(pub TabData);
    // impl UniversalQuery for TabDataQuery {
    //     type ResponseType = TabDataResponse;
    // }
}
