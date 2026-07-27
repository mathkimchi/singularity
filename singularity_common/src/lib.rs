//! For things that are used by both the manager and subapps.

pub mod sap;
/// REVIEW: this was path of least resistance. actually organize later
pub use sonamu_sync as sync;
pub mod utils;

/// Placeholder type: <https://www.reddit.com/r/rust/comments/1jbpfsh/todo_type>/
#[deprecated]
pub enum Todo {}
