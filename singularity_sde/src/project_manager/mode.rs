use crate::tab::TabHandler;
use singularity_common::utils::tree::{
    id_tree::IdTree,
    tree_node_path::{TreeNodePath, TreeTraverseOperation},
};
use singularity_ui::ui_event::{Key, KeyModifiers};

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

/// Look at devlog 2025/03/03
pub enum Actions {
    // SECTION - closing

    //
    /// Ctrl+q quits
    Quit,
    /// Ctrl+Shift+W recursively closes focused tab and children
    RecursivelyCloseFocusedTab,

    // SECTION - tab hierarchy operations

    //
    /// Alt+Enter from ChoosingFocus mode
    /// REVIEW: Rename to select?
    ChooseFocus,
    /// Alt+Enter from NOT ChoosingFocus
    OpenFocusChooser,
    /// Alt+TreeTraverseKey
    TraverseTabTree(TreeTraverseOperation),
    /// Alt+Windows+TreeTraverseKey swaps position of focused and what would be the new focused
    TreeSwapTraverse(TreeTraverseOperation),
    /// Alt+Windows+P
    /// I am fine with this technically being two different things to do but one action
    PluckPlace,
    /// Alt+Windows+Enter swaps actually focused and focusing
    TreeSwap,

    // SECTION - command palette

    //
    /// Ctrl+Shift+P opens command palette (see: https://github.com/mathkimchi/singularity/issues/11)
    OpenCommandPalette,

    // SECTION - tiling operations

    //
    // /// Alt+ArrowUp maximizes focused tab
    // MaximizeFocused,
    // /// Alt+ArrowDown minimizes focused tab
    // MinimizeFocused,
    // /// Logo+"=" (represents "+") increments title split
    // IncrementTileSplit,
    /// Logo+t transposes selected tile's container (hor<=>vertical)
    TransposeTileParent,
    /// Logo+s swaps selected tile's siblings
    SwapTileSiblings,

    // SECTION - misc

    //
    /// Key press isn't any of the keyboard actions; forward it to focused
    ForwardKeyPress(Key, KeyModifiers),
    /// Resized. Currently ignore.
    WindowResized,
    /// Mouse press
    MousePress([[u32; 2]; 2]),
}
