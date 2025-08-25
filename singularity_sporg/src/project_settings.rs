use crate::applet_data::{AppletSpawnData, AppletType, AppletTypeId};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

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

// #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
// pub struct SubappStandardSettings {
//     pub spawnable_default: Option<AppletSpawnData>,
// }

// /// This is for a tab type as opposed to a specific instance of a tab
// /// TODO: rename
// #[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
// pub struct SubappSettings {
//     pub subapp_standard_settings: Option<SubappStandardSettings>,
//     pub subapp_specific_settings: Option<HashMap<String, serde_json::Value>>,
// }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    /// this is the list of tab types
    /// REVIEW: rename
    pub applet_types: HashSet<AppletType>,
}
impl ProjectSettings {
    // pub fn get_applet(&self, applet: AppletTypeId) -> A {}
}

pub struct Project {
    project_directory: PathBuf,
    /// REVIEW: dangerous to expose this?
    pub project_settings: ProjectSettings,
}
impl Project {
    pub fn open_or_make<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path> + Clone,
        PathBuf: std::convert::From<P>,
    {
        Self::try_from_project_directory(project_directory.clone()).unwrap_or_else(|| Self {
            project_settings: ProjectSettings {
                applet_types: HashSet::from_iter(vec![AppletType {
                    type_id: AppletTypeId::new("file_manager"),
                    default_spawn: Some(AppletSpawnData::new_argless_pipe_child_process(
                        Some(AppletTypeId::new("file_manager")),
                        "./target/release/file_manager",
                        serde_json::to_value(PathBuf::from(project_directory.clone())).unwrap(),
                    )),
                }]),
            },
            project_directory: PathBuf::from(project_directory),
        })
    }

    pub fn try_from_project_directory<P>(project_directory: P) -> Option<Self>
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Some(Self {
            project_settings: Self::parse_project_settings(&project_directory)?,
            project_directory: PathBuf::from(project_directory),
        })
    }

    fn parse_project_settings(
        project_directory: impl AsRef<std::path::Path>,
    ) -> Option<ProjectSettings> {
        let core_project_settings_path = project_directory
            .as_ref()
            .join(".project/project_settings.json");
        Some(
            serde_json::from_str(&std::fs::read_to_string(&core_project_settings_path).ok()?)
                .expect("core project file should be formatted correctly"),
        )
    }

    pub fn get_project_directory(&self) -> &PathBuf {
        &self.project_directory
    }

    pub fn get_project_settings(&self) -> &ProjectSettings {
        &self.project_settings
    }

    pub fn save_to_file(&self) {
        let core_project_settings_path = self
            .project_directory
            .join(".project/project_settings.json");
        let serialized_project = serde_json::to_string_pretty(&self.project_settings).unwrap();
        std::fs::write(core_project_settings_path, serialized_project)
            .expect("failed to write serialized project to `.project/project_settings.json`");
    }
}
