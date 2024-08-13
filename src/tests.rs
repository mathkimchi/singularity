#![cfg(test)]

#[test]
fn hello() {
    println!("Hello from test!");
}

#[test]
fn run_demo() {
    crate::manager::Manager::run_demo().unwrap();
}
