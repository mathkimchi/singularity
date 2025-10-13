pub mod packets;
pub mod project_manager;
pub mod tab;

/// this should be run from cli
fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();

    // let project_manager = project_manager::ProjectManager::new(
    //     args.get(1)
    //         .unwrap_or(&"examples/root-project".to_string())
    //         .clone(),
    // );

    // project_manager.run()

    Ok(())
}
