mod demo_from_winit;
pub mod project_manager;
mod tests;

fn main() -> Result<(), std::io::Error> {
    // project_manager::ProjectManager::run_demo()

    demo_from_winit::run();

    Ok(())
}
