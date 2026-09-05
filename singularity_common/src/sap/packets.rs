use slotmap::KeyData;
use sonamu_ui::ui_event::UIEvent;

/// Client to server
#[derive(Clone, Copy, Debug)]
pub enum StandardRequest {
    /// Currently does the whole surface
    DamageSurface,
    DamageTreeview,
    Quit,
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
#[derive(Clone, Debug)]
pub enum StandardEvent {
    UIEvent(UIEvent),
    FocusChanged(bool),
    Highlighted(bool),
    // // Is a UI event already
    // Resize(DisplayContainerSize),
    CloseRequest,
    SurfaceDamageAck,
    TreeviewDamageAck,

    /// NOTE: ts is kinda niche; dk if it belongs here
    /// Currently doing by surface instead of by wl client
    WlSurfaceRegistered {
        surface_id: WlSurfaceId,
        /// REVIEW: this field prevents standard event from being Copy, I feel like I should avoid that
        key_event_queue: calloop::channel::Sender<(WlSurfaceId, u32)>,
    },
}
