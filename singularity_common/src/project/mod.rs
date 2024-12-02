use project_settings::ProjectSettings;
use std::path::PathBuf;

pub mod project_settings;

pub struct Project {
    project_directory: PathBuf,
    /// REVIEW: dangerous to expose this?
    pub project_settings: ProjectSettings,
}
impl Project {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Self {
            project_settings: Self::parse_project_settings(&project_directory),
            project_directory: PathBuf::from(project_directory),
        }
    }

    fn parse_project_settings<P>(project_directory: P) -> ProjectSettings
    where
        P: AsRef<std::path::Path>,
    {
        let core_project_settings_path = project_directory.as_ref().join(".project/core.json");
        serde_json::from_str(
            &std::fs::read_to_string(&core_project_settings_path).expect(
                "project directories should have a core project file in `.project/core.json`",
            ),
        )
        .expect("core project file should be formatted correctly")
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
