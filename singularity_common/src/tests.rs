#![cfg(test)]

use crate::subapp::dynamic_loader::load_subapp;

#[test]
fn dylib_test() {
    dbg!("Hi0");
    let mut subapp = load_subapp("../target/release/libdylib_demo.so");

    dbg!("Hi1");

    // dbg!(subapp.subapp_interface.dump_requests());
}

#[test]
fn executable_subapp_handler_test() {
    // let mut subapp_handler =
    //     UnixSocketSubappInterface::from_executable_path(&mut Command::new("../target/debug/demo"));

    // dbg!(subapp_handler.dump_requests());
    // dbg!(subapp_handler.inform_event(b"hello from the other process\n"));
    // dbg!(subapp_handler.dump_requests());
}
