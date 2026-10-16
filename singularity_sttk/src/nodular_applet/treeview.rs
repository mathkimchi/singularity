//! The root's treeview: an outline of the tree that can be clicked (focus) and dragged (pluck and place).
//!
//! Rows are placed at a fixed height, so the root can hit-test them itself.

use singularity_common::utils::tree::{
    tree_node_path::{TraversableTree, TreeNodePath},
    world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    color::Color,
    ui_element::{FONT_SIZE_F, Length, Style, UIElement, UITree},
    ui_event::{MouseEvent, MouseEventKind},
};

const ROW_HEIGHT: f32 = FONT_SIZE_F + 14.0;
const TOP_PAD: f32 = 8.0;
const LEFT_PAD: f32 = 16.0;
const INDENT: f32 = 32.0;
const MARKER_WIDTH: f32 = 6.0;
/// How far the pointer moves while pressed before it's a drag
const DRAG_THRESHOLD: f64 = 12.0;
const BTN_LEFT: u32 = 0x110;

// Rect colors are drawn as linear (text colors aren't), so these are darker than they look:
// they show as about #17191e, #262a33, #24402f, #3a3f4a
const PANEL_BG: Color = Color([2, 2, 3, 0xFF]);
const HOVER_BG: Color = Color([5, 6, 8, 0xFF]);
const FOCUS_BG: Color = Color([3, 12, 7, 0xFF]);
const GUIDE: Color = Color([11, 13, 17, 0xFF]);
const TEXT: Color = Color::LIGHT_GRAY;
/// Elements inside an applet's inner tree
const INNER_TEXT: Color = Color([0x8A, 0x93, 0xA6, 0xFF]);
const DRAGGED_TEXT: Color = Color([0x55, 0x5A, 0x64, 0xFF]);
/// Focus is on the node itself (traversal keys move it)
const MARKER_FOCUSING: Color = Color::LIGHT_YELLOW;
/// Focus is inside the node's applet (keys go to it)
const MARKER_INNER: Color = Color::LIGHT_GREEN;
const DROP_TARGET: Color = Color::CYAN;

pub struct TreeviewRow {
    pub path: WorldTreePath,
    depth: usize,
    label: String,
    /// Inside an applet's inner tree, rather than a node (or the view root's parent); drawn dimmer
    inner: bool,
}

/// One row per node of the tree, and per element of each applet's inner tree, in DFS order.
///
/// `treeview` is the view root's subtree and `base` its path, so rows get paths from the real root.
/// `up` is the view root's parent (path and label): a dim first row, to widen the view by focusing it.
pub fn treeview_rows(
    treeview: &WorldTree<String>,
    base: &TreeNodePath,
    up: Option<(TreeNodePath, String)>,
) -> Vec<TreeviewRow> {
    let mut rows = Vec::new();
    push_rows(treeview, &[], 0, false, &mut rows);
    for row in &mut rows {
        row.path.0[0].0.splice(0..0, base.0.iter().copied());
    }
    if let Some((path, label)) = up {
        rows.insert(
            0,
            TreeviewRow {
                path: WorldTreePath(Box::new([path])),
                depth: 0,
                label: format!("‹ {label}"),
                inner: true,
            },
        );
    }
    rows
}

/// `skip_root`: the world's root is already the row above (an applet's inner tree root is the applet itself)
fn push_rows(
    world: &WorldTree<String>,
    prefix: &[TreeNodePath],
    depth: usize,
    skip_root: bool,
    rows: &mut Vec<TreeviewRow>,
) {
    let WorldTree::World(root) = world else {
        return;
    };
    for path in root.iter_paths_dfs() {
        if skip_root && path.is_root() {
            continue;
        }
        let value = &root[&path];
        let row_depth = depth + path.depth();
        let mut full_path = prefix.to_vec();
        full_path.push(path);
        rows.push(TreeviewRow {
            path: WorldTreePath(full_path.clone().into_boxed_slice()),
            depth: row_depth,
            label: value.get_root_value().clone(),
            inner: !prefix.is_empty(),
        });
        push_rows(value, &full_path, row_depth, true, rows);
    }
}

/// The path of the row showing what `path` points at (an applet's inner root is the applet's row)
fn row_path(path: &WorldTreePath) -> WorldTreePath {
    let mut path = path.0.to_vec();
    while path.len() > 1 && path.last().is_some_and(TreeNodePath::is_root) {
        path.pop();
    }
    if path.is_empty() {
        path.push(TreeNodePath::new_root());
    }
    WorldTreePath(path.into_boxed_slice())
}

/// The row showing the focus: its own row, or if that is hidden (inside a collapsed node), the row it is hidden in
fn focused_row(rows: &[TreeviewRow], focus_path: &WorldTreePath) -> WorldTreePath {
    let own = row_path(focus_path);
    if rows.iter().any(|row| row.path == own) {
        return own;
    }
    rows.iter()
        .filter(|row| focus_path.0.starts_with(&row.path.0))
        .max_by_key(|row| row.path.0.len())
        .map_or(own, |row| row.path.clone())
}

/// Whether the focus path goes past its row, into the element's inner world (eg `[[2, 1], []]` is inside `[[2, 1]]`)
fn is_inside_element(focus_path: &WorldTreePath) -> bool {
    focus_path.0.len() > row_path(focus_path).0.len()
}

fn rect(color: Color, position: [f32; 2], size: [f32; 2]) -> UITree {
    UITree::leaf_styled(
        UIElement::Backgrounded(color),
        Style::absolute(position, size),
    )
}

/// Mouse state over the treeview
#[derive(Default)]
pub struct TreeviewPointer {
    hovered: Option<usize>,
    press: Option<Press>,
}
struct Press {
    path: WorldTreePath,
    position: [f64; 2],
    dragging: bool,
}

