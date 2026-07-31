use sonamu_ui::{display_units::DisplayContainerSize, ui_event::UIEvent};

/// Client to server
pub enum StandardRequest {
    /// Currently does the whole surface
    DamageSurface,
    DamageTreeview,
}

/// Server to client
pub enum StandardEvent {
    UIEvent(UIEvent),
    Focus,
    Unfocus,
    Resize(DisplayContainerSize),
    CloseRequest,
    SurfaceDamageAck,
    TreeviewDamageAck,
}
