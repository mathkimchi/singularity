//! For a rooted tree and structured index,
//! the thingss I want to do are:
//!
//! - Traverse via index:
//!  - Between parent and child
//!  - Between adjacent siblings
//! - Display tree in a structured manner
//!
//! I wish it could be safe, but I don't think it will be.
//! Invalid states are representable, and removing nodes might mess up everything.

// use std::fmt::Debug;

// /// TODO: not exactly what I want right now
// ///
// /// Ex: `tree!(3 => [ 1 => [], 7 => [ 8 ] ])`
// ///
// /// Leaf can be `item => []` or `item`
// #[macro_export]
// macro_rules! tree {
//     [$tree_item:expr] => {
//         $crate::backend::utils::TreeNode::new_leaf($tree_item)
//     };

//     [$tree_item:expr => $children:expr] => {
//         $crate::backend::utils::TreeNode::new($tree_item, $children.into())
//     }; // ($item:expr => []) => {
//        //     $crate::backend::utils::TreeNode::new_leaf($item)
//        // };
// }

// pub struct TreeNode<T> {
//     item: T,
//     children: Vec<TreeNode<T>>,
// }
// impl<T> TreeNode<T> {
//     pub fn new(item: T, children: Vec<Self>) -> Self {
//         Self { item, children }
//     }

//     pub fn new_leaf(item: T) -> Self {
//         Self {
//             item,
//             children: Vec::new(),
//         }
//     }

//     pub fn get_item_ref(&self) -> &T {
//         &self.item
//     }

//     pub fn get_children_ref(&self) -> &Vec<Self> {
//         &self.children
//     }

//     /// I don't understand lifetimes, I just did what the compiler said to do
//     fn flattened_ref_recursive<'a, 'b: 'a>(
//         flat: &mut Vec<(Vec<usize>, &'a Self)>,
//         node: &'b Self,
//         tree_index: Vec<usize>,
//     ) {
//         // add self
//         flat.push((tree_index.clone(), node));

//         // add children
//         for (child_extra_index, child) in node.children.iter().enumerate() {
//             let child_tree_index = {
//                 let mut child_tree_index = tree_index.clone();
//                 child_tree_index.push(child_extra_index);
//                 child_tree_index
//             };
//             Self::flattened_ref_recursive(flat, child, child_tree_index);
//         }
//     }

//     /// returns `[(index, node), ...]`
//     /// definitely has a horrible Big O, but this is meant for small cases
//     pub fn flattened_ref(&self) -> Vec<(Vec<usize>, &Self)> {
//         let mut flat = Vec::new();

//         Self::flattened_ref_recursive(&mut flat, self, Vec::new());

//         flat
//     }
// }
// impl<T: Debug> Debug for TreeNode<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("TreeNode")
//             .field("item", &self.item)
//             .field("children", &self.children)
//             .finish()
//     }
// }

// pub type TreeIndex = [usize];
// impl<T> std::ops::Index<&TreeIndex> for TreeNode<T> {
//     type Output = TreeNode<T>;

//     /// This can cause an error
//     ///
//     /// Returns the treenode at the index, not the item
//     fn index(&self, index: &TreeIndex) -> &Self::Output {
//         if index.is_empty() {
//             return self;
//         }

//         &self.children[index[0]][&index[1..]]
//     }
// }
// impl<T> std::ops::IndexMut<&TreeIndex> for TreeNode<T> {
//     /// Returns the treenode at the index, not the item
//     fn index_mut(&mut self, index: &TreeIndex) -> &mut Self::Output {
//         if index.is_empty() {
//             return self;
//         }

//         &mut self.children[index[0]][&index[1..]]
//     }
// }

// ------- above is a simple recursive tree implementation

#[macro_export]
macro_rules! rooted_tree {
    [$tree_item:expr] => {
        $crate::backend::utils::RootedTree::new()
    };

    [$tree_item:expr => $children:expr] => {
        $crate::backend::utils::RootedTree::new()
    };
}

pub struct RootedTree<T>(std::marker::PhantomData<T>);
impl<T> RootedTree<T> {
    pub fn new() -> Self {
        todo!()
    }

    pub fn get_root_path(&self) -> TreeNodePath {
        todo!()
    }

    pub fn iter_paths_dfs(&self) -> DfsPathsIterator {
        todo!()
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
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TreeNodePath {}
impl TreeNodePath {
    pub fn traverse_to_parent(&self) -> Option<Self> {
        todo!()
    }
    pub fn traverse_to_child(&self, _child_index: usize) -> Option<Self> {
        todo!()
    }
    pub fn traverse_to_first_child(&self) -> Option<Self> {
        todo!()
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

impl<T> std::ops::Index<&TreeNodePath> for RootedTree<T> {
    type Output = T;

    fn index(&self, _index: &TreeNodePath) -> &Self::Output {
        todo!()
    }
}
impl<T> std::ops::IndexMut<&TreeNodePath> for RootedTree<T> {
    fn index_mut(&mut self, _index: &TreeNodePath) -> &mut Self::Output {
        todo!()
    }
}