pub enum TreeviewAction {
    Focus(WorldTreePath),
    /// Pluck at this path
    StartDrag(WorldTreePath),
    /// Place at this path, or at the focus if dropped outside the treeview
    Drop(Option<WorldTreePath>),
}

pub struct TreeviewResponse {
    pub action: Option<TreeviewAction>,
    /// The treeview used the event, so the applets shouldn't get it
    pub consumed: bool,
    /// The treeview looks different now
    pub damaged: bool,
}

impl TreeviewPointer {
    /// A press that started on a row hasn't been released yet
    pub const fn is_pressed(&self) -> bool {
        self.press.is_some()
    }

    /// `position` is relative to the treeview's top-left; `in_treeview` is whether it's over the treeview
    pub fn handle_mouse(
        &mut self,
        event: &MouseEvent,
        in_treeview: bool,
        rows: &[TreeviewRow],
    ) -> TreeviewResponse {
        let [_, y] = event.position;
        // non-negative, since `y >= TOP_PAD`
        #[allow(clippy::cast_sign_loss)]
        let row = (in_treeview && y >= f64::from(TOP_PAD))
            .then(|| ((y - f64::from(TOP_PAD)) / f64::from(ROW_HEIGHT)) as usize)
            .filter(|&index| index < rows.len());

        let mut damaged = std::mem::replace(&mut self.hovered, row) != row;
        let mut action = None;
        let consumed = in_treeview || self.press.is_some();

        match event.kind {
            MouseEventKind::Motion => {
                if let Some(press) = &mut self.press
                    && !press.dragging
                    && distance(press.position, event.position) > DRAG_THRESHOLD
                {
                    press.dragging = true;
                    action = Some(TreeviewAction::StartDrag(press.path.clone()));
                    damaged = true;
                }
            }
            MouseEventKind::Button {
                button: BTN_LEFT,
                pressed: true,
            } => {
                if let Some(index) = row {
                    self.press = Some(Press {
                        path: rows[index].path.clone(),
                        position: event.position,
                        dragging: false,
                    });
                }
            }
            MouseEventKind::Button {
                button: BTN_LEFT,
                pressed: false,
            } => {
                if let Some(press) = self.press.take() {
                    let target = row.map(|index| rows[index].path.clone());
                    action = if press.dragging {
                        Some(TreeviewAction::Drop(target))
                    } else {
                        // a click: pressed and released on the same row
                        target
                            .filter(|target| *target == press.path)
                            .map(TreeviewAction::Focus)
                    };
                    damaged = true;
                }
            }
            MouseEventKind::Button { .. } | MouseEventKind::Scroll { .. } => {}
        }

        TreeviewResponse {
            action,
            consumed,
            damaged,
        }
    }

    pub fn render(&self, rows: &[TreeviewRow], width: f32, focus_path: &WorldTreePath) -> UITree {
        let focused_row = focused_row(rows, focus_path);
        let focus_inner = is_inside_element(focus_path);
        let dragging = self.press.as_ref().filter(|press| press.dragging);

        let children = rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let is_focused = row.path == focused_row;
                let is_hovered = self.hovered == Some(index);
                let is_dragged = dragging.is_some_and(|press| press.path == row.path);

                let mut parts = Vec::new();
                for level in 0..row.depth {
                    parts.push(rect(
                        GUIDE,
                        [LEFT_PAD + level as f32 * INDENT + INDENT / 2.0 - 1.0, 0.0],
                        [2.0, ROW_HEIGHT],
                    ));
                }
                if is_focused {
                    let marker = if focus_inner {
                        MARKER_INNER
                    } else {
                        MARKER_FOCUSING
                    };
                    parts.push(rect(marker, [0.0, 0.0], [MARKER_WIDTH, ROW_HEIGHT]));
                }

                let text_left = LEFT_PAD + row.depth as f32 * INDENT;
                let text_top = (ROW_HEIGHT - FONT_SIZE_F) / 2.0;
                let text_color = if is_dragged {
                    DRAGGED_TEXT
                } else if row.inner {
                    INNER_TEXT
                } else {
                    TEXT
                };
                let label = if row.label.is_empty() {
                    "…".to_string()
                } else {
                    row.label.clone()
                };
                parts.push(UITree::leaf_styled(
                    UIElement::Text(vec![(label, UIElement::glyphon_attr(text_color))]),
                    Style::absolute(
                        [text_left, text_top],
                        [(width - text_left).max(0.0), ROW_HEIGHT - text_top],
                    ),
                ));

                if dragging.is_some() && is_hovered && !is_dragged {
                    parts.push(UITree::leaf_styled(
                        UIElement::Bordered(DROP_TARGET),
                        Style {
                            border: Length::uniform_rect(Length::Px(2.0)),
                            ..Style::absolute([2.0, 0.0], [(width - 4.0).max(0.0), ROW_HEIGHT])
                        },
                    ));
                }

                let background = if is_focused {
                    UIElement::Backgrounded(FOCUS_BG)
                } else if is_hovered && dragging.is_none() {
                    UIElement::Backgrounded(HOVER_BG)
                } else {
                    UIElement::Nothing
                };
                UITree::new_styled(
                    background,
                    Style::absolute([0.0, TOP_PAD + index as f32 * ROW_HEIGHT], [width, ROW_HEIGHT]),
                    parts,
                )
            })
            .collect();

        UITree::new_styled(UIElement::Backgrounded(PANEL_BG), Style::full_size(), children)
    }
}

fn distance([x1, y1]: [f64; 2], [x2, y2]: [f64; 2]) -> f64 {
    (x2 - x1).hypot(y2 - y1)
}
