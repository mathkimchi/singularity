use std::sync::{Arc, Mutex};

pub mod executable_subapp_handler;

/// TODO
pub type Attributes = ();
/// TODO
pub type Query = ();
/// TODO
pub type Reply = ();
/// TODO
pub type Request = ();
/// TODO
pub type DisplayBuffer = ();
/// TODO
pub type Event = ();

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
    fn give_display_buffer(&self, display_buffer: &mut Arc<Mutex<DisplayBuffer>>);

    fn peek_display_buffer(&self) -> &Arc<Mutex<DisplayBuffer>>;

    fn inform_event(&self, event: Event);

    // /// wait until there is a request
    // fn get_request(&mut self) -> Request;

    /// This is better than having project manager wait for requests.
    /// If this function doesn't require waiting, then outside of the subapp handler,
    /// there should be no need for multithreading.
    fn dump_requests(&mut self) -> Vec<Request>;
}
