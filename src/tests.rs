#![cfg(test)]

use crate::demo_cli_frontend;

#[test]
fn hello() {
    println!("Hello from test!");
}

// #[test]
// fn tree_macro_test() {
//     dbg!(tree!(1));

//     dbg!(tree![1 => [tree![2 => [tree![3]]], tree![10=>[]]]]);
// }

// #[test]
// #[allow(clippy::useless_vec)]
// fn tree_index_test() {
//     let root = tree![1 => [tree![2 => [tree![3]]], tree![10=>[]]]];

//     dbg!(&root);
//     dbg!(&root[&[]]);
//     dbg!(&root[&[0]]);
//     dbg!(&root[&[1]]);

//     // should be same
//     dbg!(&root[&[0, 0]]);
//     dbg!(&root[&[0]][&[0]]);
//     dbg!(&root[&vec![0, 0]]);

//     dbg!(&root.flattened_ref());
// }

#[test]
fn run_demo_frontend() {
    demo_cli_frontend::run().unwrap();
}
