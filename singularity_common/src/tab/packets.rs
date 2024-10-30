use super::TabCreator;
use crate::{project::project_settings::TabData, utils::tree::tree_node_path::TreeNodePath};
use singularity_ui::{display_units::DisplayArea, ui_event::UIEvent};

#[derive(Debug, Clone)]
pub enum Event {
    UIEvent(UIEvent),
    Resize(DisplayArea),
    /// TODO: close forcibly
    Close,
}

pub enum Request {
    ChangeName(String),
    SpawnChildTab(Box<dyn TabCreator>, TabData),
}

/// TODO: auto generate this with macro
pub enum Query {
    Path,
    Name,
    TabData,
}

#[derive(Debug)]
pub enum Response {
    Path(TreeNodePath),
    Name(String),
    TabData(TabData),
}
/// TODO: macro this
impl Response {
    pub fn try_as_path(self) -> Option<TreeNodePath> {
        if let Self::Path(path) = self {
            Some(path)
        } else {
            None
        }
    }
    pub fn try_as_name(self) -> Option<String> {
        if let Self::Name(name) = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn try_as_tab_data(self) -> Option<TabData> {
        if let Self::TabData(tab_data) = self {
            Some(tab_data)
        } else {
            None
        }
    }
}
