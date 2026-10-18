//! What a session is saved as, to be restored on the next start (see `.claude/SESSIONS.md`)

use crate::{sap::detached::Layout, utils::tree::tree_node_path::TreeNodePath};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The whole tree, and where focus was
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub tree: SavedTree,
    /// A `WorldTreePath` of the tree
    #[serde(default)]
    pub focus: Vec<TreeNodePath>,
}

/// A node, like a `DetachedTree` but saved
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedTree {
    /// `None` for containers and splits
    pub main: Option<SavedApplet>,
    #[serde(default)]
    pub layout: Layout,
    #[serde(default)]
    pub children: Vec<Self>,
    /// A container's name; only a project's inner root has one (the project's name)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// How to make a node's main again
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SavedApplet {
    /// Made by the registered applet spawner with this name, from these args (like the command hub does)
    Spawner { name: String, args: Vec<String> },
    /// A project holding this tree as its inner tree
    Project(Box<SavedTree>),
    /// A wl client's toplevel
    Wl(SavedWl),
}

/// A wl client's toplevel, to start the client again
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedWl {
    /// The client's command line
    pub argv: Vec<String>,
    /// Where it was working: its first child's (eg a terminal's shell), else its own
    pub cwd: Option<PathBuf>,
    pub app_id: Option<String>,
    pub title: String,
}
impl SavedWl {
    /// A url at the end of the title after ` | ` (eg qutebrowser with `window.title_format = '{current_title} | {current_url}'`)
    pub fn url(&self) -> Option<&str> {
        let (_, url) = self.title.rsplit_once(" | ")?;
        url.contains("://").then_some(url)
    }
}
