use singularity_common::{
    tab::TabHandler,
    utils::tree::tree_node_path::{TraversableTree, TreeNodePath},
};
use std::collections::BTreeMap;
use uuid::Uuid;

/// NOTE: `org` prefix in front of variable stands for `ORGanizational`.
/// This refers to the organizational tree hierarchy of the tabs.
///
/// REVIEW: store uuid in here?
struct TabNode {
    tab_handler: TabHandler,

    org_children: Vec<Uuid>,
    org_path: TreeNodePath,
    _org_parent: Option<Uuid>,
}
impl TabNode {
    pub fn new_root(tab_handler: TabHandler) -> Self {
        TabNode {
            tab_handler,
            org_children: Vec::new(),
            org_path: TreeNodePath::new_root(),
            _org_parent: None,
        }
    }

    fn register_child_id(&mut self, child_id: Uuid) -> TreeNodePath {
        // NOTE: order matters
        let child_path = self
            .org_path
            .unchecked_traverse_to_child(self.org_children.len());

        self.org_children.push(child_id);

        child_path
    }
}

/// REVIEW: currently, must have at least one tab. change?
///
/// There is a lot of redundancy in storage
///
/// NOTE: Since I don't want to think too hard about what type
/// I should use to represent a reference to a tab,
/// I will just always store the uuid.
/// Then, things like tree node path or display order index
/// can be found from the uuid.
pub struct Tabs {
    /// NOTE: the BTree for BTreeMap doesn't have anything to do with the org tree
    tabs: BTreeMap<Uuid, TabNode>,

    org_root: Uuid,
    focused_tab: Uuid,

    /// currently, last in vec is "top" in gui
    display_order: Vec<Uuid>,
}
impl Tabs {
    pub fn new(root_tab: TabHandler) -> Self {
        let root_id = Uuid::new_v4();
        let mut tabs = BTreeMap::new();
        tabs.insert(root_id, TabNode::new_root(root_tab));

        Self {
            tabs,
            org_root: root_id,
            focused_tab: root_id,
            display_order: vec![root_id],
        }
    }

    pub fn add(&mut self, new_tab: TabHandler, parent_id: &Uuid) -> Option<Uuid> {
        let uuid = Uuid::new_v4();

        // register child under parent
        let path = self.tabs.get_mut(parent_id)?.register_child_id(uuid);

        // add to `tabs`
        self.tabs.insert(
            uuid,
            TabNode {
                tab_handler: new_tab,
                org_children: Vec::new(),
                org_path: path.clone(),
                _org_parent: Some(*parent_id),
            },
        );

        // add to top of display order
        self.display_order.push(uuid);

        // set focus to new tabs
        // REVIEW: is this bad?
        self.set_focused_tab_path(&path);

        Some(uuid)
    }

    pub fn get_tab_handler(&self, uuid: Uuid) -> Option<&TabHandler> {
        self.tabs.get(&uuid).map(|tab_node| &tab_node.tab_handler)
    }

    pub fn get_mut_tab_handler(&mut self, uuid: Uuid) -> Option<&mut TabHandler> {
        self.tabs
            .get_mut(&uuid)
            .map(|tab_node| &mut tab_node.tab_handler)
    }

    pub fn get_focused_tab_mut(&mut self) -> &mut TabHandler {
        self.get_mut_tab_handler(self.get_focused_tab_id()).unwrap()
    }

    pub fn get_display_order(&self) -> &Vec<Uuid> {
        &self.display_order
    }

    pub fn get_focused_tab_id(&self) -> Uuid {
        self.focused_tab
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_id(&mut self, focused_tab_id: Uuid) {
        self.focused_tab = focused_tab_id;

        // move the focused tab to end of display order (putting it on top)
        {
            self.display_order
                .retain(|tab_id| tab_id != &self.focused_tab);

            self.display_order.push(self.focused_tab);
        }
    }

    pub fn get_id_by_org_path(&self, org_path: &TreeNodePath) -> Option<Uuid> {
        let mut current_node_id = self.org_root;
        for child_number in &org_path.0 {
            let current_node = &self.tabs.get(&current_node_id).unwrap();

            let next_node_id = current_node.org_children.get(*child_number);

            if let Some(next_node_id) = next_node_id {
                current_node_id = *next_node_id;
            } else {
                return None;
            }
        }
        Some(current_node_id)
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_path(&mut self, focused_tab_path: &TreeNodePath) {
        self.set_focused_tab_id(self.get_id_by_org_path(focused_tab_path).unwrap());
    }

    pub fn get_tab_path(&self, tab_uuid: &Uuid) -> Option<&TreeNodePath> {
        self.tabs.get(tab_uuid).map(|tab_node| &tab_node.org_path)
    }

    pub fn minimize_focused_tab(&mut self) {
        // remove the focused tab from the display order as to not render it

        self.display_order
            .retain(|tab_id| tab_id != &self.focused_tab);

        // REVIEW: is this good?
        // make the topmost tab the new focused tab
        if let Some(uuid) = self.display_order.last() {
            self.focused_tab = *uuid;
        }
    }

    pub fn num_tabs(&self) -> usize {
        self.tabs.len()
    }

    // /// closes the tab and all its children
    // fn close_tab_recursively(&mut self, path: &TreeNodePath) {
    //     for child_path in self.organizational_hierarchy {}
    // }

    // /// closes the focused tab and all its children
    // pub fn close_focused_tab_recursively(&mut self) {
    //     let focused_tab_path = self.get_tab_path(&self.focused_tab).unwrap().clone();
    //     self.close_tab_recursively(&focused_tab_path);
    // }
}
impl TraversableTree for Tabs {
    fn exists_at(&self, path: &TreeNodePath) -> bool {
        self.get_id_by_org_path(path).is_some()
    }
}
// impl std::ops::Index<Uuid> for Tabs {
//     type Output = TabHandler;

//     fn index(&self, index: Uuid) -> &Self::Output {
//         self.get(index).unwrap()
//     }
// }
// impl std::ops::IndexMut<Uuid> for Tabs {
//     fn index_mut(&mut self, index: Uuid) -> &mut Self::Output {
//         self.get_mut(index).unwrap()
//     }
// }
// impl std::ops::Index<&TreeNodePath> for Tabs {
//     type Output = TabHandler;

//     fn index(&self, index: &TreeNodePath) -> &Self::Output {
//         &self[self.organizational_hierarchy[index]]
//     }
// }
// impl std::ops::IndexMut<&TreeNodePath> for Tabs {
//     fn index_mut(&mut self, index: &TreeNodePath) -> &mut Self::Output {
//         let uuid = self.organizational_hierarchy[index];
//         &mut self[uuid]
//     }
// }
