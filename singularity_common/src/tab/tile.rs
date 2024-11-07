//! this is for tab placement, like hyprland
//! Tabs are stored in a binary tree method

use crate::utils::id_map::{Id, IdMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::TabHandler;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

/// Could do recursive enums, but I will do the UUID way
///
/// REVIEW: abstract something about the pattern where items are represented by Uuids and stored in B-maps
/// Maybe like ID map
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Tile {
    Container {
        // parent_tile: Option<Id<Tile>>,
        children: [Id<Tile>; 2],
        orientation: Orientation,
    },
    /// Leaf node points to a full window
    Tab {
        /// REVIEW: adding onto the id map idea, I want to be able to specify this is a Uuid pointing to Tab
        tab_id: Id<TabHandler>,
    },
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
        };
    }

    pub fn remove(&mut self, _tab_id: Id<TabHandler>) {
        todo!()
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
}
