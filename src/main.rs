pub mod backend;
pub mod block;
pub mod demo_cli_frontend;
mod tests;

fn main() -> Result<(), std::io::Error> {
    demo_cli_frontend::run()
}
