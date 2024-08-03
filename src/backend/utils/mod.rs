//! NOTE: I wish it could be safe, but I don't think it will be.
//! Invalid states are representable, and removing nodes might mess up everything.
//! What this means that it is very easy for me to make an error in writing this.
//! However, if I don't screw up, then it should be safe from the outside.
//! The safest representation is to have a recursive struct that takes ownership of children, and stores only its own item and a vec of children.
//!
//! Terminology:
//! - The `index` of a node refers to the usize index of the node in the vector of flattened nodes.
//! - The `path` also corresponds to a node in a tree, but it specifies it in the context of the tree's structure.
//! - The `child number` of a child relative to its parent is the index that it appears in the parent's children indices.
//! - The index and path both refer to the same thing, but in different ways; the path is stored as a bunch of child numbers. Outside of this file, the index should be inaccessible.
//!
//! REVIEW: not sure if the path should correspond to a tree or not. I am leaning towards having it independent.
//!
//! CHECK: handle mutability of structure.

use std::fmt::Debug;

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
    // even this on its own can be contradictory (eg duplicates)
    children_flat_indices: Vec<usize>,

    // NOTE everything other than item and children are redundant
    // REVIEW: decide which are redundant
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

    /// FIXME
    pub fn new_test_tree_0(root_item: T, child_item: T) -> Self {
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
                    children_flat_indices: vec![],
                    flat_index: 1,
                    path: TreeNodePath(vec![0]),
                    parent_flat_index: Some(0),
                },
            ],
            root_flat_index: 0,
        }
    }
    /// FIXME
    pub fn new_test_tree_1(root_item: T, first_depth_items: Vec<T>) -> Self {
        let root_node = Node {
            item: root_item,
            children_flat_indices: (1..(first_depth_items.len() + 1)).collect(),
            flat_index: 0,
            path: TreeNodePath::new_root(),
            parent_flat_index: None,
        };

        let mut flattened_nodes = vec![root_node];

        for item in first_depth_items {
            let node = Node {
                item,
                children_flat_indices: vec![],
                flat_index: flattened_nodes.len(),
                path: TreeNodePath(vec![flattened_nodes.len() - 1]),
                parent_flat_index: Some(0),
            };

            flattened_nodes.push(node);
        }

        Self {
            flattened_nodes,
            root_flat_index: 0,
        }
    }

    /// If successful, returns the node's path. (Kind of arbitrary, I just wanted to use option)
    pub fn add_node(&mut self, item: T, parent_path: &TreeNodePath) -> Option<TreeNodePath> {
        // prepare parent info
        let parent_flat_index = self.get_node_flat_index(parent_path)?;

        // prepare info about the new node
        let child_number = self.flattened_nodes[parent_flat_index]
            .children_flat_indices
            .len();
        let path = {
            let mut path_vec = parent_path.0.clone();
            path_vec.push(child_number);
            TreeNodePath(path_vec)
        };
        let flat_index = self.flattened_nodes.len();

        // generate node
        let node = Node {
            item,
            children_flat_indices: Vec::new(),
            flat_index,
            path: path.clone(),
            parent_flat_index: Some(parent_flat_index),
        };

        // add node to flattened indices
        self.flattened_nodes.push(node);

        // update parent
        self.flattened_nodes[parent_flat_index]
            .children_flat_indices
            .push(flat_index);

        Some(path)
    }

    pub fn iter_paths_dfs(&self) -> DfsPathsIterator<'_, T> {
        DfsPathsIterator {
            tree: self,
            tree_iterator: self.flattened_nodes.iter(),
        }
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

#[cfg(test)]
impl<T> Debug for Node<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("item", &self.item)
            .field("children_flat_indices", &self.children_flat_indices)
            .field("flat_index", &self.flat_index)
            .field("path", &self.path)
            .field("parent_flat_index", &self.parent_flat_index)
            .finish()
    }
}
#[cfg(test)]
impl<T> Debug for RootedTree<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootedTree")
            .field("flattened_nodes", &self.flattened_nodes)
            .field("root_flat_index", &self.root_flat_index)
            .finish()
    }
}

/// depth first search post-order
///
/// Eg: 1 { 2 { 3, 4 }, 5 { 6 } }
///
/// REVIEW: if rooted tree stores nodes in post order, this could be much simpler
///
/// NOTE: code is based off of the Iter for Vec
/// FIXME: actually implement this
pub struct DfsPathsIterator<'a, T: 'a> {
    tree: &'a RootedTree<T>,
    // next_path: TreeNodePath,
    tree_iterator: std::slice::Iter<'a, Node<T>>,
}
impl<'a, T> Iterator for DfsPathsIterator<'a, T> {
    type Item = TreeNodePath;

    fn next(&mut self) -> Option<Self::Item> {
        self.tree_iterator.next().map(|node| node.path.clone())
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

        // check that path points to an existing node
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
    /// No wrapping
    pub fn traverse_to_previous_sibling(&self) -> Option<Self> {
        let mut sibling_path_vec = self.0.clone();
        let last_child_number = sibling_path_vec.pop()?.checked_sub(1)?;
        sibling_path_vec.push(last_child_number);
        Some(Self(sibling_path_vec))
    }
    /// No wrapping
    pub fn traverse_to_next_sibling<T>(&self, rooted_tree: &RootedTree<T>) -> Option<Self> {
        let sibling_path = {
            let mut sibling_path_vec = self.0.clone();
            let last_child_number = sibling_path_vec.pop()?.checked_add(1)?;
            sibling_path_vec.push(last_child_number);
            Self(sibling_path_vec)
        };

        // check that path points to an existing node
        if rooted_tree.get_node_flat_index(&sibling_path).is_some() {
            Some(sibling_path)
        } else {
            None
        }
    }

    /// root has depth=0
    pub fn depth(&self) -> usize {
        self.0.len()
    }
}
impl<const N: usize> From<[usize; N]> for TreeNodePath {
    fn from(val: [usize; N]) -> Self {
        TreeNodePath(val.into())
    }
}

/// REVIEW make this output option?
/// Currently just panics if impossible
impl<T> std::ops::Index<&TreeNodePath> for RootedTree<T> {
    type Output = T;

    fn index(&self, tree_node_path: &TreeNodePath) -> &Self::Output {
        &self.flattened_nodes[self.get_node_flat_index(tree_node_path).unwrap()].item
    }
}
impl<T> std::ops::IndexMut<&TreeNodePath> for RootedTree<T> {
    fn index_mut(&mut self, tree_node_path: &TreeNodePath) -> &mut Self::Output {
        let flattened_index = self.get_node_flat_index(tree_node_path).unwrap();
        &mut self.flattened_nodes[flattened_index].item
    }
}
