use super::ManagerHandler;
use crate::utils::tree::tree_node_path::TreeNodePath;

pub type DisplayBuffer = Vec<ratatui::buffer::Cell>;

pub enum Event {
    TUIEvent(ratatui::crossterm::event::Event),
    Resize(ratatui::layout::Rect),
    /// TODO: close forcibly
    Close,
}

pub enum Request {
    ChangeName(String),
    /// FIXME: Box<dyn TabCreator> didn't work for some reason, get it to work
    SpawnChildTab(Box<dyn FnOnce(ManagerHandler) + Send>),
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
