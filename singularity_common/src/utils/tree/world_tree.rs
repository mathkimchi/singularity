use crate::utils::tree::{recursive_tree::RecursiveTreeNode, tree_node_path::TreeNodePath};

pub mod world_tree_traversal;

#[derive(Debug, Clone)]
pub struct WorldTreePath(pub Box<[TreeNodePath]>);
impl WorldTreePath {
    /// getting by this will return the same thing
    #[must_use]
    pub fn new_empty() -> Self {
        Self(Box::new([]))
    }

    /// Goes one level `into`
    #[must_use]
    pub fn new_into() -> Self {
        Self(Box::new([TreeNodePath::new_root()]))
    }
}

/// A tree whose values are other trees or the base value.
#[derive(PartialEq, Eq, Clone)]
pub enum WorldTree<T> {
    Base(T),
    World(Box<RecursiveTreeNode<Self>>),
}
impl<T> WorldTree<T> {
    pub const fn new_base(val: T) -> Self {
        Self::Base(val)
    }

    /// Given an inner, returns a new world tree that has just the root and the root's value is inner.
    pub fn new_world(inner: Self) -> Self {
        Self::World(Box::new(RecursiveTreeNode::from_value(inner)))
    }

    /// Returns the value if base, otherwise, gets the root's value
    pub fn get_root_value(&self) -> &T {
        match self {
            Self::Base(value) => value,
            Self::World(recursive_tree_node) => recursive_tree_node.get_value().get_root_value(),
        }
    }

    pub fn safe_get(&self, path: &WorldTreePath) -> Option<&Self> {
        match path.0.first() {
            None => {
                // Base case:
                Some(self)
            }
            Some(tree_path) => match self {
                // If we are at the base, then we can't further index a tree path
                Self::Base(_) => None,
                Self::World(recursive_tree_node) => recursive_tree_node
                    .safe_get(tree_path)?
                    .get_value()
                    .safe_get(&WorldTreePath(path.0[1..].into())),
            },
        }
    }
}
impl WorldTree<String> {
    #[must_use]
    pub fn outer_world_to_string(&self, focus_path: Option<TreeNodePath>) -> String {
        match self {
            Self::Base(inner) => {
                if focus_path.is_some() {
                    format!(">{inner}")
                } else {
                    inner.clone()
                }
            }
            Self::World(recursive_tree_node) => recursive_tree_node
                .simple_to_string(&|node| node.get_root_value().clone(), focus_path),
        }
    }
}
