use serde::{Deserialize, Serialize};
use singularity_common::utils::{
    id_map::{Id, IdMap},
    tree::id_tree::IdTree,
};
use singularity_ui::display_units::DisplayArea;
use std::{collections::HashMap, ffi::OsString};

use crate::tile::Tiles;

// #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
// pub struct SubappFileSystemPermission {
//     location: String,
//     /// default to false
//     #[serde(default)]
//     read: bool,
//     /// default to false
//     #[serde(default)]
//     write: bool,
//     /// default to false
//     #[serde(default)]
//     execute: bool,
// }

// #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
// pub struct SubappFileSystemPermissions {
//     property: Option<SubappFileSystemPermission>,
// }

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SubappStandardSettings {
    pub spawnable_default: Option<TabData>,
}

/// This is for a tab type as opposed to a specific instance of a tab
/// TODO: rename
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SubappSettings {
    pub subapp_standard_settings: Option<SubappStandardSettings>,
    pub subapp_specific_settings: Option<HashMap<String, serde_json::Value>>,
}

/// Like `Command`. (program, args). The command and args to spawn tab.
/// TODO: can make this an Enum later when tabs can have different ways of being created.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TabSpawnCommand {
    pub program: OsString,
    pub args: Vec<OsString>,
}

/// NOTE: Read devlog ~2024/10/29 and 2025/02/19 for description; this is like SessionStorage for webdev
/// REVIEW: rename?
/// REVIEW: include Area and UIElement and TabType into this?
/// This type is kind of a black sheep
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TabData {
    pub tab_command: TabSpawnCommand,
    /// REVIEW: make this another type?
    pub session_data: serde_json::Value,
}
impl TabData {
    pub fn new(
        tab_command_program: impl Into<OsString>,
        args: impl Iterator<Item = impl Into<OsString>>,
        session_data: serde_json::Value,
    ) -> Self {
        Self {
            tab_command: TabSpawnCommand {
                program: tab_command_program.into(),
                args: args.map(|arg| arg.into()).collect(),
            },
            session_data,
        }
    }

    pub fn new_argless(
        tab_command_program: impl Into<OsString>,
        session_data: serde_json::Value,
    ) -> Self {
        Self {
            tab_command: TabSpawnCommand {
                program: tab_command_program.into(),
                args: Vec::new(),
            },
            session_data,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OpenTab {
    /// is kind of dangerous to let user change the id of a tab, but if they screw this up, it is their fault
    pub tab_area: DisplayArea,
    pub tab_data: TabData,
}
/// REVIEW: alternative name for open tab: tab session
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OpenTabs {
    pub tabs: IdMap<OpenTab>,

    /// ORGanizational tree
    pub org_tree: IdTree<OpenTab>,
    pub focused_tab: Id<OpenTab>,

    // /// currently, last in vec is "top" in gui
    // pub display_order: Vec<Uuid>,
    pub display_tiles: Tiles<OpenTab>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    /// this is the list of tab types
    /// REVIEW: rename
    pub subapps: HashMap<String, SubappSettings>,
    /// TODO: move this out of settings
    pub open_tabs: Option<OpenTabs>,
}
