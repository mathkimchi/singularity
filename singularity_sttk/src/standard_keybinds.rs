use singularity_common::{
    sap::{detached::Axis, packets::StandardRequest},
    utils::tree::{
        tree_node_path::TreeTraverseOperation,
        world_tree::world_tree_traversal::WorldTreeTraversalOperation,
    },
};
use sonamu_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};

use crate::{
    command_hub::CommandHubApplet,
    nodular_applet::{AppletContext, NodeRequest, project_applet::ProjectApplet},
};

/// Handle the standard keybinds like:
/// - `Ctrl+Shift+Q` - Quit
/// - `Super+W` - Close tab
/// - `Super+T` / `Super+N` - Start foot / qutebrowser
/// - `Ctrl+Shift+Plus` - Add child
/// - `Ctrl+Shift+|` / `Ctrl+Shift+_` - Split side by side / stacked
/// - `Super+P` - Put the node in a project
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
    standard_keybind_request(ui_event)
        .map(|request| hook.request(request))
        .is_some()
}

/// The request a standard keybind makes to the applet's node, if `ui_event` is one.
/// For applets that handle some keybinds differently (eg `WlSurfaceApplet`'s Super+W).
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
        modifiers: KeyModifiers { logo: true, .. },
        pressed: true,
        ..
    } = &ui_event
        && matches!(key.to_char(), Some('p' | 'P'))
    {
        // Super+P -> put the node and everything under it in a project
        Some(NodeRequest::Wrap(ProjectApplet::wrapper()))
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers { logo: true, .. },
        pressed: true,
        ..
    } = &ui_event
        && key.to_char() == Some('w')
    {
        // Super+W -> close this tab, its children move to the parent
        Some(NodeRequest::Close)
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers: KeyModifiers { logo: true, .. },
        pressed: true,
        ..
    } = &ui_event
        && let Some(program) = match key.to_char() {
            Some('t') => Some("foot"),
            Some('n') => Some("qutebrowser"),
            _ => None,
        }
    {
        // Super+T / Super+N -> start foot / qutebrowser
        Some(NodeRequest::Run(program))
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
        modifiers: KeyModifiers::CTRL_SHIFT,
        pressed: true,
        ..
    } = &ui_event
        && let Some(axis) = match key.to_char() {
            Some('|') => Some(Axis::Horizontal),
            Some('_') => Some(Axis::Vertical),
            _ => None,
        }
    {
        // Ctrl+Shift+| / Ctrl+Shift+_ -> split with a CommandHub beside / below
        Some(NodeRequest::Split {
            axis,
            new_pane: CommandHubApplet::get_boxed_initiator(),
        })
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
    } else if let UIEvent::Key {
        symbol: Some(key),
        modifiers,
        pressed: true,
        ..
    } = &ui_event
        && modifiers.alt
        && !modifiers.ctrl
        && !modifiers.logo
        && key.to_char() == Some('\t')
    {
        // Alt+Tab / Alt+Shift+Tab -> next / prev node in tree order
        Some(NodeRequest::ChangeFocus(
            WorldTreeTraversalOperation::Layerwise(if modifiers.shift {
                TreeTraverseOperation::BfsPrev
            } else {
                TreeTraverseOperation::BfsNext
            }),
        ))
    } else {
        None
    }
}
