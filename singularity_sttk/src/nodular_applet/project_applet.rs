//! A project: an applet holding the project's contents as its inner tree, like the root holds the outer tree.
//! Its node doesn't know it is a project; it only sees an applet with an inner tree.

use crate::{
    command_hub::{BG, CommandHubApplet},
    standard_keybinds::{BROWSER, TERMINAL},
    nodular_applet::{
        Answer, AppletContext, EmbeddedId, NodeRequest, NodularAppletInitializer, StandardApplet,
        WrapInitializer,
        node_holder::NodeHolder,
        node_tree::{AttachClient, NodeId, NodeTree, SpawnClient, root_label_mut},
    },
};
use singularity_common::{
    sap::{
        detached::{DetachedClient, DetachedTree, Layout},
        packets::{AwaitedWl, NewWlSurface, StandardEvent, StandardRequest},
        session::{SavedApplet, SavedTree},
    },
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    color::Color,
    display_units::DisplayContainerSize,
    ui_element::{FONT_SIZE_F, Style, UIElement, UITree},
    ui_event::{KeyModifiers, KeySymbol, MouseEvent, MouseEventKind, UIEvent},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Its name until it is renamed: its inner tree root's label
const LABEL: &str = "Project";

/// The overview's rows: the name, a hint, its contents, then `NEW_ROWS`
const ROW_HEIGHT: f32 = FONT_SIZE_F + 14.0;
const PAD: f32 = 16.0;
/// Rows before its contents
const CONTENTS_ROW: usize = 2;
/// Rows after its contents that add to it: label, key, and the program to start (`None`: a command hub)
const NEW_ROWS: [(&str, char, Option<&str>); 3] = [
    ("+ command hub  (n)", 'n', None),
    ("+ terminal  (t)", 't', Some(TERMINAL)),
    ("+ browser  (b)", 'b', Some(BROWSER)),
];
const TEXT: Color = Color::LIGHT_GRAY;
const HINT: Color = Color([0x8A, 0x93, 0xA6, 0xFF]);
const BTN_LEFT: u32 = 0x110;

/// Shows its contents' panes; the root's treeview shows its contents as its node's inner tree.
/// Its inner tree's root is a container (no main), so focus inside the project at its root (`[[project], []]`)
/// can add children to the project, apart from focus on the project's node (`[[project]]`).
///
/// Closing it (`CloseRequest`, eg Ctrl+W on its node) gives its contents back to its node, which they replace.
///
/// While focus is at its inner root, it shows an overview instead of its panes: its name (`r` renames it),
/// its contents (click to select) and rows that add to it (`NEW_ROWS`: a command hub, terminal or browser).
pub struct ProjectApplet {
    holder: NodeHolder,
    hook: AppletContext,
    /// The node each embedded applet (its contents' mains) is the main of
    nodes: Rc<RefCell<HashMap<EmbeddedId, NodeId>>>,
    /// Its contents were given back, so its tree is empty
    dissolved: bool,
    name: String,
    /// The new name being typed (empty keeps the old one)
    renaming: Option<String>,
}
impl ProjectApplet {
    /// Makes a project holding a subtree (`NodeRequest::Wrap`), under its inner root
    pub fn wrapper() -> WrapInitializer {
        Box::new(|subtree| {
            Box::new(move |hook| Box::new(Self::new(subtree, hook)) as Box<dyn StandardApplet>)
        })
    }

    /// Makes a saved project again, holding its saved contents
    pub fn restorer(mut contents: SavedTree) -> NodularAppletInitializer {
        Box::new(move |hook| {
            let name = contents.label.take().unwrap_or_else(|| LABEL.to_string());
            let mut project = Self::with_tree(hook, |spawn_client, attach_client| {
                NodeTree::restore(contents, spawn_client, attach_client)
            });
            project.name = name;
            Box::new(project) as Box<dyn StandardApplet>
        })
    }

    fn new(subtree: DetachedTree, hook: AppletContext) -> Self {
        let contents = DetachedTree {
            main: None,
            layout: Layout::Single,
            wl_surface: None,
            children: vec![subtree],
        };
        Self::with_tree(hook, |spawn_client, attach_client| {
            NodeTree::from_detached(contents, spawn_client, attach_client)
        })
    }

    /// Holds the tree made with clients embedded in it
    fn with_tree(
        hook: AppletContext,
        make_tree: impl FnOnce(SpawnClient, AttachClient) -> NodeTree,
    ) -> Self {
        let nodes: Rc<RefCell<HashMap<EmbeddedId, NodeId>>> = Rc::default();
        let spawn_client: SpawnClient = {
            let hook = hook.clone();
            let nodes = Rc::clone(&nodes);
            Box::new(move |id, initializer| {
                let (embedded, handle) = hook.embed(initializer);
                nodes.borrow_mut().insert(embedded, id);
                handle
            })
        };
        let attach_client: AttachClient = {
            let hook = hook.clone();
            let nodes = Rc::clone(&nodes);
            Box::new(move |id, client: DetachedClient| {
                let (embedded, handle) = hook.embed_detached(client);
                nodes.borrow_mut().insert(embedded, id);
                handle
            })
        };
        let tree = make_tree(spawn_client, attach_client);
        let send = {
            let hook = hook.clone();
            Box::new(move |request: StandardRequest| hook.request(request.into()))
        };
        Self {
            holder: NodeHolder::new(tree, send, true),
            hook,
            nodes,
            dissolved: false,
            name: LABEL.to_string(),
            renaming: None,
        }
    }

    /// Focus is on its inner root (the project itself), so it shows the overview
    fn at_root(&self) -> bool {
        !self.dissolved && self.holder.tree.focused_node() == self.holder.tree.root()
    }

    /// What `NEW_ROWS[index]` adds: a command hub in it, focused inside, or a program, whose window lands in it
    /// (focus is at its root)
    fn add_new(&mut self, index: usize) {
        let root = self.holder.tree.root();
        match NEW_ROWS[index].2 {
            Some(program) => {
                self.holder.handle_node_request(root, NodeRequest::Run(program));
            }
            None => {
                if let Some(id) = self
                    .holder
                    .tree
                    .add_child(root, CommandHubApplet::get_boxed_initiator())
                {
                    self.holder.tree.focus(id, true);
                }
            }
        }
    }

    /// The overview's keys and clicks; gives back what it doesn't take
    fn handle_overview_event(&mut self, ui_event: UIEvent) -> Option<UIEvent> {
        match ui_event {
            UIEvent::Mouse(MouseEvent {
                position: [_, y],
                kind:
                    MouseEventKind::Button {
                        button: BTN_LEFT,
                        pressed: true,
                    },
                ..
            }) => {
                let root = self.holder.tree.root();
                let contents = self.holder.tree.children(root).len();
                // non-negative, since `y >= PAD`
                #[allow(clippy::cast_sign_loss)]
                let row = (y >= f64::from(PAD))
                    .then(|| ((y - f64::from(PAD)) / f64::from(ROW_HEIGHT)) as usize);
                match row {
                    None => {}
                    Some(0) => self.renaming = Some(String::new()),
                    Some(row) if (CONTENTS_ROW..CONTENTS_ROW + contents).contains(&row) => {
                        let id = self.holder.tree.children(root)[row - CONTENTS_ROW];
                        self.holder.tree.focus(id, false);
                    }
                    Some(row) if row >= CONTENTS_ROW + contents => {
                        let index = row - CONTENTS_ROW - contents;
                        if index < NEW_ROWS.len() {
                            self.add_new(index);
                        }
                    }
                    Some(_) => {}
                }
            }
            // the panes aren't shown
            UIEvent::Mouse(_) => {}
            UIEvent::Key {
                symbol: Some(key),
                modifiers,
                pressed: true,
                ..
            } if !(modifiers.ctrl || modifiers.alt || modifiers.logo) => {
                match (&mut self.renaming, key) {
                    (Some(_), KeySymbol::Enter) => {
                        let name = self.renaming.take().unwrap_or_default();
                        if !name.trim().is_empty() {
                            self.name = name.trim().to_string();
                        }
                    }
                    (Some(_), KeySymbol::Escape) => self.renaming = None,
                    (Some(name), KeySymbol::Backspace) => {
                        name.pop();
                    }
                    (Some(name), KeySymbol::Char(key_char)) if !key_char.is_control() => {
                        name.push(key_char);
                    }
                    (None, KeySymbol::Char('r')) if modifiers == KeyModifiers::NONE => {
                        self.renaming = Some(String::new());
                    }
                    (None, KeySymbol::Char(key_char))
                        if modifiers == KeyModifiers::NONE
                            && let Some(index) =
                                NEW_ROWS.iter().position(|&(_, key, _)| key == key_char) =>
                    {
                        self.add_new(index);
                    }
                    _ => return Some(ui_event),
                }
            }
            _ => return Some(ui_event),
        }
        self.hook.damage_window();
        self.hook.damage_treeview();
        None
    }

    fn get_overview(&self, width: f32) -> UITree {
        let tree = &self.holder.tree;
        let (title, hint) = match &self.renaming {
            Some(name) => (format!("{name}▏"), "Enter: save, Esc: cancel"),
            None => (self.name.clone(), "r: rename, 1-9: select"),
        };
        let contents = (1..)
            .zip(tree.children(tree.root()))
            .map(|(index, &id)| (format!("{index}  {}", tree.label(id)), TEXT));
        let rows = [(title, TEXT), (hint.to_string(), HINT)]
            .into_iter()
            .chain(contents)
            .chain(NEW_ROWS.map(|(label, _, _)| (label.to_string(), TEXT)));
        let rows = (0..)
            .zip(rows)
            .map(|(index, (text, color))| {
                UITree::leaf_styled(
                    UIElement::Text(vec![(text, UIElement::glyphon_attr(color))]),
                    Style::absolute(
                        [PAD, PAD + index as f32 * ROW_HEIGHT],
                        [(width - PAD).max(0.0), ROW_HEIGHT],
                    ),
                )
            })
            .collect();
        UITree::new_styled(UIElement::Backgrounded(BG), Style::full_size(), rows)
    }
}
impl StandardApplet for ProjectApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        if self.dissolved {
            return;
        }
        let standard_event = match standard_event {
            StandardEvent::UIEvent(ui_event) if self.at_root() => {
                let Some(ui_event) = self.handle_overview_event(ui_event) else {
                    return;
                };
                StandardEvent::UIEvent(ui_event)
            }
            standard_event => standard_event,
        };
        match standard_event {
            StandardEvent::UIEvent(_)
            | StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..) => {
                if let Some(request) = self.holder.handle_ui_event(standard_event) {
                    self.hook.request(request);
                }
            }
            // TODO: surfaces inside projects; the root doesn't send these here yet
            StandardEvent::FocusWlSurface(surface_id) => {
                self.holder.tree.focus_wl_surface(surface_id);
            }
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                self.holder.close_wl_surface(surface_id);
            }
            StandardEvent::WlSurfaceTitle(surface_id, title) => {
                self.holder.set_wl_surface_title(surface_id, title);
            }
            // handled by the applet's client
            StandardEvent::CloseRequest
            | StandardEvent::DamageAck(_)
            | StandardEvent::WlSurfaceRegistered(_)
            | StandardEvent::Pluck(_)
            | StandardEvent::Place(..)
            | StandardEvent::Focus(_) => {}
            // only the root gets these
            StandardEvent::Capturing(_) => {}
        }
    }

    fn get_window_standard_applet(&self, container_size: DisplayContainerSize) -> UITree {
        if self.dissolved {
            return UITree::leaf(UIElement::Nothing);
        }
        if self.at_root() {
            return self.get_overview(container_size.width as f32);
        }
        self.holder.tree.get_window()
    }

    /// Its tree, whose root (a container) is labeled with the project's name
    fn get_treeview(&self) -> WorldTree<String> {
        if self.dissolved {
            return WorldTree::Base(self.name.clone());
        }
        let mut treeview = self.holder.tree.get_treeview();
        root_label_mut(&mut treeview).clone_from(&self.name);
        treeview
    }

    fn get_focus_path(&self) -> WorldTreePath {
        if self.dissolved {
            return WorldTreePath::new_into();
        }
        self.holder.tree.get_focus_path()
    }

    fn is_traversing(&self) -> bool {
        !self.dissolved && self.holder.tree.is_traversing()
    }

    /// Through its own tree, while focus is in it
    fn handles_dfs_swipes(&self) -> bool {
        !self.dissolved
    }

    fn focus(&mut self, path: &WorldTreePath) {
        if !self.dissolved {
            self.holder.focus(path);
        }
    }

    /// Answered by its holder, maybe after asking an applet inside
    fn pluck(&mut self, path: &WorldTreePath) -> Answer<Option<DetachedTree>> {
        if self.dissolved {
            return Answer::Now(None);
        }
        self.holder.start_pluck(Some(path));
        Answer::Later
    }

    fn place(
        &mut self,
        path: &WorldTreePath,
        tree: DetachedTree,
    ) -> Answer<Result<(), DetachedTree>> {
        if self.dissolved {
            return Answer::Now(Err(tree));
        }
        self.holder.start_place(Some(path), tree);
        Answer::Later
    }

    /// It closes into its contents
    fn close_requested(&mut self) -> bool {
        self.holder.dissolve = true;
        false
    }

    fn take_wl_surface(&mut self, surface: NewWlSurface) -> Result<(), NewWlSurface> {
        if self.dissolved {
            return Err(surface);
        }
        self.holder.add_wl_surface(surface);
        Ok(())
    }

    fn save(&self) -> Option<SavedApplet> {
        (!self.dissolved).then(|| {
            let mut contents = self.holder.tree.save();
            contents.label = Some(self.name.clone());
            SavedApplet::Project(Box::new(contents))
        })
    }

    fn awaiting_wl(&self) -> Vec<AwaitedWl> {
        if self.dissolved {
            return Vec::new();
        }
        self.holder.tree.awaiting_wl()
    }

    /// From a main of its contents
    fn handle_embedded_request(&mut self, id: EmbeddedId, request: NodeRequest) {
        let Some(&id) = self.nodes.borrow().get(&id) else {
            return;
        };
        if let Some(request) = self.holder.handle_node_request(id, request) {
            self.hook.request(request);
        }
    }

    fn update(&mut self) {
        if self.dissolved {
            return;
        }
        if self.holder.dissolve {
            let contents = self.holder.tree.detach_all();
            self.hook.request(NodeRequest::Unwrap(contents));
            self.dissolved = true;
            return;
        }
        self.holder.finish();
        if std::mem::take(&mut self.holder.quit) {
            self.hook.quit();
        }
        // eg `q` inside it: out to its node
        if let Some(operation) = self.holder.tree.take_escaped() {
            self.hook.change_focus(operation);
        }
        self.holder.tree.update_view_root();
        if self.holder.tree.take_damage() {
            self.hook.damage_window();
            self.hook.damage_treeview();
        }
        self.holder.tree.update_layout();
        self.holder.update_wl_focus();
    }
}
