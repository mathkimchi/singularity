use project_settings::ProjectSettings;
use std::path::PathBuf;

pub mod project_settings;

pub struct Project {
    _project_directory: PathBuf,
    _project_settings: ProjectSettings,
}
impl Project {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Self {
            _project_settings: Self::get_project_settings(&project_directory),
            _project_directory: PathBuf::from(project_directory),
        }
    }

    fn get_project_settings<P>(project_directory: P) -> ProjectSettings
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
}
