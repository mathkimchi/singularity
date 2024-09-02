#![cfg(test)]

use crate::subapp::unix_socket_subapp_interface::UnixSocketSubappInterface;
use std::process::Command;

#[test]
fn executable_subapp_handler_test() {
    let mut subapp_handler =
        UnixSocketSubappInterface::from_executable_path(&mut Command::new("../target/debug/demo"));

    // dbg!(subapp_handler.dump_requests());
    // dbg!(subapp_handler.inform_event(b"hello from the other process\n"));
    // dbg!(subapp_handler.dump_requests());
}
