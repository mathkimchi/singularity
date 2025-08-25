//! REVIEW: Move

use crate::{
    datable::{ToData, TryFromData},
    packet::{EventPacketUnion, PacketTrait, PacketTypeId, PacketUnion, RequestPacketUnion},
};
use display_packets::{DisplayEvent, DisplayRequest};
use file_packets::WriteFileRequest;
use singularity_macros::{EventPacketUnion, RequestPacketUnion};

pub mod display_packets {
    use crate::{
        datable::{ToData, TryFromData},
        packet::{
            EventPacketTrait, EventPacketUnion, PacketTrait, PacketTypeId, PacketUnion,
            RequestPacketTrait, RequestPacketUnion, UniversalQueryTrait,
        },
    };
    use singularity_common::utils::tree::tree_node_path::TreeNodePath;
    use singularity_macros::{
        Datable, Event, EventPacketUnion, Packet, Query, Request, RequestPacketUnion,
    };
    use singularity_sporg::applet_data::{AppletSpawnData, AppletTypeId};
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
    pub struct RequestSpawnChildTab(pub AppletSpawnData);
    impl RequestSpawnChildTab {
        pub fn new(
            applet_type_id: Option<AppletTypeId>,
            applet_spawn_command: impl Into<OsString>,
            args: impl Iterator<Item = impl Into<OsString>>,
            initial_session_storage: serde_json::Value,
        ) -> Self {
            Self(AppletSpawnData::new_pipe_child_process(
                applet_type_id,
                applet_spawn_command,
                args,
                initial_session_storage,
            ))
        }
    }

    /// If the specified applet type has a default instance to spawn, then spawn it. Otherwise, just ignored (TODO: I should really do some err handling)
    #[derive(Debug, Datable, Packet, Request)]
    pub struct RequestSpawnDefaultChildApplet(pub AppletTypeId);

    #[derive(Debug, RequestPacketUnion)]
    pub enum DisplayRequest {
        RequestChangeName(RequestChangeName),
        RequestUpdateWindow(RequestUpdateWindow),
        RequestSpawnChildTab(RequestSpawnChildTab),
        RequestSpawnDefaultChildApplet(RequestSpawnDefaultChildApplet),
    }

    #[derive(Debug, Datable, Packet, Query)]
    #[ResponseType(PathResponse)]
    pub struct PathQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct PathResponse(pub TreeNodePath);

    #[derive(Debug, Datable, Packet, Query)]
    #[ResponseType(NameResponse)]
    pub struct NameQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct NameResponse(pub String);

    #[derive(Debug, Datable, Packet, Query)]
    #[ResponseType(SessionStorageResponse)]
    pub struct SessionStorageQuery;
    #[derive(Debug, Datable, Packet)]
    pub struct SessionStorageResponse(pub serde_json::Value);
}

pub mod file_packets {
    use crate::{
        datable::{ToData, TryFromData},
        packet::{PacketTrait, PacketTypeId, RequestPacketTrait, UniversalQueryTrait},
    };
    use singularity_macros::{Datable, Packet, Query, Request};
    use std::path::PathBuf;

    #[derive(Debug, Datable, Packet, Query)]
    #[ResponseType(ReadFileResponse)]
    pub struct ReadFileQuery(pub PathBuf);
    #[derive(Debug, Datable, Packet)]
    pub struct ReadFileResponse(pub Vec<u8>);

    #[derive(Debug, Datable, Packet, Request)]

    pub struct WriteFileRequest(pub PathBuf, pub Vec<u8>);
    impl WriteFileRequest {
        pub fn new(p: &impl AsRef<PathBuf>, s: &impl ToString) -> Self {
            Self(p.as_ref().clone(), s.to_string().bytes().collect())
        }
    }
}

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

#[derive(Debug, EventPacketUnion)]
pub enum StandardEvent {
    #[sub_union]
    DisplayEvent(DisplayEvent),
}
#[derive(Debug, RequestPacketUnion)]
pub enum StandardRequest {
    #[sub_union]
    DisplayRequest(DisplayRequest),
    WriteFileRequest(WriteFileRequest),
}
