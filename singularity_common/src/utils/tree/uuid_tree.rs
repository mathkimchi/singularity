use std::collections::BTreeMap;
use uuid::Uuid;

use super::tree_node_path::{TraversableTree, TreeNodePath};

/// Technically, children are ordered
#[derive(Default)]
struct Node {
    children: Vec<Uuid>,
    parent: Option<Uuid>,
}

/// Only stores hierarchy, no items.
pub struct UuidTree {
    root_id: Uuid,
    nodes: BTreeMap<Uuid, Node>,
}
impl Default for UuidTree {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}
impl UuidTree {
    pub fn new(root_id: Uuid) -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(root_id, Node::default());
        Self { root_id, nodes }
    }

    pub fn add_child(&mut self, parent_id: Uuid, child_id: Uuid) -> Option<()> {
        // add child to `nodes`
        self.nodes.insert(
            child_id,
            Node {
                children: Vec::new(),
                parent: Some(parent_id),
            },
        );
        // register child under parent
        self.nodes.get_mut(&parent_id)?.children.push(child_id);

        Some(())
    }

    /// generates a new id and adds it
    pub fn create_child(&mut self, parent_id: Uuid) -> Option<Uuid> {
        let child_id = Uuid::new_v4();
        self.add_child(parent_id, child_id);
        Some(child_id)
    }

    /// removes node corresponding to id and all its children
    ///
    /// TODO: do this with loop instead?
    pub fn remove_recursive(&mut self, id: Uuid) {
        for child in self.nodes.get(&id).unwrap().children.clone() {
            self.remove_recursive(child);
        }

        if let Some(parent_id) = self.nodes.get(&id).unwrap().parent {
            // remove this from parent
            self.nodes
                .get_mut(&parent_id)
                .unwrap()
                .children
                .retain(|i| i != &id);
        } else {
            panic!("Tried to call remove on root");
        }
    }

    pub fn get_root_id(&self) -> Uuid {
        self.root_id
    }

    /// climb downwards
    pub fn get_id_from_path(&self, path: &TreeNodePath) -> Option<Uuid> {
        let mut node_id = self.root_id;

        for index in path.0.iter() {
            node_id = *self.nodes.get(&node_id)?.children.get(*index)?;
        }

        Some(node_id)
    }

    /// climb upwards
    pub fn get_path(&self, id: Uuid) -> Option<TreeNodePath> {
        let mut path_vec = Vec::new();
        let mut node_id = id;

        while let Some(parent_id) = self.nodes.get(&node_id).unwrap().parent {
            let child_index = self
                .nodes
                .get(&parent_id)
                .unwrap()
                .children
                .iter()
                .position(|id| id == &node_id)
                .unwrap();
            path_vec.push(child_index);

            node_id = parent_id;
        }

        path_vec.reverse();

        Some(path_vec.into())
    }
}
impl TraversableTree for UuidTree {
    fn exists_at(&self, path: &TreeNodePath) -> bool {
        self.get_id_from_path(path).is_some()
    }
}
