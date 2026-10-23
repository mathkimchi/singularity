//! A file tree next to the open documents.
//! Opened files are code editors embedded in the file manager's inner tree, so they can be plucked out of it.

use crate::code_editor::CodeEditorApplet;
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        detached::DetachedTree,
        packets::{NodeAction, SpawnPosition, StandardEvent, StandardRequest},
    },
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TreeNodePath, TreeTraverseOperation},
        world_tree::{WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation},
    },
};
use singularity_sttk::{
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        Answer, AppletContext, AppletSpawner, AppletSpawnerTrait, EmbeddedId, NodeRequest,
        NodularAppletInitializer, StandardApplet,
    },
    standard_keybinds::{handle_standard_keybinds, standard_keybind_request},
};
use sonamu_ui::{
    color::Color,
    display_units::DisplayContainerSize,
    ui_element::{Style, UIElement, UITree, font_size},
    ui_event::{KeyModifiers, KeySymbol, KeyTrait, MouseEvent, MouseEventKind, UIEvent},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

fn row_height() -> f32 {
    font_size() + 10.0
}
const INDENT: f32 = 28.0;
const LEFT_PAD: f32 = 12.0;
/// Of the width, for the file list
const LIST_SPLIT: f32 = 0.3;
const BTN_LEFT: u32 = 0x110;

const LIST_BG: Color = Color([0x16, 0x16, 0x1C, 0xFF]);
const SELECTED_BG: Color = Color([0x1C, 0x3D, 0x2E, 0xFF]);
const SELECTED_UNFOCUSED_BG: Color = Color([0x26, 0x2A, 0x32, 0xFF]);
const HEADER_BG: Color = Color([0x0D, 0x0D, 0x16, 0xFF]);
const DIR_TEXT: Color = Color([0x7A, 0xB8, 0xFF, 0xFF]);
const FILE_TEXT: Color = Color::LIGHT_GRAY;
const DIM_TEXT: Color = Color([0x8A, 0x93, 0xA6, 0xFF]);
const ACTIVE_TEXT: Color = Color::LIGHT_YELLOW;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    Files,
    Document,
}

/// An element of its inner tree, selected in its own treeview selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selected {
    /// The file manager itself (its inner root)
    Root,
    Document(usize),
}

/// A visible row of the file tree
struct Entry {
    path: PathBuf,
    depth: usize,
    is_dir: bool,
}

/// An embedded applet, shown next to the file list
struct Document {
    id: EmbeddedId,
    handle: ClientHandle,
    /// `None` if it was placed here rather than opened
    file: Option<PathBuf>,
}

pub struct FileManagerApplet {
    root: PathBuf,
    expanded: HashSet<PathBuf>,
    /// Visible rows, rebuilt when `expanded` changes
    entries: Vec<Entry>,
    selected: usize,
    /// First visible entry
    scroll: usize,

    documents: Vec<Document>,
    active: Option<usize>,
    pane: Pane,
    /// `Some` while selecting an element of its inner tree (traversing), rather than taking input in `pane`
    selecting: Option<Selected>,
    /// Shown instead of the header until the next key (eg a file that can't be opened)
    status: Option<String>,

