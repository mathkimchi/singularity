use crate::tab::TabHandler;
use singularity_common::utils::tree::{
    id_tree::IdTree,
    tree_node_path::{TreeNodePath, TreeTraverseOperation, TREE_TRAVERSE_KEYS},
};
use singularity_ui::{
    display_units::DisplayArea,
    ui_event::{Key, KeyModifiers, KeyTrait, UIEvent},
};

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
pub enum UserAction {
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
    ForwardKeyPressTab(Key, KeyModifiers),
    /// Key press isn't any of the keyboard actions; forward it to command palette
    ForwardKeyPressCommandPalette(Key, KeyModifiers),
    /// currently, the only None case is when non-shortcut is performed on tab choosing mode
    /// REVIEW: is this good? just use option?
    NoAction,
    /// Resized. Currently ignore.
    WindowResized,
    /// Mouse press
    MousePress([[u32; 2]; 2]),
}
impl UserAction {
    pub fn from_ui_event(curr_mode: &Mode, ui_event: UIEvent) -> Self {
        match (curr_mode, ui_event) {
            // Ctrl+Q
            (_, UIEvent::KeyPress(key, KeyModifiers::CTRL)) if key.to_char() == Some('q') => {
                Self::Quit
            }
            (
                Mode::TabFocus | Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(key, KeyModifiers::CTRL),
            ) if key.to_char() == Some('w') => Self::RecursivelyCloseFocusedTab,

            // Alt+Enter from ChoosingFocus mode
            (Mode::ChoosingFocus { .. }, UIEvent::KeyPress(key, KeyModifiers::ALT))
                if key.to_char() == Some('\n') =>
            {
                Self::ChooseFocus
            }
            // Alt+Enter from NOT ChoosingFocus
            // NOTE: this pattern must be behind choosing focus
            (_, UIEvent::KeyPress(key, KeyModifiers::ALT)) if key.to_char() == Some('\n') => {
                Self::OpenFocusChooser
            }
            // Alt+TreeTraverseKey
            (
                Mode::TabFocus | Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(key, KeyModifiers::ALT),
            ) if TREE_TRAVERSE_KEYS.contains(&key.to_char().unwrap_or(' ')) => {
                // `' '` is a placeholder for some key that isn't in tree traverse
                // sad that match doesn't support if let syntax
                Self::TraverseTabTree(
                    TreeTraverseOperation::from_char(key.to_char().unwrap()).unwrap(),
                )
            }
            // Alt+Windows+TreeTraverseKey swaps position of focused and what would be the new focused
            (
                Mode::TabFocus | Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: _,
                    },
                ),
            ) if TREE_TRAVERSE_KEYS.contains(&key.to_char().unwrap_or(' ')) => {
                // `' '` is a placeholder for some key that isn't in tree traverse
                // sad that match doesn't support if let syntax
                Self::TreeSwapTraverse(
                    TreeTraverseOperation::from_char(key.to_char().unwrap()).unwrap(),
                )
            }
            // Alt+Windows+P
            // I am fine with this technically being two different things to do but one action
            (
                Mode::TabFocus | Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: _,
                    },
                ),
            ) if key.to_char() == Some('p') => Self::PluckPlace,
            // Alt+Windows+Enter swaps actually focused and focusing
            (
                Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: _,
                    },
                ),
            ) if key.to_char() == Some('\n') => Self::TreeSwap,

            // Ctrl+Shift+P opens command palette (see: https://github.com/mathkimchi/singularity/issues/11)
            (
                Mode::TabFocus | Mode::ChoosingFocus { .. },
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: true,
                        caps_lock: false,
                        logo: false,
                        num_lock: _,
                    },
                ),
            ) if key.to_char() == Some('P') => Self::OpenCommandPalette,

            // Logo+t transposes selected tile's container (hor<=>vertical)
            (
                Mode::TabFocus,
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: false,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: _,
                    },
                ),
            ) if key.to_char() == Some('t') => Self::TransposeTileParent,
            // Logo+s swaps selected tile's siblings
            (
                Mode::TabFocus,
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: false,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: _,
                    },
                ),
            ) if key.to_char() == Some('s') => Self::SwapTileSiblings,

            // Key press isn't any of the keyboard actions; forward it to focused
            (Mode::TabFocus, UIEvent::KeyPress(key, modifiers)) => {
                Self::ForwardKeyPressTab(key, modifiers)
            }
            // Key press isn't any of the keyboard actions; forward it to command palette
            (Mode::CommandPalette { .. }, UIEvent::KeyPress(key, modifiers)) => {
                Self::ForwardKeyPressCommandPalette(key, modifiers)
            }
            // currently, the only None case is when non-shortcut is performed on tab choosing mode
            (Mode::ChoosingFocus { .. }, UIEvent::KeyPress(..)) => Self::NoAction,
            // Resized. Currently ignore.
            (_, UIEvent::WindowResized(_)) => Self::WindowResized,
            // Mouse press
            (_, UIEvent::MousePress(location, container)) => {
                assert_eq!(container, DisplayArea::FULL);
                Self::MousePress(location)
            }
        }
    }
}
