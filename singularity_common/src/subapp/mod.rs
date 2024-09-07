use serde::{Deserialize, Serialize};

pub mod temp_interface;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetName(String),
}
#[derive(Serialize, Deserialize)]
pub enum Event {
    KeyPressed { keycode: char },
}

/// Represents the subapp process on the manager side
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
/// applicable to subapp interface.
///
/// The subapp interface should probably manually
/// reimplement drop.
pub trait SubappInterface {
    fn inform_event(&mut self, event: Event);

    /// This should not wait until there is a message.
    fn dump_requests(&mut self) -> Vec<Request>;
}

pub struct Subapp {
    pub subapp_interface: Box<dyn SubappInterface>,

    pub subapp_title: String,
}
impl Subapp {
    pub fn new<S: 'static + SubappInterface>(subapp_interface: S) -> Self {
        Self::from_box(Box::new(subapp_interface))
    }

    pub fn from_box(subapp_interface: Box<dyn SubappInterface>) -> Self {
        Self {
            subapp_interface,
            subapp_title: String::new(),
        }
    }
}
