//! REVIEW:
//! Currently, I manage the paths locally - each applet's focus state determines
//! how to append to the path when it is built bottom-up.
//! So, each applet will have to implement the traversal logic.
//! But, if I managed the paths globally from the root holder
//! and made the paths determine who has focus as it is passed top to bottom,
//! there would actually be quite a few benefits:
//! - force every app to use the same keybindings (the promise of Singularity is to standardize)
//! - I'd only have to implement the traversal logic once
//! - it is safer bc if an applet with focus doesn't want to give back focus, it doesn't matter
//! - this might make sense with the focus object idea (#22)
//! - highlighting can be done by the world
//! - could kind of make a FocusedWorldTree object, where focus is stored alongside the value
//!
//! But there are also reasons to stick with the current method:
//! - it obeys the philosophy of having everything handled recursively
//! - it would allow for some unforseen usecase where the keybinds need to be different for some reason
//!
//! I think the ultimate question is:
//! Is Singularity willing to force standardization (with global traversal)
//! or does Singularity merely offer a standardized protocol and it is up to the applets
//! to go along with it?
//!
//! Philosophically, I'd prefer to offer standardization optionally,
//! because that is how I believe government and education should work.
//! But, Singularity is an opt-in system, so it's not that serious.
//! Since all this is open source, if an app doesn't follow the standard
//! without having a good reason, it can be forked to obey.
//! So, I guess I'll leave it recursive.

// // TODO:
// pub const TREE_TRAVERSE_KEYS: &[char] = &[
//     'w', 'a', 's', 'd', 'q', 'e', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
// ];

use crate::utils::tree::tree_node_path::TreeTraverseOperation;

#[derive(Debug, Clone, Copy)]
pub enum WorldTreeTraversalOperation {
    GlobalRoot,
    /// Out of a world
    PrevLayer,
    /// Into a world
    NextLayer,
    // LayerwiseDFSPrev,
    // LayerwiseDFSNext,
    // PrevSibling,
    // NextSibling,
    // Parent,
    // Child,
    Layerwise(TreeTraverseOperation),
}
