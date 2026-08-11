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

/// Server to client
#[derive(Clone, Copy, Debug)]
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
    },
}
