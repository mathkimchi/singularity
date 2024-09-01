#![cfg(test)]

use crate::subapp_handler::{ipc_subapp_handler::ExecutableSubappHandler, SubappHandler};
use std::process::Command;

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler =
        ExecutableSubappHandler::from_executable_path(&mut Command::new("../target/debug/demo"));

    dbg!(subapp_handler.dump_requests());
    // dbg!(subapp_handler.inform_event(b"hello from the other process\n"));
    // dbg!(subapp_handler.dump_requests());
}
