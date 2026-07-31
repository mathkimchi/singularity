use sonamu_ui::ui_event::UIEvent;

/// Client to server
#[derive(Clone, Copy, Debug)]
pub enum StandardRequest {
    /// Currently does the whole surface
    DamageSurface,
    DamageTreeview,
    Quit,
}

/// Server to client
#[derive(Clone, Copy, Debug)]
pub enum StandardEvent {
    UIEvent(UIEvent),
    Focus,
    Unfocus,
    // // Is a UI event already
    // Resize(DisplayContainerSize),
    CloseRequest,
    SurfaceDamageAck,
    TreeviewDamageAck,
}
