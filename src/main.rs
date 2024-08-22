pub mod backend;
// pub mod demo_cli_frontend;
pub mod elements;
pub mod manager;
pub mod subapp;
mod tests;

fn main() -> Result<(), std::io::Error> {
    manager::Manager::run_demo()
}
