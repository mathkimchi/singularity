pub mod demo;
pub mod editor;
pub mod file_manager;
pub mod task_organizer;

/// FIXME
pub fn get_tab_creator_from_type(tab_type: &str) -> Box<dyn singularity_common::tab::TabCreator> {
    use singularity_common::tab::BasicTab;
    match tab_type {
        "EDITOR" => Box::new(editor::Editor::new_tab_creator()),
        "FILE_MANAGER" => Box::new(file_manager::FileManager::new_tab_creator()),
        "TASK_ORGANIZER" => Box::new(task_organizer::TaskOrganizer::new_tab_creator()),
        // "DEMO" => Box::new(demo::Test::new_tab_creator(
        //     serde_json::from_value::<String>(tab_data).unwrap(),
        // )),
        _ => panic!(),
    }
}
