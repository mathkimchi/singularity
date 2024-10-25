use singularity_common::{
    tab::TabHandler,
    utils::tree::{tree_node_path::TreeNodePath, uuid_tree::UuidTree},
};
use std::collections::BTreeMap;
use uuid::Uuid;

/// NOTE: `org` prefix in front of variable stands for `ORGanizational`.
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
    tabs: BTreeMap<Uuid, TabHandler>,

    /// ORGanizational tree
    org_tree: UuidTree,
    focused_tab: Uuid,

    /// currently, last in vec is "top" in gui
    display_order: Vec<Uuid>,
}
impl Tabs {
    pub fn new(root_tab: TabHandler) -> Self {
        let root_id = Uuid::new_v4();
        let mut tabs = BTreeMap::new();
        tabs.insert(root_id, root_tab);

        Self {
            tabs,
            org_tree: UuidTree::new(root_id),
            focused_tab: root_id,
            display_order: vec![root_id],
        }
    }

    pub fn add(&mut self, new_tab: TabHandler, parent_id: &Uuid) -> Option<Uuid> {
        let uuid = Uuid::new_v4();

        // add to `org_tree`
        self.org_tree.add_child(*parent_id, uuid);
        // add to `tabs`
        self.tabs.insert(uuid, new_tab);
        // add to top of display order
        self.display_order.push(uuid);

        // set focus to new tabs
        // REVIEW: is this bad?
        self.set_focused_tab_id(uuid);

        Some(uuid)
    }

    pub fn get_tab_handler(&self, uuid: Uuid) -> Option<&TabHandler> {
        self.tabs.get(&uuid)
    }

    pub fn get_mut_tab_handler(&mut self, uuid: Uuid) -> Option<&mut TabHandler> {
        self.tabs.get_mut(&uuid)
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
        self.org_tree.get_id_from_path(org_path)
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_path(&mut self, focused_tab_path: &TreeNodePath) {
        self.set_focused_tab_id(self.get_id_by_org_path(focused_tab_path).unwrap());
    }

    pub fn get_tab_path(&self, tab_uuid: &Uuid) -> Option<TreeNodePath> {
        self.org_tree.get_path(*tab_uuid)
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

    pub fn get_root_id(&self) -> Uuid {
        self.org_tree.get_root_id()
    }

    /// closes the tab and all its children
    ///
    /// TODO: do this with loop instead?
    fn close_tab_recursively(&mut self, id: &Uuid) {
        for child_id in self.org_tree.get_children(*id).clone() {
            self.close_tab_recursively(&child_id);
        }

        if self.org_tree.remove_recursive(*id) {
            self.tabs.remove(id);
            self.display_order.retain(|i| i != id);
        } else {
            println!("Tried to close root");
        }
    }

    /// closes the focused tab and all its children
    pub fn close_focused_tab_recursively(&mut self) {
        self.close_tab_recursively(&self.get_focused_tab_id());
        // TODO: self focused tab to parent
        self.set_focused_tab_id(self.get_root_id());
    }
}
impl singularity_common::utils::tree::tree_node_path::TraversableTree for Tabs {
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
