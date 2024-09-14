use crate::utils::tree::tree_node_path::TreeNodePath;

pub enum Event {
    KeyPress(char),
    /// TODO: close forcibly
    Close,
}

pub enum Request {
    ChangeName(String),
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
