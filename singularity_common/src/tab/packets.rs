use serde::Deserialize;

use crate::utils::tree::tree_node_path::TreeNodePath;

pub enum Event {
    KeyPress(char),
    /// TODO: close forcibly
    Close,
}

pub enum Request {
    ChangeName(String),
}

pub trait Query: Send {
    /// TODO: learn `for<'a>` notation
    type Response: for<'a> Deserialize<'a>;
}
pub struct PathQuery {}
impl Query for PathQuery {
    type Response = TreeNodePath;
}
