#![cfg(test)]

use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    demo_cli_frontend,
};

#[test]
fn hello() {
    println!("Hello from test!");
}

#[test]
fn tree_dbgs() {
    dbg!(RootedTree::new_test_tree_1(
        "A".to_string(),
        vec!["B".to_string(), "C".to_string(), "D".to_string()],
    ));

    let mut tree = RootedTree::from_root("A");

    dbg!(&tree);
    dbg!(tree.add_node("B", &TreeNodePath::from([])));
    dbg!(&tree);
    dbg!(tree.add_node("C", &TreeNodePath::from([])));
    dbg!(&tree);
    dbg!(tree.add_node("D", &TreeNodePath::from([])));
    dbg!(&tree);
    dbg!(tree.add_node("B!", &TreeNodePath::from([0])));
    dbg!(&tree);
    dbg!(tree.add_node("C?", &TreeNodePath::from([1])));
    dbg!(&tree);
    dbg!(tree.add_node("C?!", &TreeNodePath::from([1, 0])));
    dbg!(&tree);
    println!("Next is illegal on purpose:");
    dbg!(tree.add_node(">:( ", &TreeNodePath::from([1, 1])));
    dbg!(&tree);
}

#[test]
fn run_demo_frontend() {
    demo_cli_frontend::run().unwrap();
}
