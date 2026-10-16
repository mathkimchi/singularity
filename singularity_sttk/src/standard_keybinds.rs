use singularity_common::{
    sap::packets::StandardRequest,
    utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation,
};
use sonamu_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};

use crate::{
    command_hub::CommandHubApplet,
    nodular_applet::{AppletContext, NodeRequest},
};

/// Handle the standard keybinds like:
/// - `Ctrl+Shift+Q` - Quit
/// - `Ctrl+W` - Close tab
/// - `Ctrl+Shift+Plus` - Add child
/// - `Alt+<traversal key>` - traversal
///
/// Returns whether or not the keypress was used.
///
/// NOTE: I could technically just take in keypress,
/// but this is fine.
///
/// TODO: have an enum or something more generic for a keybind system
/// (could use abstraction similar to what I did with packets).
pub fn handle_standard_keybinds(ui_event: &UIEvent, hook: &AppletContext) -> bool {
    match standard_keybind_request(ui_event) {
        Some(NodeRequest::AddChild(initializer)) => hook.add_child(initializer),
        Some(NodeRequest::ChangeFocus(operation)) => hook.change_focus(operation),
        Some(NodeRequest::Close) => hook.close(),
        Some(NodeRequest::Standard(StandardRequest::Quit)) => hook.quit(),
        Some(NodeRequest::Standard(_)) => unreachable!(),
        None => return false,
    }
    true
}

/// The request a standard keybind makes to the applet's node, if `ui_event` is one.
/// Nodes use this directly for mains that aren't applets (wl surfaces).
pub fn standard_keybind_request(ui_event: &UIEvent) -> Option<NodeRequest> {
    if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers::CTRL_SHIFT,
        pressed: true,
        ..
    } = &ui_event
        && key.to_char() == Some('Q')
    {
        // Ctrl+Shift+Q -> Quit
        Some(StandardRequest::Quit.into())
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers::CTRL,
        pressed: true,
        ..
    } = &ui_event
        && key.to_char() == Some('w')
    {
        // Ctrl+W -> close this tab, its children move to the parent
        Some(NodeRequest::Close)
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers::CTRL_SHIFT,
        pressed: true,
        ..
    } = &ui_event
        && key.to_char() == Some('+')
    {
        // Ctrl+Shift+Plus -> spawn CommandHub as child
        Some(NodeRequest::AddChild(
            CommandHubApplet::get_boxed_initiator(),
        ))
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers::ALT,
        pressed: true,
        ..
    } = &ui_event
        && key.to_char() == Some('q')
    {
        // Alt+q -> exit focus
        // NOTE: this might interfere with applets that have its own hierarchy
        Some(NodeRequest::ChangeFocus(
            WorldTreeTraversalOperation::PrevLayer,
        ))
    } else {
        None
    }
}
