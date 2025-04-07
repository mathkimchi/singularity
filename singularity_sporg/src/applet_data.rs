use serde::{Deserialize, Serialize};
use std::{ffi::OsString, hash::Hash};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AppletSpawnMethod {
    /// Like `Command`. (program, args). The command and args to spawn tab.
    PipeChildProcess {
        program: OsString,
        args: Vec<OsString>,
    },
}

/// All the data required for the SDE to spawn a new applet.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppletSpawnData {
    pub applet_type_id: AppletTypeId,

    pub method: AppletSpawnMethod,

    /// NOTE: Read devlog ~2024/10/29 and 2025/02/19 for description; this is like SessionStorage for webdev
    /// REVIEW: rename?
    /// REVIEW: include Area and UIElement and TabType into this?
    /// This type is kind of a black sheep
    pub initial_session_data: serde_json::Value,
}
impl AppletSpawnData {
    pub fn new(
        applet_type_id: AppletTypeId,
        applet_spawn_command: impl Into<OsString>,
        args: impl Iterator<Item = impl Into<OsString>>,
        initial_session_data: serde_json::Value,
    ) -> Self {
        Self {
            applet_type_id,
            method: AppletSpawnMethod::PipeChildProcess {
                program: applet_spawn_command.into(),
                args: args.map(|arg| arg.into()).collect(),
            },
            initial_session_data,
        }
    }

    pub fn new_argless(
        applet_type_id: AppletTypeId,
        applet_spawn_command: impl Into<OsString>,
        initial_session_data: serde_json::Value,
    ) -> Self {
        Self {
            applet_type_id,
            method: AppletSpawnMethod::PipeChildProcess {
                program: applet_spawn_command.into(),
                args: Vec::new(),
            },
            initial_session_data,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppletType {
    pub type_id: AppletTypeId,
    pub default_spawn: Option<AppletSpawnData>,
}
impl Hash for AppletType {
    /// Same as the `type id`'s hash.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}
impl Eq for AppletType {}

/// Standard naming scheme is snake case: `file_manager`
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct AppletTypeId(pub String);
impl AppletTypeId {
    pub fn new(s: impl ToString) -> Self {
        Self(s.to_string())
    }
}
