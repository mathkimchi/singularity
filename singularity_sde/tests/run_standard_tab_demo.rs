use std::process::Command;

#[test]
fn run_standard_tab_demo() {
    println!("Will run editor");
    Command::new("cargo")
        .args([
            "run",
            "--package",
            "singularity_standard_tabs",
            "--bin",
            "editor",
        ])
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    println!("Ran editor");
}
