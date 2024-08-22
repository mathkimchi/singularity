use std::ops::{Index, IndexMut};

use super::{rooted_tree::RootedTree, tree_node_path::TraversableTree};
use crate::backend::utils::tree_node_path::TreeNodePath;
use serde::{Deserialize, Serialize};

/// This and `RootedTree` store the same information,
/// but this is the conceptual version of it.
///
/// This should be safe (all representations are valid)
/// but rooted tree is "optimized" (at least I tried to).
/// A recursive tree node only stores information downwards.
/// In other words, it does not know where it is in
/// relation to the root and what its parent is.
/// For some operations, it acts as if it was the root.
///
/// **This should only be used for
/// intermediate representations** between some other type
/// and rooted tree.
/// For representing trees that will be used for a while,
/// use rooted tree.
///
/// REVIEW: right now, I purposefully *didn't* implement
/// useful functions for recursive tree node, to ensure
/// it is converted to rooted tree, but idk if this
/// is a good idea.
///
/// NOTE: Idk, how serialize and deserialize can be derived
/// if T isn't guranteed to be either, but I am not complaining.
#[derive(Serialize, Deserialize)]
pub struct RecursiveTreeNode<T> {
    value: T,
    children: Vec<RecursiveTreeNode<T>>,
}
impl<T> RecursiveTreeNode<T> {
    pub fn from_value(value: T) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }

    /// add child as last
    pub fn push_child_node(&mut self, child_node: Self) {
        self.children.push(child_node);
    }

    pub fn safe_get(&self, path: &TreeNodePath) -> Option<&Self> {
        let mut node = self;

        for index in path.0.iter() {
            node = node.children.get(*index)?;
        }

        Some(node)
    }
    pub fn safe_get_mut(&mut self, path: &TreeNodePath) -> Option<&mut Self> {
        let mut node = self;

        for index in path.0.iter() {
            node = node.children.get_mut(*index)?;
        }

        Some(node)
    }
}
impl<T> From<RecursiveTreeNode<T>> for RootedTree<T> {
    fn from(recursive_tree_node: RecursiveTreeNode<T>) -> Self {
        let mut rooted_tree = RootedTree::from_root(recursive_tree_node.value);

        let mut unvisited_children = vec![(TreeNodePath::new_root(), recursive_tree_node.children)];

        while !unvisited_children.is_empty() {
            let mut new_unvisited_children = Vec::new();

            for (parent_path, children) in unvisited_children {
                for child in children {
                    let child_path = rooted_tree.add_node(child.value, &parent_path).unwrap();
                    new_unvisited_children.push((child_path, child.children));
                }
            }

            unvisited_children = new_unvisited_children;
        }

        rooted_tree
    }
}

impl<T> Index<&TreeNodePath> for RecursiveTreeNode<T> {
    type Output = T;

    fn index(&self, path: &TreeNodePath) -> &Self::Output {
        &self.safe_get(path).unwrap().value
    }
}
impl<T> IndexMut<&TreeNodePath> for RecursiveTreeNode<T> {
    fn index_mut(&mut self, path: &TreeNodePath) -> &mut Self::Output {
        &mut self.safe_get_mut(path).unwrap().value
    }
}

impl<T> TraversableTree for RecursiveTreeNode<T> {
    fn exists_at(&self, path: &TreeNodePath) -> bool {
        self.safe_get(path).is_some()
    }
}
