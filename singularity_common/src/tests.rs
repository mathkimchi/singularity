#![cfg(test)]

use std::process::Command;

use crate::subapp_handler::{executable_subapp_handler::ExecutableSubappHandler, SubappHandler};

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler = ExecutableSubappHandler::from_executable_path(
        Command::new("python3").arg("../examples/placeholder_executable.py"),
    );

    dbg!(subapp_handler.dump_requests());
}
