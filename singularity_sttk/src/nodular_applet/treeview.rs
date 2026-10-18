//! The root's treeview: an outline of the tree that can be clicked (focus) and dragged (pluck and place).
//!
//! Rows are placed at a fixed height, so the root can hit-test them itself.

use super::node_tree::INACTIVE_MARK;
use singularity_common::{
    sap::packets::SpawnPosition,
    utils::tree::{
        tree_node_path::{TraversableTree, TreeNodePath},
        world_tree::{WorldTree, WorldTreePath},
    },
};
use sonamu_ui::{
    color::Color,
    ui_element::{Cursor, CursorIcon, FONT_SIZE_F, Length, Style, UIElement, UITree},
    ui_event::{MouseEvent, MouseEventKind},
};

const ROW_HEIGHT: f32 = FONT_SIZE_F + 14.0;
const TOP_PAD: f32 = 8.0;
const LEFT_PAD: f32 = 16.0;
const INDENT: f32 = 32.0;
const MARKER_WIDTH: f32 = 6.0;
/// The clickable width of a row's `▾`/`▸`, from the start of its label
const TRIANGLE_WIDTH: f32 = FONT_SIZE_F;
/// How far the pointer moves while pressed before it's a drag
const DRAG_THRESHOLD: f64 = 12.0;
const BTN_LEFT: u32 = 0x110;
/// Rows per scroll step
const SCROLL_STEP: usize = 3;

/// The alpha of everything in the treeview (about 80%), so what is behind it shows through
const OPACITY: u8 = 0xCC;

/// Also for the command hub's and project's backgrounds
pub(crate) const fn with_opacity(Color([r, g, b, _]): Color) -> Color {
    Color([r, g, b, OPACITY])
}

// Rect colors are drawn as linear (text colors aren't), so these are darker than they look:
// they show as about #17191e, #262a33, #24402f, #3a3f4a
const PANEL_BG: Color = with_opacity(Color([2, 2, 3, 0]));
const HOVER_BG: Color = with_opacity(Color([5, 6, 8, 0]));
const FOCUS_BG: Color = with_opacity(Color([3, 12, 7, 0]));
const GUIDE: Color = with_opacity(Color([11, 13, 17, 0]));
/// An expanded inner tree's box
const INNER_BOX_BG: Color = with_opacity(Color([3, 3, 5, 0]));
const INNER_BOX_BORDER: Color = GUIDE;
const TEXT: Color = with_opacity(Color::LIGHT_GRAY);
const INACTIVE_TEXT: Color = with_opacity(Color([0x70, 0x75, 0x80, 0]));
const DRAGGED_TEXT: Color = with_opacity(Color([0x55, 0x5A, 0x64, 0]));
/// Focus is on the node itself (traversal keys move it)
const MARKER_FOCUSING: Color = with_opacity(Color::LIGHT_YELLOW);
/// Focus is inside the node's applet (keys go to it)
const MARKER_INNER: Color = with_opacity(Color::LIGHT_GREEN);
const DROP_TARGET: Color = with_opacity(Color::CYAN);
/// The share of a row's height at its top and at its bottom that drops between siblings
const EDGE_ZONE: f32 = 0.25;
const INSERT_LINE_HEIGHT: f32 = 3.0;

pub struct TreeviewRow {
    pub path: WorldTreePath,
    depth: usize,
    label: String,
    /// Inside an applet's inner tree, rather than a node (or the view root's parent)
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
    push_rows(treeview, &[], 0, false, 0, &mut rows);
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

/// When focus is inside an applet's inner tree (eg `[[2], [0]]`), the rows of the deepest one it is in,
/// under a first row for the node holding it (focusing it goes back out).
/// `treeview` is the whole tree.
pub fn inner_layer_rows(
    treeview: &WorldTree<String>,
    focus_path: &WorldTreePath,
) -> Option<Vec<TreeviewRow>> {
    (1..focus_path.0.len()).rev().find_map(|layer| {
        let holder = WorldTreePath(focus_path.0[..layer].into());
        let world = treeview.safe_get(&holder)?;
        let WorldTree::World(root) = world else {
            return None;
        };
        // eg a terminal's label, or a collapsed node
        root.safe_get(&TreeNodePath(vec![0]))?;
        let label = world.get_root_value().trim_start_matches("▾ ");
        let mut rows = vec![TreeviewRow {
            label: format!("‹ {label}"),
            path: holder.clone(),
            depth: 0,
            inner: true,
        }];
        push_rows(world, &holder.0, 0, true, layer, &mut rows);
        Some(rows)
    })
}

/// `skip_root`: the world's root is already the row above (an applet's inner tree root is the applet itself).
/// `layer`: how many worlds deep the view is; rows deeper than it are `inner`.
fn push_rows(
    world: &WorldTree<String>,
    prefix: &[TreeNodePath],
    depth: usize,
    skip_root: bool,
    layer: usize,
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
            inner: prefix.len() > layer,
        });
        push_rows(value, &full_path, row_depth, true, layer, rows);
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
    /// Where in the hovered row the pointer is
    zone: DropZone,
    press: Option<Press>,
    /// The first row shown
    scroll: usize,
    /// The focused row last scrolled to, so it is scrolled to again only when focus moves
    followed: Option<WorldTreePath>,
}

