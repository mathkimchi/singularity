#![cfg(test)]

use std::process::Command;

use crate::subapp_handler::{executable_subapp_handler::ExecutableSubappHandler, SubappHandler};

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler = ExecutableSubappHandler::from_executable_path(
        Command::new("python3").arg("../examples/placeholder_executable.py"),
    );

    // python's input waits until the user enters, which we simulate with `\n`
    dbg!(subapp_handler.inform_event(b"hello from the other process\n"));
    dbg!(subapp_handler.dump_requests());
}
