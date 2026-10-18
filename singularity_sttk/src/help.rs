//! Lists the keybinds. Super+? opens it under the focused node (the root does that); Esc, q or Super+? closes it.

use crate::{
    command_hub::{ACCENT, BAR_BG, BG, CURSOR, DIM, OUTPUT, RUNNING, fill_row, put},
    nodular_applet::{
        AppletContext, AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer, StandardApplet,
    },
    standard_keybinds::handle_standard_keybinds,
};
use singularity_common::{sap::packets::StandardEvent, utils::tree::world_tree::WorldTree};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{CharGrid, UITree},
    ui_event::{KeySymbol, MouseEventKind, UIEvent},
};

/// `(keys, what they do)`; empty keys start a section
const BINDS: &[(&str, &str)] = &[
    ("", "Anywhere"),
    ("Super+?", "show or hide this help"),
    (
        "Alt+P",
        "pluck what's focused; again to place it at the focus",
    ),
    (
        "Super+P",
        "put the node (and under it) in a project; Ctrl+W on the project undoes it",
    ),
    (
        "Super+V",
        "view only the focused node's subtree; on the view root, view everything",
    ),
    ("Ctrl+Shift+Q", "quit Sonamu"),
    ("Ctrl+Alt+Backspace", "quit (on a TTY)"),
    ("Ctrl+Alt+F1..F12", "switch VT (on a TTY)"),
    ("", "Inside an applet (green border)"),
    ("Super+W", "close it; its children move up"),
    ("Super+T / Super+N", "start foot / qutebrowser"),
    ("Ctrl+Shift++", "add a command hub as its child"),
    ("Ctrl+Shift+|", "split side by side with a new command hub"),
    ("Ctrl+Shift+_", "split stacked with a new command hub"),
    (
        "Alt+Q",
        "out of the applet, to traversal (its own, if it has one)",
    ),
    ("", "Traversal (yellow border, treeview shown)"),
    ("a, 0", "parent; past the view root, widens the view"),
    ("d, 1-8, 9", "first, nth or last child"),
    ("w, s", "previous or next sibling"),
    ("e", "into its inner tree"),
    (
        "q",
        "out of this inner tree, to the node holding it (or the root)",
    ),
    ("Enter", "confirm: into the applet"),
    ("Space", "collapse or expand its inner tree in the treeview"),
    (
        "Super+W, Ctrl+Shift++/|/_",
        "as inside, on the selected node (Ctrl+W on a project undoes it)",
    ),
    ("other keys", "go into the applet"),
    ("", "Mouse"),
    ("click a pane", "focus inside it"),
    ("click a row", "focus that node (treeview)"),
    (
        "drag a row",
        "pluck it; drop it on a row or a pane to place it",
    ),
    ("", "Command hub"),
    ("Tab", "complete commands and applet names"),
    ("Up, Down", "recall commands"),
    ("PageUp, PageDown", "scroll"),
    ("Esc", "clear the prompt"),
    ("help", "(command) list the commands"),
    ("", "File manager"),
    ("Ctrl+Tab", "switch between the file list and the document"),
    ("Up, Down, j, k", "move"),
    ("Enter, Right, l", "open a file or expand a folder"),
    ("Left, h", "collapse, or go to the parent folder"),
    ("Alt+Q", "(in a document) back to the file list"),
    ("", "This help"),
    ("Esc, q", "close"),
    ("Up, Down, PgUp, PgDn", "scroll"),
];

/// Below this, keys go above their descriptions instead of beside them
const MIN_DESCRIPTION_WIDTH: usize = 24;
const SCROLL_STEP: usize = 3;

/// A row of the list: keys (if any) and text
struct Row {
    keys: &'static str,
    text: String,
    section: bool,
}

pub struct HelpApplet {
    /// Rows scrolled past
    scroll: usize,
    size: DisplayContainerSize,
    hook: AppletContext,
}
impl HelpApplet {
    #[must_use]
    pub fn get_boxed_initiator() -> NodularAppletInitializer {
        Box::new(|hook: AppletContext| {
            Box::new(Self {
                scroll: 0,
                size: DisplayContainerSize::new(800, 600),
                hook,
            })
        })
    }

    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct HelpSpawner;
        impl AppletSpawnerTrait for HelpSpawner {
            fn create_initializer(&self, _args: &[&str]) -> Option<NodularAppletInitializer> {
                Some(HelpApplet::get_boxed_initiator())
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(HelpSpawner)
    }

