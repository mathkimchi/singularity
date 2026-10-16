//! The server holds what was plucked until it's placed.

use crate::runner::AppletRunner;
use singularity_common::{
    sap::{detached::DetachedTree, packets::StandardEvent},
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    color::Color,
    ui_element::{FONT_SIZE_F, Length, Style, UIElement, UITree},
};
use std::fmt::Write;

/// One pluck or place in flight at a time
#[derive(Default)]
pub(crate) enum PluckPlace {
    #[default]
    Idle,
    /// Waiting for `Plucked`
    Plucking {
        /// A drag can be dropped before the pluck arrives
        queued_place: Option<WorldTreePath>,
    },
    /// Waiting for `Placed`
    Placing,
}

const GHOST_OFFSET: f32 = 16.0;
const GHOST_PADDING: f32 = 10.0;
/// Rough monospace advance at `FONT_SIZE_F`
const CHAR_WIDTH: f32 = FONT_SIZE_F * 0.6;

impl AppletRunner {
    /// Alt+P: pluck at the root's focus, or place there if something is plucked
    pub(crate) fn toggle_pluck_place(&mut self) {
        if !matches!(self.pluck_place, PluckPlace::Idle) {
            return;
        }
        if let Some(plucked) = self.plucked.take() {
            self.place(None, plucked);
        } else {
            self.pluck(None);
        }
    }

    pub(crate) fn pluck(&mut self, path: Option<WorldTreePath>) {
        if matches!(self.pluck_place, PluckPlace::Idle) && self.plucked.is_none() {
            self.pluck_place = PluckPlace::Plucking { queued_place: None };
            self.root_client.send_event(StandardEvent::Pluck(path));
        }
    }

    fn place(&mut self, path: Option<WorldTreePath>, plucked: DetachedTree) {
        self.pluck_place = PluckPlace::Placing;
        self.root_client
            .send_event(StandardEvent::Place(path, plucked));
        self.needs_redraw = true;
    }

    /// `RequestPlace` from the root (eg a treeview drop)
    pub(crate) fn request_place(&mut self, path: WorldTreePath) {
        match &mut self.pluck_place {
            PluckPlace::Plucking { queued_place } => *queued_place = Some(path),
            PluckPlace::Placing => {}
            PluckPlace::Idle => {
                if let Some(plucked) = self.plucked.take() {
                    self.place(Some(path), plucked);
                }
            }
        }
    }

    pub(crate) fn plucked(&mut self, plucked: Option<DetachedTree>) {
        let PluckPlace::Plucking { queued_place } = std::mem::take(&mut self.pluck_place) else {
            eprintln!("Unexpected `Plucked`");
            self.plucked = plucked;
            return;
        };
        self.plucked = plucked;
        if let Some(path) = queued_place
            && let Some(plucked) = self.plucked.take()
        {
            self.place(Some(path), plucked);
        }
        self.needs_redraw = true;
    }

    pub(crate) fn placed(&mut self, given_back: Option<DetachedTree>) {
        if !matches!(self.pluck_place, PluckPlace::Placing) {
            eprintln!("Unexpected `Placed`");
        }
        self.pluck_place = PluckPlace::Idle;
        // REVIEW: the root always places, so this shouldn't happen
        if given_back.is_some() {
            self.plucked = given_back;
        }
        self.needs_redraw = true;
    }

    /// What's plucked, as a box following the pointer (like a drag icon)
    pub(crate) fn plucked_ghost(&self) -> Option<UITree> {
        let plucked = self.plucked.as_ref()?;
        let treeview = plucked.peek_treeview();
        let mut label = treeview.get_root_value().clone();
        let descendants = count_nodes(&treeview) - 1;
        if descendants > 0 {
            let _ = write!(label, " +{descendants}");
        }

        let [x, y] = self.pointer_position.map(|coord| coord as f32);
        let size = [
            label.chars().count() as f32 * CHAR_WIDTH + GHOST_PADDING * 2.0,
            FONT_SIZE_F + GHOST_PADDING * 2.0,
        ];
        let text = UITree::leaf_styled(
            UIElement::Text(vec![(label, UIElement::glyphon_attr(Color::WHITE))]),
            Style::absolute([GHOST_PADDING, GHOST_PADDING], [size[0], FONT_SIZE_F]),
        );
        Some(UITree::new_styled(
            // rect colors are drawn as linear; shows as about #1e2430
            UIElement::Backgrounded(Color([3, 4, 7, 0xE0])),
            Style::absolute([x + GHOST_OFFSET, y + GHOST_OFFSET], size),
            vec![UITree::new_styled(
                UIElement::Bordered(Color::CYAN),
                Style {
                    border: Length::uniform_rect(Length::Px(2.0)),
                    ..Style::full_size()
                },
                vec![text],
            )],
        ))
    }
}

fn count_nodes(treeview: &WorldTree<String>) -> usize {
    use singularity_common::utils::tree::tree_node_path::TraversableTree;
    match treeview {
        WorldTree::Base(_) => 1,
        WorldTree::World(root) => root.iter_paths_dfs().count(),
    }
}
