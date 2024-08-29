#![cfg(test)]

use crate::subapp_handler::{executable_subapp_handler::ExecutableSubappHandler, SubappHandler};

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler = ExecutableSubappHandler::from_executable_path("ls");

    dbg!(subapp_handler.dump_requests());
}
