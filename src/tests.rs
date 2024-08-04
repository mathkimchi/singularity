#![cfg(test)]

use crate::demo_cli_frontend;

#[test]
fn hello() {
    println!("Hello from test!");
}

#[test]
fn run_demo_frontend() {
    demo_cli_frontend::run().unwrap();
}
