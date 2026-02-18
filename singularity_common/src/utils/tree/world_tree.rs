use crate::utils::tree::{recursive_tree::RecursiveTreeNode, tree_node_path::TreeNodePath};

// pub struct WorldTreePath(Vec<TreeNodePath>);
pub struct WorldTreePath<'a>(pub &'a [TreeNodePath]);

/// A tree whose values are other trees or the base value.
#[derive(PartialEq, Eq, Clone)]
pub enum WorldTree<T> {
    Base(T),
    World(Box<RecursiveTreeNode<WorldTree<T>>>),
}
impl<T> WorldTree<T> {
    pub fn new_base(val: T) -> Self {
        Self::Base(val)
    }

    /// Given an inner, returns a new world tree that has just the root and the root's value is inner.
    pub fn new_world(inner: Self) -> Self {
        Self::World(Box::new(RecursiveTreeNode::from_value(inner)))
    }

    /// Returns the value if base, otherwise, gets the root's value
    pub fn get_root_value(&self) -> &T {
        match self {
            WorldTree::Base(value) => value,
            WorldTree::World(recursive_tree_node) => {
                recursive_tree_node.get_value().get_root_value()
            }
        }
    }

    pub fn safe_get(&self, path: WorldTreePath) -> Option<&Self> {
        match path.0.first() {
            None => {
                // Base case:
                Some(self)
            }
            Some(tree_path) => match self {
                // If we are at the base, then we can't further index a tree path
                WorldTree::Base(_) => None,
                WorldTree::World(recursive_tree_node) => recursive_tree_node
                    .safe_get(tree_path)?
                    .get_value()
                    .safe_get(WorldTreePath(&path.0[1..])),
            },
        }
    }
}
