use singularity_common::{
    project::{project_settings::TabData, Project},
    tab::{tile::Tiles, TabHandler},
    utils::{
        id_map::{Id, IdMap},
        tree::{id_tree::IdTree, tree_node_path::TreeNodePath},
    },
};
use singularity_ui::display_units::DisplayArea;

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
    tabs: IdMap<TabHandler>,

    /// ORGanizational tree
    org_tree: IdTree<TabHandler>,
    focused_tab: Id<TabHandler>,

    // /// currently, last in vec is "top" in gui
    // display_order: Vec<Uuid>,
    display_tiles: Tiles,
}
impl Tabs {
    pub fn parse_from_project(project: &Project) -> Self {
        if let Some(open_tabs) = project.get_project_settings().open_tabs.clone() {
            Self {
                tabs: open_tabs
                    .tabs
                    .into_iter()
                    .map(|(id, open_tab)| {
                        (
                            uuid::Uuid::from(id).into(),
                            TabHandler::new(
                                singularity_standard_tabs::get_tab_creator_from_type(
                                    open_tab.tab_data.tab_type.as_str(),
                                ),
                                open_tab.tab_data,
                                open_tab.tab_area,
                            ),
                        )
                    })
                    .collect(),
                org_tree: open_tabs.org_tree,
                focused_tab: open_tabs.focused_tab,
                display_tiles: open_tabs.display_tiles,
            }
        } else {
            // create new project
            use singularity_common::tab::BasicTab;
            use singularity_standard_tabs::{
                file_manager::FileManager, task_organizer::TaskOrganizer,
            };

            let mut tabs = Tabs::new_from_root(TabHandler::new(
                FileManager::new_tab_creator(),
                TabData {
                    tab_type: "FILE_MANAGER".to_string(),
                    session_data: serde_json::to_value(project.get_project_directory().clone())
                        .unwrap(),
                },
                DisplayArea::new((0., 0.), (0.5, 1.)),
            ));

            tabs.add(
                TabHandler::new(
                    TaskOrganizer::new_tab_creator(),
                    TabData {
                        tab_type: "TASK_ORGANIZER".to_string(),
                        session_data: serde_json::to_value(project.get_project_directory().clone())
                            .unwrap(),
                    },
                    DisplayArea::new((0.5, 0.), (1.0, 1.)),
                ),
                &tabs.get_root_id(),
            );

            tabs
        }
    }

    fn new_from_root_with_id(root_tab: TabHandler, root_id: Id<TabHandler>) -> Self {
        let mut tabs = IdMap::new();
        tabs.insert(root_id, root_tab);

        Self {
            tabs,
            org_tree: IdTree::new(root_id),
            focused_tab: root_id,
            display_tiles: Tiles::new_from_root(root_id),
        }
    }

    pub fn new_from_root(root_tab: TabHandler) -> Self {
        Self::new_from_root_with_id(root_tab, Id::generate())
    }

    pub fn add(
        &mut self,
        new_tab: TabHandler,
        parent_id: &Id<TabHandler>,
    ) -> Option<Id<TabHandler>> {
        let uuid = Id::generate();

        // add to `org_tree`
        self.org_tree.add_child(*parent_id, uuid);
        // add to `tabs`
        self.tabs.insert(uuid, new_tab);
        // add to top of display order
        self.display_tiles.give_sibling(self.focused_tab, uuid);

        // set focus to new tabs
        // REVIEW: is this bad?
        self.set_focused_tab_id(uuid);

        Some(uuid)
    }

    pub fn get_tab_handler(&self, uuid: Id<TabHandler>) -> Option<&TabHandler> {
        self.tabs.get(&uuid)
    }

    pub fn get_mut_tab_handler(&mut self, uuid: Id<TabHandler>) -> Option<&mut TabHandler> {
        self.tabs.get_mut(&uuid)
    }

