use sonamu_ui::{display_units::DisplayContainerSize, ui_event::UIEvent};

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
    Resize(DisplayContainerSize),
    CloseRequest,
    SurfaceDamageAck,
    TreeviewDamageAck,
}
