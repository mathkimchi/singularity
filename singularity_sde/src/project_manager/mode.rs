use crate::tab::TabHandler;
use singularity_common::utils::tree::{id_tree::IdTree, tree_node_path::TreeNodePath};

#[derive(Debug, Clone)]
pub enum Mode {
    /// Focused on some app
    TabFocus,
    /// If ChoosingFocus, there should be a special window app focuser
    ChoosingFocus {
        focusing_index: TreeNodePath,
        plucked: Option<IdTree<TabHandler>>,
    },
    CommandPalette {
        /// The string that the user has typed so far.
        /// TODO: make this use Textbox component to support cursor and stuff without duplicate code
        command_buffer: String,
    },
}
impl Mode {
    pub fn try_as_choosing_focus(&self) -> Option<(&TreeNodePath, &Option<IdTree<TabHandler>>)> {
        match self {
            Mode::ChoosingFocus {
                focusing_index,
                plucked,
            } => Some((focusing_index, plucked)),
            _ => None,
        }
    }

    pub fn try_as_choosing_focus_mut(
        &mut self,
    ) -> Option<(&mut TreeNodePath, &mut Option<IdTree<TabHandler>>)> {
        match self {
            Mode::ChoosingFocus {
                focusing_index,
                plucked,
            } => Some((focusing_index, plucked)),
            _ => None,
        }
    }

    pub fn try_get_focusing_index(&self) -> Option<&TreeNodePath> {
        match self {
            Mode::ChoosingFocus {
                focusing_index,
                plucked: _,
            } => Some(focusing_index),
            _ => None,
        }
    }
}

// /// Look at devlog 2025/03/03
// pub enum Actions {
//     Quit,
//     TreeTraverse(),
// }
