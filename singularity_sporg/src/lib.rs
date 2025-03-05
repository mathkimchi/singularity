use project_settings::{ProjectSettings, SubappSettings, SubappStandardSettings, TabData};
use std::{collections::HashMap, path::PathBuf};

pub mod project_settings;
pub mod tile;

pub struct Project {
    project_directory: PathBuf,
    /// REVIEW: dangerous to expose this?
    pub project_settings: ProjectSettings,
}
impl Project {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path> + Clone,
        PathBuf: std::convert::From<P>,
    {
        Self::try_from_project_directory(project_directory.clone()).unwrap_or_else(|| Self {
            project_settings: ProjectSettings {
                subapps: HashMap::from_iter(vec![(
                    "file_manager".to_string(),
                    SubappSettings {
                        subapp_standard_settings: Some(SubappStandardSettings {
                            spawnable_default: Some(TabData::new_argless(
                                "./target/release/file_manager",
                                serde_json::to_value(
                                    project_directory.as_ref().to_str().unwrap().to_string(),
                                )
                                .unwrap(),
                            )),
                        }),
                        subapp_specific_settings: None,
                    },
                )]),
                open_tabs: None,
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
        let core_project_settings_path = project_directory.as_ref().join(".project/core.json");
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
        let core_project_settings_path = self.project_directory.join(".project/core.json");
        let serialized_project = serde_json::to_string_pretty(&self.project_settings).unwrap();
        std::fs::write(core_project_settings_path, serialized_project)
            .expect("failed to write serialized project to `.project/core.json`");
    }
}
