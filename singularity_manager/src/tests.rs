#![cfg(test)]

use crate::project_manager;
use singularity_common::project::Project;
use std::process::Command;

#[test]
fn print_runner_directory() {
    Command::new("ls").arg("-a").spawn().unwrap();
}

#[test]
fn run_demo() {
    // create demo manager
    let manager = project_manager::ProjectManager::new("../examples/root-project");

    // manager.tabs.add(
    //     TabHandler::new(
    //         singularity_common::components::timer_widget::TimerWidget::new_tab_creator((
    //             std::time::Duration::from_secs(10),
    //             false,
    //         )),
    //         Self::generate_tab_area(1, 1),
    //     ),
    //     &manager
    //         .tabs
    //         .get_id_by_org_path(&TreeNodePath::new_root())
    //         .unwrap(),
    // );

    // manager.tabs.add(
    //     TabHandler::new(
    //         TimeManager::new_tab_creator(),
    //         TabData {
    //             tab_type: "TASK_ORGANIZER".to_string(),
    //             session_data: serde_json::to_value("../examples/root-project").unwrap(),
    //         },
    //         singularity_ui::display_units::DisplayArea::new((0.5, 0.), (1.0, 1.)),
    //     ),
    //     &manager
    //         .tabs
    //         .get_id_by_org_path(&TreeNodePath::new_root())
    //         .unwrap(),
    // );

    manager.run().unwrap();
}

#[test]
fn project_parse() {
    Project::new("../examples/root-project");
}
