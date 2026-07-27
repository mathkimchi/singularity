use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, ffi::OsString, hash::Hash};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AppletSpawnMethod {
    /// Like `Command`. (program, args). The command and args to spawn tab.
    PipeChildProcess {
        program: OsString,
        args: Vec<OsString>,
    },
    Dylib {
        path: OsString,
    },
}

/// All the data required for the SDE to spawn a new applet.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AppletSpawnData {
    pub applet_type_id: Option<AppletTypeId>,

    pub method: AppletSpawnMethod,

    /// NOTE: Read devlog ~2024/10/29 and 2025/02/19 for description; this is like SessionStorage for webdev
    /// REVIEW: rename?
    /// REVIEW: include Area and UIElement and TabType into this?
    /// This type is kind of a black sheep
    pub initial_session_storage: serde_json::Value,
}
impl AppletSpawnData {
    pub fn new_pipe_child_process(
        applet_type_id: Option<AppletTypeId>,
        applet_spawn_command: impl Into<OsString>,
        args: impl Iterator<Item = impl Into<OsString>>,
        initial_session_storage: serde_json::Value,
    ) -> Self {
        Self {
            applet_type_id,
            method: AppletSpawnMethod::PipeChildProcess {
                program: applet_spawn_command.into(),
                args: args.map(Into::into).collect(),
            },
            initial_session_storage,
        }
    }

    pub fn new_argless_pipe_child_process(
        applet_type_id: Option<AppletTypeId>,
        applet_spawn_command: impl Into<OsString>,
        initial_session_storage: serde_json::Value,
    ) -> Self {
        Self {
            applet_type_id,
            method: AppletSpawnMethod::PipeChildProcess {
                program: applet_spawn_command.into(),
                args: Vec::new(),
            },
            initial_session_storage,
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
impl Borrow<AppletTypeId> for AppletType {
    /// https://stackoverflow.com/questions/45384928/is-there-any-way-to-look-up-in-hashset-by-only-the-value-the-type-is-hashed-on
    fn borrow(&self) -> &AppletTypeId {
        &self.type_id
    }
}

/// Standard naming scheme is snake case: `file_manager`
/// REVIEW: make this a hash of the string instead of the string?
/// REVIEW: call this `AppletTypeIdName`?
///
/// NOTE: This is currently an enum because for pre-alpha,
/// all the applets are known.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum AppletTypeId {
    FileManager,
}
