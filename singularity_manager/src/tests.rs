#![cfg(test)]

use singularity_common::project::Project;

use crate::project_manager;

#[test]
fn hello() {
    println!("Hello from test!");
}

#[test]
fn run_demo() {
    // create demo manager
    let manager = project_manager::ProjectManager::new("examples/root-project");

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

    manager.run().unwrap();
}

#[test]
fn project_parse() {
    Project::new("../examples/root-project");
}
