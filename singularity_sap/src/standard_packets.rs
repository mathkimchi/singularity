//! REVIEW: Move

pub mod display_packets {
    use crate::{
        datable::{ToData, TryFromData},
        packet::{
            EventPacketTrait, EventPacketUnion, PacketTrait, PacketTypeId, PacketUnion,
            RequestPacketTrait, UniversalQueryTrait,
        },
    };
    use singularity_common::utils::tree::tree_node_path::TreeNodePath;
    use singularity_macros::{Datable, Event, EventPacketUnion, Packet, Request};
    use singularity_sporg::project_settings::TabData;
    use singularity_ui::{display_units::DisplayArea, ui_element::UIElement, ui_event::UIEvent};
    use std::ffi::OsString;

    #[derive(Debug, Datable, Packet, Event)]
    pub struct ResizeEvent(pub DisplayArea);
    #[derive(Debug, Datable, Packet, Event)]
    pub struct FocusedEvent;
    #[derive(Debug, Datable, Packet, Event)]
    pub struct UnfocusedEvent;
    #[derive(Debug, Datable, Packet, Event)]
    pub struct CloseWarningEvent;

    #[derive(Debug, EventPacketUnion)]
    pub enum DisplayEvent {
        UIEvent(UIEvent),
        Resize(ResizeEvent),
        Focused(FocusedEvent),
        Unfocused(UnfocusedEvent),
        Close(CloseWarningEvent),
    }

    /// TODO: use shared mem instead of requests for display
    #[derive(Debug, Datable, Packet, Request)]
    pub struct RequestUpdateWindow {
        pub contents: UIElement,
    }

    #[derive(Debug, Datable, Packet, Request)]
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

    #[derive(Debug, Datable, Packet, Request)]
    pub struct RequestSpawnChildTab(pub TabData);
    impl RequestSpawnChildTab {
        pub fn new(
            tab_command_program: impl Into<OsString>,
            args: impl Iterator<Item = impl Into<OsString>>,
            session_data: serde_json::Value,
        ) -> Self {
            Self(TabData::new(tab_command_program, args, session_data))
        }
    }

    #[derive(Debug, Datable, Packet)]
    pub struct PathQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct PathResponse(pub TreeNodePath);
    impl UniversalQueryTrait for PathQuery {
        type ResponseType = PathResponse;
    }

    #[derive(Debug, Datable, Packet)]
    pub struct NameQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct NameResponse(pub String);
    impl UniversalQueryTrait for NameQuery {
        type ResponseType = NameResponse;
    }

    #[derive(Debug, Datable, Packet)]
    pub struct SessionDataQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct SessionDataResponse(pub serde_json::Value);
    impl UniversalQueryTrait for SessionDataQuery {
        type ResponseType = SessionDataResponse;
    }
}

// pub mod file_packets {
//     use crate::{
//         datable::{ToData, TryFromData},
//         packet::{IdType, PacketTrait, UniversalQuery},
//     };
//     use singularity_macros::{Datable, Packet};

//     #[derive(Debug, Datable, Packet)]
//     pub struct ReadFileRequest;
//     #[derive(Debug, Datable, Packet)]
//     pub struct ReadFileResponse(pub serde_json::Value);
//     impl UniversalQuery for ReadFileRequest {
//         type ResponseType = ReadFileResponse;
//     }
// }

// pub mod broadcast {
//     //! Broadcast messages are themselves packets.
//     //! TODO: might be good to do `trait BroadcastMessage: Packet` and the same thing for events, for type-safety.

//     use crate::{
//         datable::{ToData, TryFromData},
//         packet::{IdType, PacketTrait},
//     };
//     use singularity_macros::{Datable, Packet};

//     /// From one subapp to server, which should forward the message to all subscribed subapps.
//     #[derive(Debug, Datable, Packet)]
//     pub struct BroadcastRequest {
//         message_bytes: Vec<u8>,
//     }

//     pub struct BroadcastEvent {}
// }
