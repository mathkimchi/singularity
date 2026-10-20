//! A project: an applet holding the project's contents as its inner tree, like the root holds the outer tree.
//! Its node doesn't know it is a project; it only sees an applet with an inner tree.

use crate::{
    command_hub::{BG, CommandHubApplet},
    nodular_applet::{
        Answer, AppletContext, EmbeddedId, NodeRequest, NodularAppletInitializer, StandardApplet,
        WrapInitializer,
        node_holder::NodeHolder,
        node_tree::{AttachClient, NodeId, NodeTree, SpawnClient, root_label_mut},
    },
    standard_keybinds::{BROWSER, TERMINAL},
};
use singularity_common::{
    sap::{
        detached::{DetachedClient, DetachedTree, Layout},
        packets::{AwaitedWl, NewWlSurface, SpawnPosition, StandardEvent, StandardRequest},
        session::{SavedApplet, SavedTree},
    },
    settings,
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    color::Color,
    display_units::DisplayContainerSize,
    ui_element::{Style, UIElement, UITree, font_size},
    ui_event::{KeyModifiers, KeySymbol, MouseEvent, MouseEventKind, UIEvent},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

/// Its name until it is renamed: its inner tree root's label
const LABEL: &str = "Project";

/// The overview's rows: the name, its directory, a hint, its contents, then `NEW_ROWS`
fn row_height() -> f32 {
    font_size() + 14.0
}
const PAD: f32 = 16.0;
const DIRECTORY_ROW: usize = 1;
/// Rows before its contents (or the directory options, while setting the directory)
const CONTENTS_ROW: usize = 3;
/// Most directory options shown
const MAX_OPTIONS: usize = 20;
/// Rows after its contents that add to it: label, key, and the program to start (`None`: a command hub)
const NEW_ROWS: [(&str, char, Option<&str>); 3] = [
    ("+ command hub  (n)", 'n', None),
    ("+ terminal  (t)", 't', Some(TERMINAL)),
    ("+ browser  (b)", 'b', Some(BROWSER)),
];
// Other text is the settings' `palette.fg`
const HINT: Color = Color([0x8A, 0x93, 0xA6, 0xFF]);
const ERROR: Color = Color([0xE0, 0x6C, 0x75, 0xFF]);
const BTN_LEFT: u32 = 0x110;

/// The home directory, which paths show as `~`
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// The path, with the home directory as `~`
fn tilde(path: &Path) -> String {
    match home().and_then(|home| path.strip_prefix(home).ok().map(Path::to_path_buf)) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// The typed path with `~` expanded; `None` if it is relative
fn expand(path: &str) -> Option<PathBuf> {
    if path == "~" {
        home()
    } else if let Some(rest) = path.strip_prefix("~/") {
        home().map(|home| home.join(rest))
    } else {
        path.starts_with('/').then(|| PathBuf::from(path))
    }
}

/// A directory path being typed (absolute or from `~`), with the directories it could complete to
struct DirectoryInput {
    path: String,
    /// Paths, as typed, of the directories in `path`'s parent whose names start with its last part
    options: Vec<String>,
    /// The last Enter was on a path that isn't a directory
    invalid: bool,
    /// The option picked with the arrows, which Tab fills in
    selected: Option<usize>,
}
impl DirectoryInput {
    fn new(path: String) -> Self {
        let mut input = Self {
            path: String::new(),
            options: Vec::new(),
            invalid: false,
            selected: None,
        };
        input.set(path);
        input
    }

    fn set(&mut self, path: String) {
        self.options = path
            .rsplit_once('/')
            .and_then(|(parent, prefix)| {
                // only the root's parent is empty
                let parent = if parent.is_empty() { "/" } else { parent };
                let join = if parent == "/" { "" } else { "/" };
                let entries = std::fs::read_dir(expand(parent)?).ok()?;
                let mut options: Vec<String> = entries
                    .flatten()
                    .filter(|entry| entry.path().is_dir())
                    .filter_map(|entry| entry.file_name().into_string().ok())
                    // hidden ones only once a `.` is typed
                    .filter(|name| {
                        name.starts_with(prefix)
                            && (prefix.starts_with('.') || !name.starts_with('.'))
                    })
                    .map(|name| format!("{parent}{join}{name}"))
                    .collect();
                options.sort();
                Some(options)
            })
            .unwrap_or_default();
        self.path = path;
        self.invalid = false;
        self.selected = None;
    }

    /// Moves the arrow selection through the shown options, down if `down`; wraps around
    fn select(&mut self, down: bool) {
        let shown = self.options.len().min(MAX_OPTIONS);
        if shown == 0 {
            return;
        }
        self.selected = Some(match (self.selected, down) {
            (None, true) => 0,
            (None, false) => shown - 1,
            (Some(index), true) => (index + 1) % shown,
            (Some(index), false) => (index + shown - 1) % shown,
        });
    }

    /// Fills in the selected option, else completes to the options' longest common prefix,
    /// or into the only option once it is reached
    fn complete(&mut self) {
        if let Some(option) = self.selected.and_then(|index| self.options.get(index)) {
            self.set(option.clone());
            return;
        }
        let Some((first, rest)) = self.options.split_first() else {
            return;
        };
        if rest.is_empty() && *first == self.path {
            self.set(format!("{first}/"));
            return;
        }
        let common = rest.iter().fold(first.as_str(), |common, option| {
            let len = common
                .char_indices()
                .zip(option.chars())
                .take_while(|((_, a), b)| a == b)
                .last()
                .map_or(0, |((index, a), _)| index + a.len_utf8());
            &common[..len]
        });
        if common.len() > self.path.len() {
            self.set(common.to_string());
        }
    }

    /// The directory to save (`Ok(None)` clears it), or `Err` if the path isn't an absolute directory
    fn parse(&self) -> Result<Option<PathBuf>, ()> {
        let path = self.path.trim();
        if path.is_empty() {
            return Ok(None);
        }
        expand(path)
            .and_then(|path| std::fs::canonicalize(path).ok())
            .filter(|path| path.is_dir())
            .map(Some)
            .ok_or(())
    }
}

/// Shows its contents' panes; the root's treeview shows its contents as its node's inner tree.
/// Its inner tree's root is a container (no main), so focus inside the project at its root (`[[project], []]`)
/// can add children to the project, apart from focus on the project's node (`[[project]]`).
///
/// Closing it (`CloseRequest`, eg Ctrl+W on its node) gives its contents back to its node, which they replace.
///
/// While focus is at its inner root, it shows an overview instead of its panes: its name (`r` renames it), its directory (`d` sets it),
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
    /// The new directory being typed (empty clears it)
    setting_directory: Option<DirectoryInput>,
}
impl ProjectApplet {
    /// Makes a project holding a subtree (`NodeRequest::Wrap`), under its inner root,
    /// in the directory of the tree it is made in
    pub fn wrapper() -> WrapInitializer {
        Box::new(|subtree, directory| {
            Box::new(move |hook| {
                let mut project = Self::new(subtree, hook);
                project.holder.directory = directory;
                Box::new(project) as Box<dyn StandardApplet>
            })
        })
    }

    /// Makes a saved project again, holding its saved contents
    pub fn restorer(mut contents: SavedTree) -> NodularAppletInitializer {
        Box::new(move |hook| {
            let name = contents.label.take().unwrap_or_else(|| LABEL.to_string());
            let directory = contents.directory.take();
            let mut project = Self::with_tree(hook, |spawn_client, attach_client| {
                NodeTree::restore(contents, spawn_client, attach_client)
            });
            project.name = name;
            project.holder.directory = directory;
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
            setting_directory: None,
        }
    }

    /// Starts typing a new directory, from the current one (the runner's if it has none)
    fn start_setting_directory(&mut self) {
        let directory = self
            .holder
            .directory
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .map(|directory| tilde(&directory))
            .unwrap_or_default();
        self.renaming = None;
        self.setting_directory = Some(DirectoryInput::new(directory));
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
                self.holder
                    .handle_node_request(root, NodeRequest::Run(program));
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

    /// A key while setting the directory; whether it took it
    fn handle_directory_key(&mut self, key: KeySymbol, modifiers: KeyModifiers) -> bool {
        let Some(input) = &mut self.setting_directory else {
            return false;
        };
        match key {
            KeySymbol::Enter => match input.parse() {
                Ok(directory) => {
                    self.holder.directory = directory;
                    self.setting_directory = None;
                    // republish, so the session saves it
                    self.hook.damage_treeview();
                }
                Err(()) => input.invalid = true,
            },
            KeySymbol::Escape => self.setting_directory = None,
            KeySymbol::Backspace => {
                let mut path = input.path.clone();
                path.pop();
                input.set(path);
            }
            KeySymbol::Char('\t') if modifiers == KeyModifiers::NONE => input.complete(),
            KeySymbol::ArrowKeyDown => input.select(true),
            KeySymbol::ArrowKeyUp => input.select(false),
            KeySymbol::Char(key_char) if !key_char.is_control() => {
                let mut path = input.path.clone();
                path.push(key_char);
                input.set(path);
            }
            _ => return false,
        }
        true
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
                    .then(|| ((y - f64::from(PAD)) / f64::from(row_height())) as usize);
                match row {
                    None => {}
                    Some(row) if row >= CONTENTS_ROW && self.setting_directory.is_some() => {
                        if let Some(input) = &mut self.setting_directory
                            && let Some(option) = input.options.get(row - CONTENTS_ROW).cloned()
                        {
                            input.set(option);
                        }
                    }
                    Some(0) => {
                        self.setting_directory = None;
                        self.renaming = Some(String::new());
                    }
                    Some(DIRECTORY_ROW) => self.start_setting_directory(),
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
                if self.setting_directory.is_some() {
                    if !self.handle_directory_key(key, modifiers) {
                        return Some(ui_event);
                    }
                    self.hook.damage_window();
                    return None;
                }
                match (&mut self.renaming, key) {
                    (None, KeySymbol::Char('d')) if modifiers == KeyModifiers::NONE => {
                        self.start_setting_directory();
                    }
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
        let text = settings::get().palette.fg;
        let title = match &self.renaming {
            Some(name) => format!("{name}▏"),
            None => self.name.clone(),
        };
        let (directory, hint) = match (&self.renaming, &self.setting_directory) {
            (_, Some(input)) => (
                (format!("directory: {}▏", input.path), text),
                if input.invalid {
                    ("Not a directory", ERROR)
                } else {
                    (
                        "Up/Down + Tab or click: complete, Enter: save (empty clears), Esc: cancel",
                        HINT,
                    )
                },
            ),
            (renaming, None) => (
                (
                    format!(
                        "directory: {}",
                        self.holder.directory.as_ref().map_or_else(
                            || "none (the runner's)".to_string(),
                            |directory| tilde(directory),
                        )
                    ),
                    HINT,
                ),
                if renaming.is_some() {
                    ("Enter: save, Esc: cancel", HINT)
                } else {
                    ("r: rename, d: set directory, 1-9: select", HINT)
                },
            ),
        };
        let below: Vec<(String, Color)> = match &self.setting_directory {
            Some(input) => input
                .options
                .iter()
                .take(MAX_OPTIONS)
                .enumerate()
                .map(|(index, option)| {
                    if input.selected == Some(index) {
                        (format!("› {option}"), settings::get().palette.accent)
                    } else {
                        (format!("  {option}"), text)
                    }
                })
                .collect(),
            None => (1..)
                .zip(tree.children(tree.root()))
                .map(|(index, &id)| (format!("{index}  {}", tree.label(id)), text))
                .chain(NEW_ROWS.map(|(label, _, _)| (label.to_string(), text)))
                .collect(),
        };
        let rows = [(title, text), directory, (hint.0.to_string(), hint.1)]
            .into_iter()
            .chain(below);
        let rows = (0..)
            .zip(rows)
            .map(|(index, (text, color))| {
                UITree::leaf_styled(
                    UIElement::Text(vec![(text, UIElement::glyphon_attr(color))]),
                    Style::absolute(
                        [PAD, PAD + index as f32 * row_height()],
                        [(width - PAD).max(0.0), row_height()],
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
            | StandardEvent::Focus(_)
            | StandardEvent::ToggleCollapsed(_) => {}
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

    fn toggle_collapsed(&mut self, path: &WorldTreePath) {
        if !self.dissolved {
            self.holder.toggle_collapsed(path);
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
        position: SpawnPosition,
        tree: DetachedTree,
    ) -> Answer<Result<(), DetachedTree>> {
        if self.dissolved {
            return Answer::Now(Err(tree));
        }
        self.holder.start_place(Some(path), position, tree);
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
            contents.directory.clone_from(&self.holder.directory);
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
