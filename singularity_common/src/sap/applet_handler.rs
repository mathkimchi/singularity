use crate::sap::{packets::StandardEvent, server_handler::ServerHandler};

/// Alternative name would be `AppletHandler`.
/// Implemented by Applet, used by SDE/server.
pub trait ReactiveApplet {
    fn initialize(server_handler: &mut impl ServerHandler) -> Self;

    fn handle_event(&mut self, event: StandardEvent, server_handler: &mut impl ServerHandler);
}
