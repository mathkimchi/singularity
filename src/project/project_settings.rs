use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct SubappFileSystemPermissions {
    property: Option<SubappFileSystemPermission>,
}

#[derive(Serialize, Deserialize)]
pub struct SubappStandardSettings {
    subapp_file_system_permissions: Option<SubappFileSystemPermissions>,
}

#[derive(Serialize, Deserialize)]
pub struct SubappSettings {
    subapp_standard_settings: Option<SubappStandardSettings>,
    subapp_specific_settings: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectSettings {
    subapps: HashMap<String, SubappSettings>,
}
