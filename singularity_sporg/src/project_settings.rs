use crate::applet_data::{AppletSpawnData, AppletType, AppletTypeId};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    /// this is the list of tab types
    /// REVIEW: rename
    pub applet_types: HashSet<AppletType>,
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
        PathBuf: From<P>,
    {
        Self::try_from_project_directory(project_directory.clone()).unwrap_or_else(|| Self {
            project_settings: ProjectSettings {
                applet_types: HashSet::from_iter(vec![AppletType {
                    type_id: AppletTypeId::FileManager,
                    default_spawn: Some(AppletSpawnData::new_argless_pipe_child_process(
                        Some(AppletTypeId::FileManager),
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
        PathBuf: From<P>,
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

    #[must_use]
    pub const fn get_project_directory(&self) -> &PathBuf {
        &self.project_directory
    }

    #[must_use]
    pub const fn get_project_settings(&self) -> &ProjectSettings {
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