/// Where in a row the pointer is: the middle appends a child, the edges insert a sibling
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum DropZone {
    Top,
    #[default]
    Middle,
    Bottom,
}

/// Where a drop on `row` goes: the path of the parent, and the position among its children.
/// Only nodes of the tree have siblings, so anything else (eg an applet's inner tree) is appended to.
fn drop_target(row: &TreeviewRow, zone: DropZone) -> (WorldTreePath, SpawnPosition) {
    let append = || (row.path.clone(), SpawnPosition::AppendChild);
    let [node] = &*row.path.0 else {
        return append();
    };
    let Some((&index, parent)) = node.0.split_last().filter(|_| !row.inner) else {
        return append();
    };
    let child_index = match zone {
        DropZone::Middle => return append(),
        DropZone::Top => index,
        DropZone::Bottom => index + 1,
    };
    (
        WorldTreePath(Box::new([TreeNodePath(parent.to_vec())])),
        SpawnPosition::InsertChild { child_index },
    )
}

struct Press {
    path: WorldTreePath,
    position: [f64; 2],
    dragging: bool,
    /// On the row's `▾`/`▸`, so a click collapses or expands it instead of focusing it
    on_triangle: bool,
}

pub enum TreeviewAction {
    Focus(WorldTreePath),
    /// Its `▾`/`▸` was clicked
    ToggleCollapsed(WorldTreePath),
    /// Pluck at this path
    StartDrag(WorldTreePath),
    /// Place at this path, or at the focus if dropped outside the treeview
    Drop(Option<(WorldTreePath, SpawnPosition)>),
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
        height: f32,
    ) -> TreeviewResponse {
        let [_, y] = event.position;
        // non-negative, since `y >= TOP_PAD`
        #[allow(clippy::cast_sign_loss)]
        let row = (in_treeview && y >= f64::from(TOP_PAD))
            .then(|| self.scroll + ((y - f64::from(TOP_PAD)) / f64::from(ROW_HEIGHT)) as usize)
            .filter(|&index| index < rows.len());

        let within = ((y - f64::from(TOP_PAD)) / f64::from(ROW_HEIGHT)).fract() as f32;
        let zone = if row.is_none() || (EDGE_ZONE..=1.0 - EDGE_ZONE).contains(&within) {
            DropZone::Middle
        } else if within < EDGE_ZONE {
            DropZone::Top
        } else {
            DropZone::Bottom
        };
        let mut damaged = std::mem::replace(&mut self.hovered, row) != row;
        damaged |= std::mem::replace(&mut self.zone, zone) != zone;
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
                    let row = &rows[index];
                    let text_left = f64::from(LEFT_PAD + row.depth as f32 * INDENT);
                    self.press = Some(Press {
                        path: row.path.clone(),
                        position: event.position,
                        dragging: false,
                        on_triangle: has_triangle(row)
                            && (text_left..text_left + f64::from(TRIANGLE_WIDTH))
                                .contains(&event.position[0]),
                    });
                }
            }
            MouseEventKind::Button {
                button: BTN_LEFT,
                pressed: false,
            } => {
                if let Some(press) = self.press.take() {
                    action = if press.dragging {
                        Some(TreeviewAction::Drop(
                            row.map(|index| drop_target(&rows[index], zone)),
                        ))
                    } else {
                        // a click: pressed and released on the same row
                        let target = row.map(|index| rows[index].path.clone());
                        target.filter(|target| *target == press.path).map(|target| {
                            if press.on_triangle {
                                TreeviewAction::ToggleCollapsed(target)
                            } else {
                                TreeviewAction::Focus(target)
                            }
                        })
                    };
                    damaged = true;
                }
            }
            MouseEventKind::Scroll { delta, .. } if in_treeview => {
                let scroll = if delta[1] > 0.0 {
                    self.scroll + SCROLL_STEP
                } else {
                    self.scroll.saturating_sub(SCROLL_STEP)
                }
                .min(max_scroll(rows.len(), height));
                damaged |= std::mem::replace(&mut self.scroll, scroll) != scroll;
            }
            MouseEventKind::Button { .. } | MouseEventKind::Scroll { .. } => {}
        }

        TreeviewResponse {
            action,
            consumed,
            damaged,
        }
    }

    /// Scrolls to the focused row if focus moved there since the last render
    pub fn render(
        &mut self,
        rows: &[TreeviewRow],
        [width, height]: [f32; 2],
        focus_path: &WorldTreePath,
    ) -> UITree {
        let focused_row = focused_row(rows, focus_path);
        let focus_inner = is_inside_element(focus_path);

        if self.followed.as_ref() != Some(&focused_row) {
            if let Some(index) = rows.iter().position(|row| row.path == focused_row) {
                let shown = shown_rows(height);
                self.scroll = self.scroll.clamp((index + 1).saturating_sub(shown), index);
            }
            self.followed = Some(focused_row.clone());
        }
        self.scroll = self.scroll.min(max_scroll(rows.len(), height));
        let scroll = self.scroll;
        let row_top = |index: usize| TOP_PAD + (index as f32 - scroll as f32) * ROW_HEIGHT;

        let dragging = self.press.as_ref().filter(|press| press.dragging);

        let mut children = inner_box_trees(rows, width, row_top);

        children.extend(rows.iter().enumerate().skip(scroll).map(|(index, row)| {
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
            } else if is_inactive(row) {
                INACTIVE_TEXT
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
            if has_triangle(row) {
                parts.push(
                    UITree::leaf_styled(
                        UIElement::Nothing,
                        Style::absolute([text_left, 0.0], [TRIANGLE_WIDTH, ROW_HEIGHT]),
                    )
                    .with_cursor(Some(Cursor::Named(CursorIcon::Pointer))),
                );
            }

            let hovered_drop = dragging.is_some() && is_hovered && !is_dragged;
            if hovered_drop && drop_target(row, self.zone).1 != SpawnPosition::AppendChild {
                // a line between siblings
                let line_left = LEFT_PAD + row.depth as f32 * INDENT;
                let line_top = if self.zone == DropZone::Top {
                    0.0
                } else {
                    ROW_HEIGHT - INSERT_LINE_HEIGHT
                };
                parts.push(rect(
                    DROP_TARGET,
                    [line_left, line_top],
                    [(width - line_left - 4.0).max(0.0), INSERT_LINE_HEIGHT],
                ));
            } else if hovered_drop {
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
                Style::absolute([0.0, row_top(index)], [width, ROW_HEIGHT]),
                parts,
            )
        }));

        UITree::new_styled(
            UIElement::Backgrounded(PANEL_BG),
            Style::full_size(),
            children,
        )
    }
}

