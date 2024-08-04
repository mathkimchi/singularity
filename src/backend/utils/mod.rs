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
//!
//! TODO: macro for creation

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
            next_path: Some(TreeNodePath::new_root()),
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

/// depth first search post-order
///
/// Eg: 1 { 2 { 3, 4 }, 5 { 6 } }
///
/// REVIEW: if rooted tree stores nodes in post order, this could be much simpler
///
/// NOTE: code is based off of the Iter for Vec
pub struct DfsPathsIterator<'a, T: 'a> {
    tree: &'a RootedTree<T>,
    next_path: Option<TreeNodePath>,
}
impl<'a, T> Iterator for DfsPathsIterator<'a, T> {
    type Item = TreeNodePath;

    /// # Explanation of finding the next path non-recursively:
    ///
    /// ## Terminology
    /// - `visited`
    /// - `fully explored`: node and all its children are fully visited
    /// - `a is left of b`: a should be visited before b
    ///
    /// ## Properties
    ///
    /// The next node should:
    /// 1. be unvisited
    /// 2. have all ancestors visited
    /// 3. have all older sibling fully expored
    ///
    /// iff fully explored:
    /// - node's last child's last child's ... is visited
    /// - aka node [is visited] and [[is leaf] or [node's last child is fully explored]]
    ///
    /// iff a visited before b:
    /// - a is parent of b or when they first split, a stems from the older sibling
    /// - easiest to compare the paths
    ///
    /// ## Cases to gain insight for generalization:
    /// - current has child: then next should be the first child
    /// - current is leaf but has next sibling: then current is fully explored and next is next sibling
    /// - current is leaf and last sibling: then parent is fully explored
    /// - parent is fully explored but has next sibling: parent's next sibling is next
    /// - parent is fully explored and last sibling: then grandparent is fully explored if grandparent has
    /// - ancestor D is fully explored but has next sibling: ancestor D's next sibling is next
    /// - ancestor D(epth) is fully explored and last sibling: ancestor D-1 is fully explored
    ///
    /// So, once at a leaf, the youngest ancestor (or self)'s next sibling that exists is next
    /// if none of those exist then everything is fully explored
    fn next(&mut self) -> Option<Self::Item> {
        let current_path = self.next_path.clone()?;

        if let Some(first_child_path) = current_path.traverse_to_first_child(self.tree) {
            // has child
            self.next_path = Some(first_child_path);
        } else {
            // current is leaf
            // climb up (traverse to parents) until there is a next sibling or until at root
            let mut intermediate_path = current_path.clone();

            self.next_path = loop {
                if let Some(next_path) = intermediate_path.traverse_to_next_sibling(self.tree) {
                    break Some(next_path);
                }

                if let Some(intermediate_path_parent) = intermediate_path.traverse_to_parent() {
                    intermediate_path = intermediate_path_parent
                } else {
                    // intermediate path is root
                    break None;
                }
            };
        }

        Some(current_path)
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

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
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
