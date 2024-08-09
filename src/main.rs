pub mod backend;
pub mod demo_cli_frontend;
pub mod subapp;
mod tests;

fn main() -> Result<(), std::io::Error> {
    demo_cli_frontend::run()
}
