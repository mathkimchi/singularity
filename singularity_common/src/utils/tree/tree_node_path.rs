/// Like pointers but has the context of the tree structure
/// Also like a file path
///
/// NOTE: Most functions for a path are better thought of as functions for the Node that the path refers to
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct TreeNodePath(pub Vec<usize>);
impl TreeNodePath {
    pub fn new_root() -> Self {
        Self(Vec::new())
    }

    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// root has depth=0
    pub fn depth(&self) -> usize {
        self.0.len()
    }
}
impl<T: Into<Vec<usize>>> From<T> for TreeNodePath {
    fn from(val: T) -> Self {
        TreeNodePath(val.into())
    }
}

pub trait TraversableTree {
    fn exists_at(&self, path: &TreeNodePath) -> bool;

    fn iter_paths_dfs(&self) -> DfsPathsIterator<'_, Self>
    where
        Self: std::marker::Sized,
    {
        DfsPathsIterator {
            tree_to_traverse: self,
            next_path: Some(TreeNodePath::new_root()),
        }
    }

    fn collect_paths_dfs(&self) -> Vec<TreeNodePath>
    where
        Self: std::marker::Sized,
    {
        self.iter_paths_dfs().collect()
    }
}
/// For the traverse functions, some require the original tree to be safe
mod tree_node_path_traversal_impls {
    use super::{TraversableTree, TreeNodePath};
    impl TreeNodePath {
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

        pub fn unchecked_traverse_to_child(&self, child_index: usize) -> Self {
            let mut child_path_vec = self.0.clone();

            child_path_vec.push(child_index);

            Self(child_path_vec)
        }

        /// Needs the tree to make sure that the child exists
        pub fn traverse_to_child<T: TraversableTree>(
            &self,
            tree_to_traverse: &T,
            child_index: usize,
        ) -> Option<Self> {
            let child_path = {
                let mut child_path_vec = self.0.clone();

                child_path_vec.push(child_index);

                Self(child_path_vec)
            };

            // check that path points to an existing node
            if tree_to_traverse.exists_at(&child_path) {
                Some(child_path)
            } else {
                None
            }
        }

        /// Needs the tree to make sure that the child exists
        pub fn traverse_to_first_child<T: TraversableTree>(
            &self,
            tree_to_traverse: &T,
        ) -> Option<Self> {
            self.traverse_to_child(tree_to_traverse, 0)
        }

        /// No wrapping
        pub fn traverse_to_previous_sibling(&self) -> Option<Self> {
            let mut sibling_path_vec = self.0.clone();
            let last_child_number = sibling_path_vec.pop()?.checked_sub(1)?;
            sibling_path_vec.push(last_child_number);
            Some(Self(sibling_path_vec))
        }

        /// No wrapping
        pub fn traverse_to_next_sibling<T: TraversableTree>(
            &self,
            tree_to_traverse: &T,
        ) -> Option<Self> {
            let sibling_path = {
                let mut sibling_path_vec = self.0.clone();
                let last_child_number = sibling_path_vec.pop()?.checked_add(1)?;
                sibling_path_vec.push(last_child_number);
                Self(sibling_path_vec)
            };

            // check that path points to an existing node
            if tree_to_traverse.exists_at(&sibling_path) {
                Some(sibling_path)
            } else {
                None
            }
        }

        pub fn traverse_dfs_next<T: TraversableTree>(&self, tree_to_traverse: &T) -> Option<Self> {
            if let Some(first_child_path) = self.traverse_to_first_child(tree_to_traverse) {
                // has child
                Some(first_child_path)
            } else {
                // current is leaf
                // climb up (traverse to parents) until there is a next sibling or until at root
                let mut intermediate_path = self.clone();

                loop {
                    if let Some(next_path) =
                        intermediate_path.traverse_to_next_sibling(tree_to_traverse)
                    {
                        break Some(next_path);
                    }

                    if let Some(intermediate_path_parent) = intermediate_path.traverse_to_parent() {
                        intermediate_path = intermediate_path_parent
                    } else {
                        // intermediate path is root
                        break None;
                    }
                }
            }
        }

        /// This is a helper function, traversing trees based on wasd input
        ///
        /// returns None if keycode isn't wasd or if the traversal is invalid
        ///
        /// REVIEW: not sure if this belongs here, as it should be pure logic but this is more input handling
        /// TODO: q and e for bfs next
        /// TODO: seperate functions for wrapped traversal
        pub fn checked_traverse_based_on_wasd<T: TraversableTree>(
            &self,
            tree_to_traverse: &T,
            traverse_key: char,
        ) -> Option<Self> {
            match traverse_key {
                'a' => self.traverse_to_parent(),
                'd' => self.traverse_to_first_child(tree_to_traverse),
                'w' => self.traverse_to_previous_sibling(),
                's' => self.traverse_to_next_sibling(tree_to_traverse),
                _ => None,
            }
        }

        /// `checked_traverse_based_on_wasd` but if something goes wrong, return self
        pub fn clamped_traverse_based_on_wasd<T: TraversableTree>(
            &self,
            tree_to_traverse: &T,
            traverse_key: char,
        ) -> Self {
            self.checked_traverse_based_on_wasd(tree_to_traverse, traverse_key)
                .unwrap_or(self.clone())
        }
    }
}

/// depth first search post-order
///
/// Eg: 1 { 2 { 3, 4 }, 5 { 6 } }
///
/// REVIEW: if rooted tree stores nodes in post order, this could be much simpler
///
/// NOTE: code is based off of the Iter for Vec
pub struct DfsPathsIterator<'a, T: 'a + TraversableTree> {
    tree_to_traverse: &'a T,
    next_path: Option<TreeNodePath>,
}
impl<'a, T: TraversableTree> Iterator for DfsPathsIterator<'a, T> {
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

        self.next_path = current_path.traverse_dfs_next(self.tree_to_traverse);

        Some(current_path)
    }
}
