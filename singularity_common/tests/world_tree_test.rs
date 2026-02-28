use singularity_common::utils::tree::{
    recursive_tree::RecursiveTreeNode,
    tree_node_path::TreeNodePath,
    world_tree::{WorldTree, WorldTreePath},
};

fn assert_index_zero_does_nothing(world_tree: &WorldTree<impl PartialEq>) {
    assert!(world_tree.safe_get(WorldTreePath([].into())) == Some(world_tree));
}

#[test]
fn test_world_tree() {
    let a = WorldTree::Base("Hi");

    assert_eq!(a.get_root_value(), &"Hi");
    assert_index_zero_does_nothing(&a);
    assert!(
        a.safe_get(WorldTreePath([TreeNodePath(Vec::new())].into()))
            .is_none()
    );

    let b = WorldTree::new_world(a);

    assert_eq!(b.get_root_value(), &"Hi");
    assert_index_zero_does_nothing(&b);
    assert!(
        b.safe_get(WorldTreePath([TreeNodePath(Vec::new())].into()))
            == Some(&WorldTree::Base("Hi"))
    );

    // TODO: test more things
}

#[test]
fn test_printing() {
    /*
     * From the DEVLOG 2026-02-19 10:56AM:
     *
     *
     * // Just a value (level 0)
     * ( [] )
     *
     * // Structure of a normal tree (level 1)
     * ( [[]] )
     * |
     * |--- ( [[0]] )
     * |
     * |--- ( [[1]] )
     * |
     * |--- ( [[2]] )
     *          |
     *          |--- ( [[2, 0]] )
     *          |
     *          |--- ( [[2, 1]] )
     *
     * // Level 2: tree in a tree
     * // Now imagine that the above tree is still the outermost tree,
     * // but replace ([[2, 1]]) with the following tree:
     * ...
     * ( [[2, 1], []] )
     * |
     * |--- ( [[2, 1], [0]] )
     * |           |
     * |           |--- ( [[2, 1], [0, 0]] )
     * |           |
     * |           |--- ( [[2, 1], [0, 1]] )
     * |
     * |--- ( [[2, 1], [1]] )
     *             |
     *             |--- ( [[2, 1], [1, 0]] )
     *             |
     *             |--- ( [[2, 1], [1, 1]] )
     */
    let level_0 = WorldTree::Base("([])".to_string());
    println!("Level 0:");
    println!("{}", level_0.outer_world_to_string(None));

    let level_1 = WorldTree::World(Box::new(RecursiveTreeNode::new(
        WorldTree::Base("([[]])".to_string()),
        vec![
            RecursiveTreeNode::from_value(WorldTree::Base("([[0]])".to_string())),
            RecursiveTreeNode::from_value(WorldTree::Base("([[1]])".to_string())),
            RecursiveTreeNode::new(
                WorldTree::Base("([[2]])".to_string()),
                vec![
                    RecursiveTreeNode::from_value(WorldTree::Base("([[2, 0]])".to_string())),
                    RecursiveTreeNode::from_value(WorldTree::Base("([[2, 1]])".to_string())),
                ],
            ),
            RecursiveTreeNode::new(
                WorldTree::Base("([[3]])".to_string()),
                vec![
                    RecursiveTreeNode::from_value(WorldTree::Base("([[3, 0]])".to_string())),
                    RecursiveTreeNode::from_value(WorldTree::Base("([[3, 1]])".to_string())),
                ],
            ),
        ],
    )));
    println!("Level 1:");
    println!("{}", level_1.outer_world_to_string(None));

    let level_2 = WorldTree::World(Box::new(RecursiveTreeNode::new(
        WorldTree::Base("([[]])".to_string()),
        vec![
            RecursiveTreeNode::from_value(WorldTree::Base("([[0]])".to_string())),
            RecursiveTreeNode::from_value(WorldTree::Base("([[1]])".to_string())),
            RecursiveTreeNode::new(
                WorldTree::Base("([[2]])".to_string()),
                vec![
                    RecursiveTreeNode::from_value(WorldTree::Base("([[2, 0]])".to_string())),
                    RecursiveTreeNode::from_value(WorldTree::World(Box::new(
                        RecursiveTreeNode::new(
                            WorldTree::Base("([[2, 1], []])".to_string()),
                            vec![
                                RecursiveTreeNode::new(
                                    WorldTree::Base("([[2, 1], [0]])".to_string()),
                                    vec![
                                        RecursiveTreeNode::from_value(WorldTree::Base(
                                            "([[2, 1], [0, 0]])".to_string(),
                                        )),
                                        RecursiveTreeNode::from_value(WorldTree::Base(
                                            "([[2, 1], [0, 1]])".to_string(),
                                        )),
                                    ],
                                ),
                                RecursiveTreeNode::new(
                                    WorldTree::Base("([[2, 1], [1]])".to_string()),
                                    vec![
                                        RecursiveTreeNode::from_value(WorldTree::Base(
                                            "([[2, 1], [1, 0]])".to_string(),
                                        )),
                                        RecursiveTreeNode::from_value(WorldTree::Base(
                                            "([[2, 1], [1, 1]])".to_string(),
                                        )),
                                    ],
                                ),
                            ],
                        ),
                    ))),
                ],
            ),
            RecursiveTreeNode::new(
                WorldTree::Base("([[3]])".to_string()),
                vec![
                    RecursiveTreeNode::from_value(WorldTree::Base("([[3, 0]])".to_string())),
                    RecursiveTreeNode::from_value(WorldTree::Base("([[3, 1]])".to_string())),
                ],
            ),
        ],
    )));
    println!("Level 2 outer:");
    println!("{}", level_2.outer_world_to_string(None));
    println!("Level 2 indexed:");
    println!(
        "{}",
        level_2
            .safe_get(WorldTreePath(Box::new([TreeNodePath(vec![2, 1])])))
            .unwrap()
            .outer_world_to_string(None)
    );
}
