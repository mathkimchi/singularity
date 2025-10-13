//! REVIEW: I am not sure where this belongs
//! it is implemenented in SDE but used in Ratk of STTK

use crate::{Todo, sap::packets::StandardRequest};

/// This represents the server on the applet side.
pub trait ServerHandler {
    /// Called by Applet, implemented by server (SDE).
    fn query(&mut self, query: Todo) -> Todo;

    /// Called by Applet, implemented by server (SDE).
    fn request(&mut self, request: StandardRequest);
}