    /// Every row at `width`, descriptions word-wrapped
    fn rows(width: usize) -> Vec<Row> {
        let key_width = BINDS.iter().map(|(keys, _)| keys.len()).max().unwrap_or(0);
        let beside = width >= 2 + key_width + 2 + MIN_DESCRIPTION_WIDTH;
        let mut rows = Vec::new();
        for &(keys, text) in BINDS {
            if keys.is_empty() {
                if !rows.is_empty() {
                    rows.push(Row {
                        keys: "",
                        text: String::new(),
                        section: false,
                    });
                }
                rows.push(Row {
                    keys: "",
                    text: text.to_string(),
                    section: true,
                });
                continue;
            }
            if !beside {
                rows.push(Row {
                    keys,
                    text: String::new(),
                    section: false,
                });
            }
            let indent = if beside { 2 + key_width + 2 } else { 4 };
            for (index, line) in word_wrap(text, width.saturating_sub(indent))
                .into_iter()
                .enumerate()
            {
                rows.push(Row {
                    keys: if beside && index == 0 { keys } else { "" },
                    text: format!("{}{line}", " ".repeat(indent)),
                    section: false,
                });
            }
        }
        rows
    }

    fn max_scroll(&self) -> usize {
        let (width, height) = CharGrid::largest_fittable_size(self.size);
        Self::rows(width)
            .len()
            .saturating_sub(height.saturating_sub(1))
    }

    /// Positive scrolls down
    fn scroll_by(&mut self, rows: isize) {
        self.scroll = self
            .scroll
            .saturating_add_signed(rows)
            .min(self.max_scroll());
        self.hook.damage_window();
    }
}
impl StandardApplet for HelpApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        let StandardEvent::UIEvent(ui_event) = standard_event else {
            return;
        };
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }
        let (_, height) = CharGrid::largest_fittable_size(self.size);
        let page = height.saturating_sub(2).max(1) as isize;
        match ui_event {
            UIEvent::Key {
                symbol: Some(key),
                pressed: true,
                ..
            } => match key {
                KeySymbol::Escape | KeySymbol::Char('q') => self.hook.close(),
                KeySymbol::ArrowKeyDown | KeySymbol::Char('j') => self.scroll_by(1),
                KeySymbol::ArrowKeyUp | KeySymbol::Char('k') => self.scroll_by(-1),
                KeySymbol::PageDown | KeySymbol::Char(' ') => self.scroll_by(page),
                KeySymbol::PageUp => self.scroll_by(-page),
                _ => {}
            },
            UIEvent::Mouse(mouse_event) => {
                if let MouseEventKind::Scroll { delta, .. } = mouse_event.kind {
                    let step = SCROLL_STEP as isize;
                    self.scroll_by(if delta[1] > 0.0 { step } else { -step });
                }
            }
            UIEvent::WindowResized(size) => {
                self.size = size;
                self.scroll = self.scroll.min(self.max_scroll());
            }
            UIEvent::Key { .. } | UIEvent::Swipe { .. } => {}
        }
    }

    fn get_window_standard_applet(&self, container_size: DisplayContainerSize) -> UITree {
        let (width, height) = CharGrid::largest_fittable_size(container_size);
        let mut grid = CharGrid::new_empty(width, height);
        if width == 0 || height == 0 {
            return grid.element().into();
        }
        for row in 0..height {
            fill_row(&mut grid, row, BG);
        }

        fill_row(&mut grid, 0, BAR_BG);
        let title_end = put(&mut grid, 0, 1, "Keybinds", CURSOR);
        let note = "Esc closes, Up/Down scroll ";
        if let Some(column) = width
            .checked_sub(note.len())
            .filter(|&column| column > title_end + 1)
        {
            put(&mut grid, 0, column, note, DIM);
        }

        for (row, line) in (1..height).zip(Self::rows(width).iter().skip(self.scroll)) {
            if line.section {
                put(&mut grid, row, 1, &line.text, ACCENT);
            } else {
                put(&mut grid, row, 0, &line.text, OUTPUT);
                put(&mut grid, row, 2, line.keys, RUNNING);
            }
        }

        grid.element().into()
    }

    fn get_treeview(&self) -> WorldTree<String> {
        WorldTree::Base("Help".to_string())
    }
}

/// Greedy, on spaces; words longer than `width` are cut
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = vec![String::new()];
    for word in text.split(' ') {
        let line = lines.last_mut().expect("starts with one");
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(String::new());
        }
        let line = lines.last_mut().expect("starts with one");
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    lines
        .into_iter()
        .flat_map(|line| {
            line.chars()
                .collect::<Vec<_>>()
                .chunks(width)
                .map(|chunk| chunk.iter().collect())
                .collect::<Vec<String>>()
        })
        .collect()
}
