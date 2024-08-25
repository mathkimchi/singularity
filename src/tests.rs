#![cfg(test)]

use crate::project::Project;

#[test]
fn hello() {
    println!("Hello from test!");
}

#[test]
fn run_demo() {
    crate::project_manager::ProjectManager::run_demo().unwrap();
}

#[test]
fn project_parse() {
    Project::new("examples/root-project");
}
