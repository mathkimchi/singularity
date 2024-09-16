pub mod project_manager;
mod tests;

fn main() -> Result<(), std::io::Error> {
    project_manager::ProjectManager::run_demo()
}
