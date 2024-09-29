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
    /// FIXME: Since tab handler stores uuid, storing the key as well
    /// is kind of redundant.
    tabs: BTreeMap<Uuid, TabHandler>,

    focused_tab: Uuid,

    organizational_hierarchy: RootedTree<Uuid>,

    /// currently, last in vec is "top" in gui
    display_order: Vec<Uuid>,
}
impl Tabs {
    pub fn new(root_tab: TabHandler) -> Self {
        let root_id = root_tab.get_uuid();
        let mut tabs = BTreeMap::new();
        tabs.insert(root_id, root_tab);

        Self {
            tabs,
            organizational_hierarchy: RootedTree::from_root(root_id),
            display_order: vec![root_id],
            focused_tab: root_id,
        }
    }

    /// NOTE: Currently doesn't change focus
    pub fn add(&mut self, new_tab: TabHandler, parent_path: &TreeNodePath) {
        self.organizational_hierarchy
            .add_node(new_tab.get_uuid(), parent_path);
        self.display_order.push(new_tab.get_uuid());
        self.tabs.insert(new_tab.get_uuid(), new_tab);
    }

    pub fn get(&self, uuid: Uuid) -> Option<&TabHandler> {
        self.tabs.get(&uuid)
    }

    pub fn get_mut(&mut self, uuid: Uuid) -> Option<&mut TabHandler> {
        self.tabs.get_mut(&uuid)
    }

    pub fn get_organizational_hierarchy(&self) -> &RootedTree<Uuid> {
        &self.organizational_hierarchy
    }

    pub fn get_display_order(&self) -> &RootedTree<Uuid> {
        &self.organizational_hierarchy
    }

    pub fn get_focused_tab_id(&self) -> Uuid {
        self.focused_tab
    }

    pub fn get_focused_tab_mut(&mut self) -> &mut TabHandler {
        self.get_mut(self.get_focused_tab_id()).unwrap()
    }

    pub fn set_focused_tab_path(&mut self, focused_tab_path: TreeNodePath) {
        self.focused_tab = self.organizational_hierarchy[&focused_tab_path];
    }

    pub fn get_tab_path(&mut self, tab_uuid: Uuid) -> TreeNodePath {
        todo!()
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
