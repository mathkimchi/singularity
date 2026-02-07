use singularity_common::utils::tree::{
    tree_node_path::TreeNodePath,
    world_tree::{WorldTree, WorldTreePath},
};

fn assert_index_zero_does_nothing(world_tree: &WorldTree<impl PartialEq>) {
    assert!(world_tree.safe_get(WorldTreePath(&[])) == Some(world_tree));
}

#[test]
fn test_world_tree() {
    let a = WorldTree::Base("Hi");

    assert_eq!(a.get_root_value(), &"Hi");
    assert_index_zero_does_nothing(&a);
    assert!(
        a.safe_get(WorldTreePath(&[TreeNodePath(Vec::new())]))
            .is_none()
    );

    let b = WorldTree::new_world(a);

    assert_eq!(b.get_root_value(), &"Hi");
    assert_index_zero_does_nothing(&b);
    assert!(b.safe_get(WorldTreePath(&[TreeNodePath(Vec::new())])) == Some(&WorldTree::Base("Hi")));

    // TODO: test more things
}