    pub fn get_focused_tab_mut(&mut self) -> &mut TabHandler {
        self.get_mut_tab_handler(self.get_focused_tab_id()).unwrap()
    }

    pub fn get_display_tiles(&self) -> &Tiles {
        &self.display_tiles
    }

    pub fn transpose_focused_tile_parent(&mut self) {
        let container_tile_id = self
            .display_tiles
            .get_parent_tile_id(
                self.display_tiles
                    .get_leaf_tile_id(self.focused_tab)
                    .unwrap(),
            )
            .unwrap();

        self.display_tiles.transpose_container(container_tile_id);
    }

    pub fn swap_focused_tile_siblings(&mut self) {
        let container_tile_id = self
            .display_tiles
            .get_parent_tile_id(
                self.display_tiles
                    .get_leaf_tile_id(self.focused_tab)
                    .unwrap(),
            )
            .unwrap();

        self.display_tiles.swap_children(container_tile_id);
    }

    pub fn get_focused_tab_id(&self) -> Id<TabHandler> {
        self.focused_tab
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_id(&mut self, focused_tab_id: Id<TabHandler>) {
        self.focused_tab = focused_tab_id;

        // move the focused tab to end of display order (putting it on top)
        {
            // self.display_order
            //     .retain(|tab_id| tab_id != &self.focused_tab);

            // self.display_order.push(self.focused_tab);
        }
    }

    pub fn get_id_by_org_path(&self, org_path: &TreeNodePath) -> Option<Id<TabHandler>> {
        self.org_tree.get_id_from_path(org_path)
    }

    /// NOTE: has a side effect of putting the newly focused tab on display top
    pub fn set_focused_tab_path(&mut self, focused_tab_path: &TreeNodePath) {
        self.set_focused_tab_id(self.get_id_by_org_path(focused_tab_path).unwrap());
    }

    pub fn get_tab_path(&self, tab_uuid: &Id<TabHandler>) -> Option<TreeNodePath> {
        self.org_tree.get_path(*tab_uuid)
    }

    // pub fn minimize_focused_tab(&mut self) {
    //     todo!()

    //     // // remove the focused tab from the display order as to not render it

    //     // self.display_order
    //     //     .retain(|tab_id| tab_id != &self.focused_tab);

    //     // // REVIEW: is this good?
    //     // // make the topmost tab the new focused tab
    //     // if let Some(uuid) = self.display_order.last() {
    //     //     self.focused_tab = *uuid;
    //     // }
    // }

    pub fn num_tabs(&self) -> usize {
        self.tabs.len()
    }

    pub fn collect_tab_ids(&self) -> Vec<Id<TabHandler>> {
        self.tabs.keys().cloned().collect()
    }

    pub fn get_root_id(&self) -> Id<TabHandler> {
        self.org_tree.get_root_id()
    }

    /// closes the tab and all its children
    ///
    /// TODO: do this with loop instead?
    fn close_tab_recursively(&mut self, id: &Id<TabHandler>) {
        for child_id in self.org_tree.get_children(*id).clone() {
            self.close_tab_recursively(&child_id);
        }

        if self.org_tree.remove_recursive(*id) {
            self.tabs.remove(id);
            self.display_tiles.remove(*id);
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

    /// Save this session
    /// REVIEW: Rename to export?
    pub fn save_session(&self) -> singularity_common::project::project_settings::OpenTabs {
        use singularity_common::project::project_settings::{OpenTab, OpenTabs};

        OpenTabs {
            tabs: self
                .tabs
                .iter()
                .map(|(id, handler)| {
                    (
                        uuid::Uuid::from(*id).into(),
                        OpenTab {
                            // TODO
                            tab_area: handler.get_area(),
                            tab_data: handler.get_tab_data().clone(),
                        },
                    )
                })
                .collect(),
            org_tree: self.org_tree.clone(),
            focused_tab: self.focused_tab,
            display_tiles: self.display_tiles.clone(),
        }
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
