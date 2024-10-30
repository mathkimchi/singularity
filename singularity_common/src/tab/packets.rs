use super::TabCreator;
use crate::utils::tree::tree_node_path::TreeNodePath;
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
    SpawnChildTab(Box<dyn TabCreator>),
}

/// TODO: auto generate this with macro
pub enum Query {
    Path,
    Name,
}

#[derive(Debug)]
pub enum Response {
    Path(TreeNodePath),
    Name(String),
}
