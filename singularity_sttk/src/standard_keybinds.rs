use singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation;
use sonamu_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};

use crate::{
    command_hub::CommandHubApplet,
    nodular_applet::{NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet},
};

/// Handle the standard keybinds like:
/// - `Ctrl+Shift+Q` - Quit
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
pub fn handle_standard_keybinds(ui_event: &UIEvent, hook: &impl NodularRunnerHook) -> bool {
    if let UIEvent::KeyPress(key, KeyModifiers::CTRL_SHIFT) = &ui_event
        && key.to_char() == Some('Q')
    {
        // Ctrl+Shift+Q -> Quit
        hook.close();

        true
    } else if let UIEvent::KeyPress(key, KeyModifiers::CTRL_SHIFT) = &ui_event
        && key.to_char() == Some('+')
    {
        // Ctrl+Shift+Plus -> spawn CommandHub as child
        hook.add_child(RecursiveNodeApplet::boxed_get_boxed_initializer(
            CommandHubApplet::get_boxed_initiator(),
        ));

        true
    } else if let UIEvent::KeyPress(key, KeyModifiers::ALT) = &ui_event
        && key.to_char() == Some('q')
    {
        // Alt+q -> exit focus
        // NOTE: this might interfere with applets that have its own hierarchy
        hook.change_focus(WorldTreeTraversalOperation::PrevLayer);

        true
    } else {
        false
    }
}