    size: DisplayContainerSize,
    hook: AppletContext,
}
impl CreatableNodularApplet<PathBuf> for FileManagerApplet {
    fn new(root: PathBuf, hook: AppletContext) -> Self {
        let mut file_manager = Self {
            root,
            expanded: HashSet::new(),
            entries: Vec::new(),
            selected: 0,
            scroll: 0,
            documents: Vec::new(),
            active: None,
            pane: Pane::Files,
            selecting: None,
            status: None,
            // until the node sends a size
            size: DisplayContainerSize::new(800, 600),
            hook,
        };
        file_manager.rebuild_entries();
        file_manager
    }
}
impl FileManagerApplet {
    /// Args: the root directory (default: the working directory)
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct Spawner;
        impl AppletSpawnerTrait for Spawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let root = match args.first() {
                    Some(root) => PathBuf::from(root),
                    None => std::env::current_dir().ok()?,
                };
                Some(FileManagerApplet::get_boxed_initiator(root))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(Spawner)
    }

    fn title(&self) -> String {
        let name = self.root.file_name().map_or_else(
            || self.root.display().to_string(),
            |name| name.to_string_lossy().into(),
        );
        format!("{name}/")
    }

    fn rebuild_entries(&mut self) {
        let selected = self
            .entries
            .get(self.selected)
            .map(|entry| entry.path.clone());
        self.entries.clear();
        push_entries(&self.root, 0, &self.expanded, &mut self.entries);
        // keep the same file selected
        self.selected = selected
            .and_then(|path| self.entries.iter().position(|entry| entry.path == path))
            .unwrap_or(0)
            .min(self.entries.len().saturating_sub(1));
        self.clamp_scroll();
    }

    // sizes are non-negative
    #[allow(clippy::cast_sign_loss)]
    fn list_width(&self) -> u32 {
        (self.size.width as f32 * LIST_SPLIT) as u32
    }

    #[allow(clippy::cast_sign_loss)]
    fn document_size(&self) -> DisplayContainerSize {
        DisplayContainerSize::new(
            self.size.width.saturating_sub(self.list_width()),
            self.size.height.saturating_sub(row_height() as u32),
        )
    }

    /// Entries that fit under the header
    #[allow(clippy::cast_sign_loss)]
    fn visible_rows(&self) -> usize {
        ((self.size.height as f32 / row_height()) as usize)
            .saturating_sub(1)
            .max(1)
    }

    fn clamp_scroll(&mut self) {
        let rows = self.visible_rows();
        self.scroll = self
            .scroll
            .clamp((self.selected + 1).saturating_sub(rows), self.selected);
    }

    fn select(&mut self, index: usize) {
        self.selected = index.min(self.entries.len().saturating_sub(1));
        self.clamp_scroll();
    }

    /// A directory toggles, a file opens
    fn activate_selected(&mut self) {
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        if entry.is_dir {
            let path = entry.path.clone();
            if !self.expanded.remove(&path) {
                self.expanded.insert(path);
            }
            self.rebuild_entries();
        } else {
            self.open(entry.path.clone());
        }
    }

    /// Collapses the selected directory, or selects the parent directory
    fn collapse_selected(&mut self) {
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        if entry.is_dir && self.expanded.remove(&entry.path) {
            self.rebuild_entries();
        } else if let Some(parent) = entry.path.parent()
            && let Some(index) = self.entries.iter().position(|entry| entry.path == parent)
        {
            self.select(index);
        }
    }

    /// Shows the file's editor, embedding a new one if it isn't open
    fn open(&mut self, file: PathBuf) {
        if let Some(index) = self
            .documents
            .iter()
            .position(|document| document.file.as_ref() == Some(&file))
        {
            self.set_active(Some(index));
            self.set_pane(Pane::Document);
            return;
        }
        // shown in the header rather than as an editor showing the error
        if let Err(err) = std::fs::read_to_string(&file) {
            self.status = Some(format!("Can't open: {err}"));
            return;
        }

        let (id, handle) = self
            .hook
            .embed(CodeEditorApplet::get_boxed_initiator(file.clone()));
        self.add_document(Document {
            id,
            handle,
            file: Some(file),
        });
    }

    fn add_document(&mut self, document: Document) {
        document
            .handle
            .send_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                self.document_size(),
            )));
        self.documents.push(document);
        self.set_active(Some(self.documents.len() - 1));
        self.set_pane(Pane::Document);
        self.hook.damage_treeview();
    }

    /// Takes the document out, keeping the others' order
    fn remove_document(&mut self, index: usize) -> Document {
        let document = self.documents.remove(index);
        self.active = match self.active {
            Some(active) if active > index => Some(active - 1),
            // the next one (or the new last one) takes its place
            Some(active) if active == index => {
                (!self.documents.is_empty()).then(|| index.min(self.documents.len() - 1))
            }
            active => active,
        };
        if self.active.is_none() {
            self.pane = Pane::Files;
        }
        self.send_document_focus();
        self.hook.damage_treeview();
        document
    }

    fn set_active(&mut self, active: Option<usize>) {
        if let Some(document) = self.active_document() {
            document
                .handle
                .send_event(StandardEvent::FocusChanged(false));
        }
        self.active = active;
        self.send_document_focus();
    }

    /// Takes input in the pane (leaving its selector)
    fn set_pane(&mut self, pane: Pane) {
        self.selecting = None;
        self.pane = if self.active.is_some() {
            pane
        } else {
            Pane::Files
        };
        self.send_document_focus();
    }

    /// Tells the active document whether it has the keys
    fn send_document_focus(&self) {
        if let Some(document) = self.active_document() {
            document.handle.send_event(StandardEvent::FocusChanged(
                self.pane == Pane::Document && self.selecting.is_none(),
            ));
        }
        // the focus path changed
        self.hook.damage_treeview();
    }

    /// Starts or stops selecting in its own selector
    fn set_selecting(&mut self, selecting: Option<Selected>) {
        if let Some(Selected::Document(index)) = selecting {
            self.set_active(Some(index));
        }
        self.selecting = selecting;
        self.send_document_focus();
        self.hook.damage_window();
    }

    /// Enter: takes input in the selected element (the file list, or a document)
    fn confirm(&mut self, selected: Selected) {
        self.selecting = None;
        match selected {
            Selected::Root => self.set_pane(Pane::Files),
            Selected::Document(index) => {
                self.set_active(Some(index));
                self.set_pane(Pane::Document);
            }
        }
    }

    /// A key in its own selector: traversal of its inner tree (the root, then one document per child)
    fn handle_selector_key(&mut self, selected: Selected, key: KeySymbol, ui_event: UIEvent) {
        // on the selected element: a document closes here, the rest is the file manager's node's
        if let Some(request) = standard_keybind_request(&ui_event) {
            match (selected, request) {
                (Selected::Document(index), NodeRequest::Close) => {
                    self.remove_document(index);
                    self.set_selecting(Some(Selected::Root));
                }
                (_, request) => self.hook.request(request),
            }
            return;
        }
        if matches!(key, KeySymbol::Enter) {
            self.confirm(selected);
            return;
        }
        // documents have no inner trees to collapse
        if matches!(key, KeySymbol::Char(' ')) {
            return;
        }
        let Some(operation) = key
            .to_char()
            .and_then(WorldTreeTraversalOperation::from_char)
        else {
            // any other key goes into the selected element
            self.confirm(selected);
            self.handle_key(ui_event);
            return;
        };
        self.traverse_selector(selected, operation);
    }

    /// Moves its own selector, or out of it to its node
    fn traverse_selector(&mut self, selected: Selected, operation: WorldTreeTraversalOperation) {
        let last = self.documents.len().checked_sub(1);
        match (selected, operation) {
            // out of its inner tree, to its node
            (
                Selected::Root,
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::Parent),
            )
            | (
                _,
                WorldTreeTraversalOperation::PrevLayer | WorldTreeTraversalOperation::GlobalRoot,
            ) => {
                self.selecting = None;
                self.hook.change_focus(operation);
            }
            (
                Selected::Document(_),
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::Parent),
            ) => self.set_selecting(Some(Selected::Root)),
            (Selected::Root, WorldTreeTraversalOperation::NextLayer) => self.confirm(selected),
            // into the document's inner tree, at its root
            (Selected::Document(index), WorldTreeTraversalOperation::NextLayer) => {
                self.confirm(selected);
                self.documents[index]
                    .handle
                    .send_event(StandardEvent::Focus(WorldTreePath::new_into()));
            }
            (
                Selected::Root,
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::Child(index)),
            ) if index < self.documents.len() => {
                self.set_selecting(Some(Selected::Document(index)));
            }
            (
                Selected::Root,
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::LastChild),
            ) => {
                if let Some(last) = last {
                    self.set_selecting(Some(Selected::Document(last)));
                }
            }
            (
                Selected::Document(index),
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::RelShiftSibling(
                    shift,
                )),
            ) => {
                let index = index.saturating_add_signed(shift).min(last.unwrap_or(0));
                self.set_selecting(Some(Selected::Document(index)));
            }
            _ => {}
        }
    }

    fn active_document(&self) -> Option<&Document> {
        self.documents.get(self.active?)
    }

    /// A standard keybind in the file list; returns whether it was one
    fn handle_files_keybind(&mut self, ui_event: &UIEvent) -> bool {
        // eg Alt+q: out of the file list, to its own selector
        if let Some(NodeRequest::ChangeFocus(WorldTreeTraversalOperation::PrevLayer)) =
            standard_keybind_request(ui_event)
        {
            self.set_selecting(Some(Selected::Root));
            return true;
        }
        handle_standard_keybinds(ui_event, &self.hook)
    }

    fn handle_key(&mut self, ui_event: UIEvent) {
        if let Some(selected) = self.selecting {
            match ui_event {
                UIEvent::Key {
                    symbol: Some(key),
                    pressed: true,
                    ..
                } => self.handle_selector_key(selected, key, ui_event),
                // eg 4 fingers up / down like Alt+q / `e`
                UIEvent::Swipe { .. } => {
                    if let Some(NodeRequest::ChangeFocus(operation)) =
                        standard_keybind_request(&ui_event)
                    {
                        self.traverse_selector(selected, operation);
                    }
                }
                // releases, and modifiers on their own, stay in the selector
                _ => {}
            }
            return;
        }
        let UIEvent::Key {
            symbol: Some(key),
            modifiers,
            pressed: true,
            ..
        } = ui_event
        else {
            match self.pane {
                Pane::Document => {
                    if let Some(document) = self.active_document() {
                        document.handle.send_event(StandardEvent::UIEvent(ui_event));
                    }
                }
                // eg swipes
                Pane::Files => {
                    self.handle_files_keybind(&ui_event);
                }
            }
            return;
        };
        self.status = None;

        // Ctrl+Tab switches between the file list and the document
        if modifiers == KeyModifiers::CTRL && matches!(key, KeySymbol::Char('\t')) {
            self.set_pane(match self.pane {
                Pane::Files => Pane::Document,
                Pane::Document => Pane::Files,
            });
            return;
        }

        match self.pane {
            // the document handles its own standard keybinds (eg Ctrl+W closes it)
            Pane::Document => {
                if let Some(document) = self.active_document() {
                    document.handle.send_event(StandardEvent::UIEvent(ui_event));
                }
            }
            Pane::Files => {
                if self.handle_files_keybind(&ui_event) {
                    return;
                }
                match (key, modifiers) {
                    (KeySymbol::ArrowKeyUp | KeySymbol::Char('k'), KeyModifiers::NONE) => {
                        self.select(self.selected.saturating_sub(1));
                    }
                    (KeySymbol::ArrowKeyDown | KeySymbol::Char('j'), KeyModifiers::NONE) => {
                        self.select(self.selected + 1);
                    }
                    (
                        KeySymbol::Enter | KeySymbol::ArrowKeyRight | KeySymbol::Char('l'),
                        KeyModifiers::NONE,
                    ) => self.activate_selected(),
                    (KeySymbol::ArrowKeyLeft | KeySymbol::Char('h'), KeyModifiers::NONE) => {
                        self.collapse_selected();
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_mouse(&mut self, mut mouse_event: MouseEvent) {
        let list_width = f64::from(self.list_width());
        let [x, y] = mouse_event.position;

        if x >= list_width {
            if let MouseEventKind::Button { pressed: true, .. } = mouse_event.kind {
                self.set_pane(Pane::Document);
            }
            if let Some(document) = self.active_document() {
                mouse_event.position = [x - list_width, y - f64::from(row_height())];
                document
                    .handle
                    .send_event(StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)));
            }
            return;
        }

        match mouse_event.kind {
            MouseEventKind::Button {
                button: BTN_LEFT,
                pressed: true,
            } => {
                self.set_pane(Pane::Files);
                // non-negative, since it's past the header
                #[allow(clippy::cast_sign_loss)]
                let row = (y >= f64::from(row_height()))
                    .then(|| ((y / f64::from(row_height())) as usize - 1) + self.scroll)
                    .filter(|&index| index < self.entries.len());
                if let Some(index) = row {
                    self.select(index);
                    self.activate_selected();
                }
            }
            MouseEventKind::Scroll { delta, .. } => {
                let max_scroll = self.entries.len().saturating_sub(self.visible_rows());
                self.scroll = if delta[1] > 0.0 {
                    (self.scroll + 1).min(max_scroll)
                } else if delta[1] < 0.0 {
                    self.scroll.saturating_sub(1)
                } else {
                    self.scroll
                };
            }
            MouseEventKind::Button { .. } | MouseEventKind::Motion => {}
        }
    }

    fn render_list(&self, width: f32) -> UITree {
        let mut rows = vec![row(
            0,
            width,
            HEADER_BG,
            text(LEFT_PAD, self.title(), DIM_TEXT, width),
        )];
        for (row_index, (index, entry)) in self
            .entries
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(self.visible_rows())
            .enumerate()
        {
            let background = if index != self.selected {
                Color::TRANSPARENT
            } else if self.pane == Pane::Files {
                SELECTED_BG
            } else {
                SELECTED_UNFOCUSED_BG
            };
            let name = entry
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let (label, color) = if entry.is_dir {
                let arrow = if self.expanded.contains(&entry.path) {
                    '▾'
                } else {
                    '▸'
                };
                (format!("{arrow} {name}"), DIR_TEXT)
            } else {
                let is_open = self
                    .documents
                    .iter()
                    .any(|document| document.file.as_ref() == Some(&entry.path));
                (
                    format!("  {name}"),
                    if is_open { ACTIVE_TEXT } else { FILE_TEXT },
                )
            };
            rows.push(row(
                row_index + 1,
                width,
                background,
                text(LEFT_PAD + entry.depth as f32 * INDENT, label, color, width),
            ));
        }
        UITree::new_styled(UIElement::Backgrounded(LIST_BG), Style::full_size(), rows)
    }

    /// The open documents' names, then the active document
    fn render_documents(&self, width: f32) -> UITree {
        let header = match &self.status {
            Some(status) => vec![(status.clone(), UIElement::glyphon_attr(Color::RED))],
            None if self.documents.is_empty() => vec![(
                "Open a file (Enter or click)".to_string(),
                UIElement::glyphon_attr(DIM_TEXT),
            )],
            None => self
                .documents
                .iter()
                .enumerate()
                .map(|(index, document)| {
                    let name = document.handle.get_treeview().tree.get_root_value().clone();
                    let color = if Some(index) == self.active {
                        ACTIVE_TEXT
                    } else {
                        DIM_TEXT
                    };
                    (format!(" {name} "), UIElement::glyphon_attr(color))
                })
                .collect(),
        };
        let header = UITree::new_styled(
            UIElement::Backgrounded(HEADER_BG),
            Style::absolute([0.0, 0.0], [width, row_height()]),
            vec![UITree::leaf_styled(
                UIElement::Text(header),
                Style::absolute([LEFT_PAD, text_top()], [width, row_height() - text_top()]),
            )],
        );

        let size = self.document_size();
        let body = self.active_document().map_or_else(
            || UITree::leaf(UIElement::Nothing),
            |document| document.handle.get_surface(),
        );
        let body = UITree::new_styled(
            UIElement::Nothing,
            Style::absolute([0.0, row_height()], [size.width as f32, size.height as f32]),
            vec![body],
        );
        UITree::new_styled(UIElement::Nothing, Style::full_size(), vec![header, body])
    }
}
impl StandardApplet for FileManagerApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        match standard_event {
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                self.size = size;
                self.clamp_scroll();
                let document_size = self.document_size();
                for document in &self.documents {
                    document
                        .handle
                        .send_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                            document_size,
                        )));
                }
            }
            StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)) => self.handle_mouse(mouse_event),
            StandardEvent::UIEvent(ui_event @ UIEvent::Key { .. }) => self.handle_key(ui_event),
            _ => return,
        }
        self.hook.damage_window();
    }

    fn get_window_standard_applet(&self, container_size: DisplayContainerSize) -> UITree {
        let list_width = self.list_width() as f32;
        let documents_width = container_size.width as f32 - list_width;
        UITree::new_container(vec![
            UITree::new_styled(
                UIElement::Nothing,
                Style::absolute([0.0, 0.0], [list_width, container_size.height as f32]),
                vec![self.render_list(list_width)],
            ),
            UITree::new_styled(
                UIElement::Nothing,
                Style::absolute(
                    [list_width, 0.0],
                    [documents_width, container_size.height as f32],
                ),
                vec![self.render_documents(documents_width)],
            ),
        ])
    }

    /// The root is the file manager; each open document is a child
    fn get_treeview(&self) -> WorldTree<String> {
        WorldTree::World(Box::new(RecursiveTreeNode::new(
            WorldTree::Base(self.title()),
            self.documents
                .iter()
                .map(|document| RecursiveTreeNode::from_value(document.handle.get_treeview().tree))
                .collect(),
        )))
    }

    /// Only its root; its open documents aren't saved
    fn save(&self) -> Option<singularity_common::sap::session::SavedApplet> {
        singularity_sttk::session::saved_with_path("file_manager", &self.root)
    }

    /// On the selected element while selecting; otherwise inside the active document while the document pane has
    /// the keys, or inside the file manager itself (the file list)
    fn get_focus_path(&self) -> WorldTreePath {
        match (self.selecting, self.active_document()) {
            (Some(Selected::Document(index)), _) => {
                WorldTreePath(Box::new([TreeNodePath(vec![index])]))
            }
            (None, Some(document)) if self.pane == Pane::Document => document_path(
                self.active.expect("has an active document"),
                document.handle.get_treeview().focus,
            ),
            _ => WorldTreePath::new_into(),
        }
    }

    fn is_traversing(&self) -> bool {
        self.selecting.is_some()
    }

    /// Selects the root (`[[]]`, eg `e` into it) or a document (`[[i]]`), or goes inside a document for longer paths
    fn focus(&mut self, path: &WorldTreePath) {
        match path.0.split_first() {
            Some((node_path, inner))
                if node_path.0.len() == 1 && node_path.0[0] < self.documents.len() =>
            {
                let index = node_path.0[0];
                if inner.is_empty() {
                    self.set_selecting(Some(Selected::Document(index)));
                } else {
                    self.confirm(Selected::Document(index));
                    self.documents[index]
                        .handle
                        .send_event(StandardEvent::Focus(WorldTreePath(inner.into())));
                }
            }
            _ => self.set_selecting(Some(Selected::Root)),
        }
        self.hook.damage_window();
        self.hook.damage_treeview();
    }

    /// Closes a document (`[[i]]`), like Ctrl+W on it in its selector
    fn node_action(&mut self, path: &WorldTreePath, action: NodeAction) {
        if action == NodeAction::Close
            && let [TreeNodePath(index)] = &*path.0
            && let &[index] = index.as_slice()
            && index < self.documents.len()
        {
            self.remove_document(index);
            if self.selecting.is_some() {
                self.set_selecting(Some(Selected::Root));
            }
            self.hook.damage_window();
        }
    }

    /// A document (with everything inside it); the root plucks the file manager itself
    fn pluck(&mut self, path: &WorldTreePath) -> Answer<Option<DetachedTree>> {
        let index = match path.0.first().map(|node_path| node_path.0.as_slice()) {
            Some([index, ..]) if *index < self.documents.len() => *index,
            _ => return Answer::Now(None),
        };
        let document = self.remove_document(index);
        self.hook.damage_window();
        Answer::Now(Some(DetachedTree::leaf(document.handle.detach())))
    }

    /// Becomes a document; trees with children or wl surfaces are given back
    fn place(
        &mut self,
        _path: &WorldTreePath,
        _position: SpawnPosition,
        tree: DetachedTree,
    ) -> Answer<Result<(), DetachedTree>> {
        // TODO: wl surfaces inside inner trees; children of documents
        if tree.main.is_none() || tree.wl_surface.is_some() || !tree.children.is_empty() {
            return Answer::Now(Err(tree));
        }
        let (id, handle) = self.hook.embed_detached(tree.main.expect("checked above"));
        self.add_document(Document {
            id,
            handle,
            file: None,
        });
        self.hook.damage_window();
        Answer::Now(Ok(()))
    }

    fn handle_embedded_request(&mut self, id: EmbeddedId, request: NodeRequest) {
        let Some(index) = self.documents.iter().position(|document| document.id == id) else {
            return;
        };
        match request {
            NodeRequest::Standard(StandardRequest::Damage(_)) => {}
            NodeRequest::Standard(StandardRequest::Quit) => self.hook.quit(),
            // its path is inside its own inner tree, which is child `index` of this one
            NodeRequest::Standard(StandardRequest::RequestPluck(path)) => self
                .hook
                .request(StandardRequest::RequestPluck(document_path(index, path)).into()),
            NodeRequest::Standard(StandardRequest::RequestPlace(path, position)) => {
                self.hook.request(
                    StandardRequest::RequestPlace(document_path(index, path), position).into(),
                )
            }
            NodeRequest::Standard(_) => return,
            // documents can't hold children, split, be wrapped or hold wl surfaces here, so this node does
            request @ (NodeRequest::AddChild(_)
            | NodeRequest::Split { .. }
            | NodeRequest::Wrap(_)
            | NodeRequest::Run(_)
            | NodeRequest::AddWlSurface(_)
            | NodeRequest::FocusSelf) => {
                self.hook.request(request);
            }
            // eg Alt+q: out of the document, to it in the selector
            NodeRequest::ChangeFocus(
                WorldTreeTraversalOperation::PrevLayer
                | WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::Parent),
            ) => self.set_selecting(Some(Selected::Document(index))),
            // eg 3 finger swipes: prev / next node from this one
            request @ NodeRequest::ChangeFocus(WorldTreeTraversalOperation::Layerwise(
                TreeTraverseOperation::BfsPrev | TreeTraverseOperation::BfsNext,
            )) => self.hook.request(request),
            // TODO: a placed project dissolving here loses its contents (they'd need to be the node's children)
            NodeRequest::ChangeFocus(_) | NodeRequest::SwapSibling(_) | NodeRequest::Unwrap(_) => {
                return;
            }
            NodeRequest::Close => {
                // dropping the handle ends it
                self.remove_document(index);
            }
        }
        // a document's surface or title may have changed
        self.hook.damage_window();
        self.hook.damage_treeview();
    }
}

