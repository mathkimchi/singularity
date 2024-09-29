use singularity_common::{
    tab::TabHandler,
    utils::tree::{rooted_tree::RootedTree, tree_node_path::TreeNodePath},
};
use std::collections::BTreeMap;
use uuid::Uuid;

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
    tabs: BTreeMap<Uuid, (TabHandler, TreeNodePath)>,

    focused_tab: Uuid,

    organizational_hierarchy: RootedTree<Uuid>,

    /// currently, last in vec is "top" in gui
    display_order: Vec<Uuid>,
}
impl Tabs {
    pub fn new(root_tab: TabHandler) -> Self {
        let root_id = Uuid::new_v4();
        let mut tabs = BTreeMap::new();
        tabs.insert(root_id, (root_tab, TreeNodePath::new_root()));

        Self {
            tabs,
            organizational_hierarchy: RootedTree::from_root(root_id),
            display_order: vec![root_id],
            focused_tab: root_id,
        }
    }

    /// NOTE: Currently doesn't change focus
    pub fn add(&mut self, new_tab: TabHandler, parent_path: &TreeNodePath) {
        let uuid = Uuid::new_v4();

        let path = self
            .organizational_hierarchy
            .add_node(uuid, parent_path)
            .unwrap();
        self.display_order.push(uuid);
        self.tabs.insert(uuid, (new_tab, path));
    }

    pub fn get(&self, uuid: Uuid) -> Option<&TabHandler> {
        self.tabs.get(&uuid).map(|(tab, _)| tab)
    }

    pub fn get_mut(&mut self, uuid: Uuid) -> Option<&mut TabHandler> {
        self.tabs.get_mut(&uuid).map(|(tab, _)| tab)
    }

    pub fn get_organizational_hierarchy(&self) -> &RootedTree<Uuid> {
        &self.organizational_hierarchy
    }

    pub fn get_display_order(&self) -> &Vec<Uuid> {
        &self.display_order
    }

    pub fn get_focused_tab_id(&self) -> Uuid {
        self.focused_tab
    }

    pub fn get_focused_tab_mut(&mut self) -> &mut TabHandler {
        self.get_mut(self.get_focused_tab_id()).unwrap()
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_path(&mut self, focused_tab_path: TreeNodePath) {
        self.focused_tab = self.organizational_hierarchy[&focused_tab_path];

        // move the focused tab to end of display order (putting it on top)
        {
            self.display_order
                .retain(|tab_id| tab_id != &self.focused_tab);

            self.display_order.push(self.focused_tab);
        }
    }

    pub fn get_tab_path(&mut self, tab_uuid: &Uuid) -> Option<&TreeNodePath> {
        self.tabs.get(tab_uuid).map(|(_tab, path)| path)
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
}
impl std::ops::Index<Uuid> for Tabs {
    type Output = TabHandler;

    fn index(&self, index: Uuid) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl std::ops::IndexMut<Uuid> for Tabs {
    fn index_mut(&mut self, index: Uuid) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
impl std::ops::Index<&TreeNodePath> for Tabs {
    type Output = TabHandler;

    fn index(&self, index: &TreeNodePath) -> &Self::Output {
        &self[self.organizational_hierarchy[index]]
    }
}
impl std::ops::IndexMut<&TreeNodePath> for Tabs {
    fn index_mut(&mut self, index: &TreeNodePath) -> &mut Self::Output {
        let uuid = self.organizational_hierarchy[index];
        &mut self[uuid]
    }
}
