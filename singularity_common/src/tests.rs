#![cfg(test)]

use crate::subapp_handler::{executable_subapp_handler::ExecutableSubappHandler, SubappHandler};

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler = ExecutableSubappHandler::new();

    dbg!(subapp_handler.dump_requests());
}