/// A path inside document `index`, as a path of the file manager's inner tree
fn document_path(index: usize, path: WorldTreePath) -> WorldTreePath {
    let mut full = vec![TreeNodePath(vec![index])];
    full.extend(path.0);
    WorldTreePath(full.into_boxed_slice())
}

/// Directories first, then files, each sorted by name; hidden files are skipped
fn push_entries(dir: &Path, depth: usize, expanded: &HashSet<PathBuf>, entries: &mut Vec<Entry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<(bool, PathBuf)> = read_dir
        .filter_map(Result::ok)
        .filter(|child| !child.file_name().to_string_lossy().starts_with('.'))
        .map(|child| (child.path().is_dir(), child.path()))
        .collect();
    children.sort_by(|(a_dir, a), (b_dir, b)| b_dir.cmp(a_dir).then_with(|| a.cmp(b)));

    for (is_dir, path) in children {
        let expand = is_dir && expanded.contains(&path);
        entries.push(Entry {
            path: path.clone(),
            depth,
            is_dir,
        });
        if expand {
            push_entries(&path, depth + 1, expanded, entries);
        }
    }
}

fn text_top() -> f32 {
    (row_height() - font_size()) / 2.0
}

fn text(left: f32, label: String, color: Color, width: f32) -> UITree {
    UITree::leaf_styled(
        UIElement::Text(vec![(label, UIElement::glyphon_attr(color))]),
        Style::absolute(
            [left, text_top()],
            [(width - left).max(0.0), row_height() - text_top()],
        ),
    )
}

fn row(index: usize, width: f32, background: Color, content: UITree) -> UITree {
    let background = if background == Color::TRANSPARENT {
        UIElement::Nothing
    } else {
        UIElement::Backgrounded(background)
    };
    UITree::new_styled(
        background,
        Style::absolute([0.0, index as f32 * row_height()], [width, row_height()]),
        vec![content],
    )
}
