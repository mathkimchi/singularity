pub mod ipc_subapp_handler;

pub type Request = String;
pub type Event<'a> = &'a [u8];

/// Represents subapp on the manager side
/// For those who have worked with multiple client handling,
/// the manager is like the server and the subapps are like clients.
///
/// A probably more accurate analogy is that the manager is like
/// a window manager and subapps are like windows, but I don't know much
/// about this.
/// I am basing the communication data roughly on [X window system's protocols](https://en.wikipedia.org/wiki/X_Window_System_core_protocol)
///
/// REVIEW: move this to manager package?
///
/// The standard understanding of mutability will not be
/// applicable to subapp handler.
///
/// The subapp handler should probably manually
/// reimplement drop.
pub trait SubappHandler {
    fn inform_event(&mut self, event: Event);

    /// This should not wait until there is a message.
    fn dump_requests(&mut self) -> Vec<Request>;
}
