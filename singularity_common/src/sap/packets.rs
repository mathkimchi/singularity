use crate::{sap::detached::DetachedTree, utils::tree::world_tree::WorldTreePath};
use slotmap::KeyData;
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_event::{MouseEvent, UIEvent},
};

/// A piece of client state the server reads through a `SharedState`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateKind {
    /// Currently the whole surface
    Surface,
    Treeview,
}

/// Client to server
#[derive(Debug)]
pub enum StandardRequest {
    /// The client published a new value of this state
    Damage(StateKind),
    Quit,
    /// The wl surface Sonamu focus is on (`None` if it's on something else), sent when it changes.
    /// Keyboard (and so clipboard and text input) focus follows it.
    WlKeyboardFocus(Option<WlSurfaceId>),
    /// Answers `StandardEvent::Pluck`; `None` if nothing there can be plucked
    Plucked(Option<DetachedTree>),
    /// Answers `StandardEvent::Place`; gives the tree back if it wasn't placed
    Placed(Option<DetachedTree>),
    /// Asks the server to pluck at this path of the client's treeview (eg a drag started there)
    RequestPluck(WorldTreePath),
    /// Asks the server to place what it plucked at this path of the client's treeview
    RequestPlace(WorldTreePath),
}

slotmap::new_key_type! {
    pub struct WlSurfaceId;
}
impl WlSurfaceId {
    pub fn as_u64(self) -> u64 {
        self.0.as_ffi()
    }

    /// Not doing `impl From ...` bc I'm lazy and it lowkey looks ugly
    pub fn from_u64(data: u64) -> Self {
        Self(KeyData::from_ffi(data))
    }
}

/// Server to client
#[derive(Debug)]
pub enum StandardEvent {
    UIEvent(UIEvent),
    FocusChanged(bool),
    Highlighted(bool),
    // // Is a UI event already
    // Resize(DisplayContainerSize),
    CloseRequest,
    /// The server read this state, so the client may publish a new value
    DamageAck(StateKind),

    /// NOTE: ts is kinda niche; dk if it belongs here
    /// Currently doing by surface instead of by wl client
    WlSurfaceRegistered {
        surface_id: WlSurfaceId,
        /// REVIEW: this field prevents standard event from being Copy, I feel like I should avoid that
        wl_event_queue: calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
        // surface_updated: calloop::channel::Channel<()>,
    },
    /// Focus whichever applet shows this surface (eg for xdg-activation).
    /// Sent from the root down; applets that don't contain it ignore it.
    FocusWlSurface(WlSurfaceId),
    /// The surface's toplevel is gone (eg the client disconnected); its applet should close.
    /// Sent from the root down like `FocusWlSurface`.
    WlSurfaceDestroyed(WlSurfaceId),
    /// Take out the closest pluckable element at this path of the client's treeview
    /// (`None`: at its focus), and answer with `StandardRequest::Plucked`
    Pluck(Option<WorldTreePath>),
    /// Take ownership of the tree at this path (`None`: at its focus),
    /// and answer with `StandardRequest::Placed`
    Place(Option<WorldTreePath>, DetachedTree),
}

/// Wl surface applet to compositor
#[derive(Clone, Copy, Debug)]
pub enum WlSurfaceEvent {
    /// `keycode` is an xkb keycode (evdev + 8)
    Key {
        keycode: u32,
        pressed: bool,
    },
    Resize(DisplayContainerSize),
    Mouse(MouseEvent),
    /// Asks the client to close the toplevel; its tile closes once the toplevel is destroyed
    Close,
}