/// A box behind each shown inner tree's rows. `row_top` is where a row is drawn.
fn inner_box_trees(
    rows: &[TreeviewRow],
    width: f32,
    row_top: impl Fn(usize) -> f32,
) -> Vec<UITree> {
    inner_boxes(rows)
        .flat_map(|(holder, end)| {
            let left = LEFT_PAD + rows[holder].depth as f32 * INDENT + INDENT / 2.0 - 1.0;
            let position = [left, row_top(holder + 1)];
            let size = [
                (width - left - 4.0).max(0.0),
                (end - holder) as f32 * ROW_HEIGHT,
            ];
            [
                rect(INNER_BOX_BG, position, size),
                UITree::leaf_styled(
                    UIElement::Bordered(INNER_BOX_BORDER),
                    Style {
                        border: Length::uniform_rect(Length::Px(2.0)),
                        ..Style::absolute(position, size)
                    },
                ),
            ]
        })
        .collect()
}

/// Its node is inactive (see `INACTIVE_MARK`)
fn is_inactive(row: &TreeviewRow) -> bool {
    row.label
        .trim_start_matches(['▾', '▸', ' '])
        .starts_with(INACTIVE_MARK.trim_end())
}

/// Its label starts with a `▾`/`▸` that collapses or expands it when clicked
fn has_triangle(row: &TreeviewRow) -> bool {
    row.label.starts_with(['▾', '▸'])
}

/// How many rows fit
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn shown_rows(height: f32) -> usize {
    (((height - TOP_PAD) / ROW_HEIGHT).floor() as usize).max(1)
}

/// The furthest the rows can scroll: until the last one is at the bottom
fn max_scroll(row_count: usize, height: f32) -> usize {
    row_count.saturating_sub(shown_rows(height))
}

/// Each shown inner tree: the row holding it, and the last of the rows of it (which directly follow)
fn inner_boxes(rows: &[TreeviewRow]) -> impl Iterator<Item = (usize, usize)> {
    rows.iter().enumerate().filter_map(|(holder, row)| {
        let len = rows[holder + 1..]
            .iter()
            .take_while(|inner| {
                inner.inner
                    && inner.path.0.len() > row.path.0.len()
                    && inner.path.0.starts_with(&row.path.0)
            })
            .count();
        (len > 0).then_some((holder, holder + len))
    })
}

fn distance([x1, y1]: [f64; 2], [x2, y2]: [f64; 2]) -> f64 {
    (x2 - x1).hypot(y2 - y1)
}
