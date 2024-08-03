//! NOTE: I wish it could be safe, but I don't think it will be.
//! Invalid states are representable, and removing nodes might mess up everything.
//! What this means that it is very easy for me to make an error in writing this.
//! However, if I don't screw up, then it should be safe from the outside.
//! The safest representation is to have a recursive struct that takes ownership of children, and stores only its own item and a vec of children.
//!
//! Terminology:
//! - The `index` of a node refers to the usize index of the node in the vector of flattened nodes.
//! - The `path` also corresponds to a node in a tree, but it specifies it in the context of the tree's structure.
//! - The index and path both refer to the same thing, but in different ways. Outside of this file, the index should be inaccessible.
//!
//! REVIEW: not sure if the path should correspond to a tree or not. I am leaning towards having it independent.
//!
//! NOTE: Currently, all trees are immutable once made.

#[macro_export]
macro_rules! rooted_tree {
    [$tree_item:expr] => {
        $crate::backend::utils::RootedTree::from_root($tree_item)
    };

    [$tree_item:expr => $children:expr] => {
        $crate::backend::utils::RootedTree::from_root($tree_item)
    };
}

struct Node<T> {
    item: T,
    // even this on its own can be contradictory
    children_flat_indices: Vec<usize>,

    // everything other than item and children are redundant
    flat_index: usize,
    path: TreeNodePath,
    parent_flat_index: Option<usize>,
}

/// The rooted tree has exactly one root
pub struct RootedTree<T> {
    /// REVIEW: put this in some kind of order?
    flattened_nodes: Vec<Node<T>>,
    root_flat_index: usize,
}
impl<T> RootedTree<T> {
    pub fn from_root(root_item: T) -> Self {
        Self {
            flattened_nodes: vec![Node {
                item: root_item,
                children_flat_indices: Vec::new(),
                flat_index: 0,
                path: TreeNodePath::new_root(),
                parent_flat_index: None,
            }],
            root_flat_index: 0,
        }
    }

    pub fn new_test_tree(root_item: T, child_item: T) -> Self {
        Self {
            flattened_nodes: vec![
                Node {
                    item: root_item,
                    children_flat_indices: vec![1],
                    flat_index: 0,
                    path: TreeNodePath::new_root(),
                    parent_flat_index: None,
                },
                Node {
                    item: child_item,
                    children_flat_indices: vec![0],
                    flat_index: 1,
                    path: TreeNodePath(vec![0]),
                    parent_flat_index: Some(1),
                },
            ],
            root_flat_index: 0,
        }
    }

    // pub fn push_leaf() {}

    // pub fn swap_items() {
    //     todo!()
    // }

    pub fn iter_paths_dfs(&self) -> DfsPathsIterator {
        todo!()
    }

    fn get_node_flat_index(&self, tree_node_path: &TreeNodePath) -> Option<usize> {
        let path_vec = tree_node_path.0.clone();
        let mut current_flat_index = self.root_flat_index;

        for child_number in path_vec {
            let current_node = &self.flattened_nodes[current_flat_index];

            let next_flat_index = current_node.children_flat_indices.get(child_number);

            if let Some(next_flat_index) = next_flat_index {
                current_flat_index = *next_flat_index;
            } else {
                return None;
            }
        }

        Some(current_flat_index)
    }
}

/// dfs postorder
pub struct DfsPathsIterator {}
impl Iterator for DfsPathsIterator {
    type Item = TreeNodePath;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// Like pointers but has the context of the tree structure
/// Also like a file path
///
/// NOTE: Most functions for a path are better thought of as functions for the Node that the path refers to
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TreeNodePath(Vec<usize>);
impl TreeNodePath {
    /// REVIEW should this function be a static fn for path or a rooted tree's obj method?
    pub fn new_root() -> Self {
        Self(Vec::new())
    }

    // For the traverse functions, some require the original tree to be safe

    pub fn traverse_to_parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            // the parent path is this path without the last element
            let mut parent_path_vec = self.0.clone();
            parent_path_vec.pop();
            Some(Self(parent_path_vec))
        }
    }
    /// Needs the rooted tree to make sure that the child exists
    pub fn traverse_to_child<T>(
        &self,
        rooted_tree: &RootedTree<T>,
        child_index: usize,
    ) -> Option<Self> {
        let child_path = {
            let mut child_path_vec = self.0.clone();

            child_path_vec.push(child_index);

            Self(child_path_vec)
        };

        if rooted_tree.get_node_flat_index(&child_path).is_some() {
            Some(child_path)
        } else {
            None
        }
    }
    /// Needs the rooted tree to make sure that the child exists
    pub fn traverse_to_first_child<T>(&self, rooted_tree: &RootedTree<T>) -> Option<Self> {
        self.traverse_to_child(rooted_tree, 0)
    }
    pub fn traverse_to_previous_sibling(&self) -> Option<Self> {
        todo!()
    }
    pub fn traverse_to_next_sibling(&self) -> Option<Self> {
        todo!()
    }

    /// root has depth=0
    pub fn depth(&self) -> usize {
        todo!()
    }
}

/// REVIEW make this output option?
impl<T> std::ops::Index<&TreeNodePath> for RootedTree<T> {
    type Output = T;

    fn index(&self, _path: &TreeNodePath) -> &Self::Output {
        todo!()
    }
}
impl<T> std::ops::IndexMut<&TreeNodePath> for RootedTree<T> {
    fn index_mut(&mut self, _path: &TreeNodePath) -> &mut Self::Output {
        todo!()
    }
}
