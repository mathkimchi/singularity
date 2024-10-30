use serde::{Deserialize, Serialize};
use singularity_ui::display_units::DisplayArea;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubappFileSystemPermission {
    location: String,
    /// default to false
    #[serde(default)]
    read: bool,
    /// default to false
    #[serde(default)]
    write: bool,
    /// default to false
    #[serde(default)]
    execute: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubappFileSystemPermissions {
    property: Option<SubappFileSystemPermission>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubappStandardSettings {
    subapp_file_system_permissions: Option<SubappFileSystemPermissions>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SubappSettings {
    subapp_standard_settings: Option<SubappStandardSettings>,
    subapp_specific_settings: Option<HashMap<String, serde_json::Value>>,
}

/// NOTE: Read devlog ~2024/10/29 for description; this is like SessionStorage for webdev
/// REVIEW: rename?
/// REVIEW: include Area and UIElement and TabType into this?
/// This type is kind of a black sheep
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TabData {
    pub tab_type: String,
    pub session_data: serde_json::Value,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenTab {
    // /// FIXME, right now, this works with finite tabs, is a glorified enum
    // pub tab_type: String,
    /// is kind of dangerous let user change the id of a tab, but if they screw this up, it is their fault
    pub tab_area: DisplayArea,
    pub tab_data: TabData,
}
/// REVIEW: alternative name for open tab: tab session
#[derive(Clone, Serialize, Deserialize)]
pub struct OpenTabs {
    pub tabs: std::collections::BTreeMap<Uuid, OpenTab>,

    /// ORGanizational tree
    pub org_tree: crate::utils::tree::uuid_tree::UuidTree,
    pub focused_tab: Uuid,

    /// currently, last in vec is "top" in gui
    pub display_order: Vec<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub subapps: HashMap<String, SubappSettings>,
    /// TODO: move this out of settings
    pub open_tabs: Option<OpenTabs>,
}
