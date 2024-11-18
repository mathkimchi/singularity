//! this is for tab placement, like hyprland
//! Tabs are stored in a binary tree method

use crate::utils::id_map::{Id, IdMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::TabHandler;

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}
impl Orientation {
    fn get_transpose(&self) -> Self {
        match self {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        }
    }
}

/// Could do recursive enums, but I will do the UUID way
///
/// REVIEW: abstract something about the pattern where items are represented by Uuids and stored in B-maps
/// Maybe like ID map
#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub enum Tile {
    Container {
        // parent_tile: Option<Id<Tile>>,
        children: [Id<Tile>; 2],
        orientation: Orientation,
        split: f32,
    },
    /// Leaf node points to a full window
    Tab {
        /// REVIEW: adding onto the id map idea, I want to be able to specify this is a Uuid pointing to Tab
        tab_id: Id<TabHandler>,
    },
}
impl Tile {
    pub fn try_as_container(&self) -> Option<([Id<Tile>; 2], Orientation, f32)> {
        if let Tile::Container {
            children,
            orientation,
            split,
        } = *self
        {
            Some((children, orientation, split))
        } else {
            None
        }
    }

    pub fn try_as_tab(&self) -> Option<Id<TabHandler>> {
        if let Tile::Tab { tab_id } = *self {
            Some(tab_id)
        } else {
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tiles {
    root_id: Id<Tile>,
    tiles: IdMap<Tile>,
    /// TODO: rename leaf to tab
    leaf_registry: BTreeMap<Id<TabHandler>, Id<Tile>>,
}
impl Tiles {
    pub fn new_from_root(tab_id: Id<TabHandler>) -> Self {
        let root_tile_id = Id::generate();
        let mut tiles = IdMap::new();
        tiles.insert(root_tile_id, Tile::Tab { tab_id });
        let mut leaf_registry = BTreeMap::new();
        leaf_registry.insert(tab_id, root_tile_id);

        Self {
            root_id: root_tile_id,
            tiles,
            leaf_registry,
        }
    }

    pub fn give_sibling(&mut self, older_tab_id: Id<TabHandler>, younger_tab_id: Id<TabHandler>) {
        let original_tile_id = self.leaf_registry[&older_tab_id];

        let older_tile_id = Id::generate();
        let younger_tile_id = Id::generate();
        self.tiles.insert(
            older_tile_id,
            Tile::Tab {
                tab_id: older_tab_id,
            },
        );
        self.tiles.insert(
            younger_tile_id,
            Tile::Tab {
                tab_id: younger_tab_id,
            },
        );
        self.leaf_registry.insert(older_tab_id, older_tile_id);
        self.leaf_registry.insert(younger_tab_id, younger_tile_id);

        let original_tile = self.tiles.get_mut(&original_tile_id).unwrap();

        assert!(matches!(original_tile, Tile::Tab { tab_id } if tab_id==&older_tab_id));

        *original_tile = Tile::Container {
            children: [older_tile_id, younger_tile_id],
            orientation: Orientation::Horizontal,
            split: 0.5,
        };
    }

    pub fn remove(&mut self, tab_id: Id<TabHandler>) {
        let tile_to_remove = self.get_leaf_tile_id(tab_id).unwrap();
        let parent_tile_id = self
            .get_parent_tile_id(tile_to_remove)
            .expect("tried to remove root tile");

        let sibling_id = self.tiles[&parent_tile_id]
            .try_as_container()
            .expect("parent must be variant `Tile::Container`")
            .0
            .into_iter()
            .find(|c| c != &tile_to_remove)
            .unwrap();

        // the sibling takes the place of the parent
        let sibling = self.tiles.remove(&sibling_id).unwrap();
        self.tiles.insert(parent_tile_id, sibling).unwrap();
        if let Some(sibling_tab) = sibling.try_as_tab() {
            // update the leaf registry entry for sibling tab
            self.leaf_registry
                .insert(sibling_tab, parent_tile_id)
                .unwrap();
        }

        self.leaf_registry.remove(&tab_id).unwrap();
        self.tiles.remove(&tile_to_remove).unwrap();
    }

    pub fn get_root_tile(&self) -> Id<Tile> {
        self.root_id
    }

    // pub fn get_leaf_tile_id(&self, tab_id: Id<TabHandler>) -> Option<Id<Tile>> {
    //     self.leaf_registry.get(&tab_id).copied()
    // }

    pub fn get_tile(&self, tile_id: Id<Tile>) -> Option<&Tile> {
        self.tiles.get(&tile_id)
    }

    pub fn transpose_container(&mut self, container_tile_id: Id<Tile>) {
        if let Some(Tile::Container {
            children: _,
            orientation,
            split: _,
        }) = self.tiles.get_mut(&container_tile_id)
        {
            *orientation = orientation.get_transpose();
        }
    }

    pub fn swap_children(&mut self, container_tile_id: Id<Tile>) {
        if let Some(Tile::Container {
            children,
            orientation: _,
            split: _,
        }) = self.tiles.get_mut(&container_tile_id)
        {
            children.swap(0, 1);
        }
    }

    /// NOTE: currently searches for parent that has the child
    /// REVIEW: optimize by storing the parents
    pub fn get_parent_tile_id(&mut self, child_tile_id: Id<Tile>) -> Option<Id<Tile>> {
        self.tiles.iter().find_map(|(parent_id, parent_tile)| {
            if let Tile::Container { children, .. } = parent_tile {
                if children.contains(&child_tile_id) {
                    Some(*parent_id)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn get_leaf_tile_id(&self, tab_handler: Id<TabHandler>) -> Option<Id<Tile>> {
        self.leaf_registry.get(&tab_handler).copied()
    }
}
