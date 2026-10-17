use crate::{
    help::HelpApplet,
    nodular_applet::{
        AppletSpawner, NodeRequest, NodularAppletInitializer,
        node_holder::{NodeHolder, client_spawners},
        node_tree::{NodeId, NodeTree},
        status_bar::{self, STATUS_BAR_HEIGHT},
        treeview::{TreeviewAction, TreeviewPointer, TreeviewRow, treeview_rows},
    },
    session,
};
use calloop::{
    EventLoop, LoopHandle,
    channel::{Channel, Sender},
    timer::{TimeoutAction, Timer},
};
use singularity_common::{
    sap::{
        packets::{StandardEvent, StandardRequest, StateKind, Treeview},
        raw_client_initializer::RawClientInitializer,
        session::Session,
        shared_state::{SharedState, StatePublisher},
    },
    utils::tree::world_tree::WorldTreePath,
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{TrackSize, UITree},
    ui_event::{KeyModifiers, KeySymbol, MouseEvent, UIEvent},
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

/// Holds the tree of nodes, is held by a Basic Applet runner (applet runner).
///
/// Every applet in the tree is a client of this, on its own thread.
pub struct RootNodeApplet {
    holder: NodeHolder,
    surface: StatePublisher<UITree>,
    request_sender: Sender<StandardRequest>,

    latest_size: DisplayContainerSize,

    treeview_pointer: TreeviewPointer,
    /// The last treeview drop and the node it was on.
    /// A drop can come before its drag's pluck finishes, which can shift the paths.
    last_drop: Option<(WorldTreePath, NodeId)>,
    /// The help Super+? opened, to close it on the next one
    help: Option<OpenHelp>,
    /// See `TREEVIEW_ONLY_WHILE_TRAVERSING`
    treeview_shown: bool,
    /// Shown in the strip at the top, above the treeview and the tree
    status_text: String,
    /// A wl client is capturing the screen (eg recording)
    capturing: bool,
    /// The bar's `REC`, which flashes while capturing
    rec_shown: bool,
    /// Where the session is saved, if anywhere
    session_path: Option<PathBuf>,
    /// The last one written, so unchanged ones aren't written again
    saved_session: Option<Session>,
}

/// A help node, and where it was opened
struct OpenHelp {
    node: NodeId,
    parent: NodeId,
    /// Focus was inside the parent's applet, so it goes back there when the help closes
    parent_inner: bool,
}
impl RootNodeApplet {
    /// Of the width
    const TREEVIEW_PERCENT: u32 = 20;
    /// How often the session is saved (if it changed), besides before quitting
    const SAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(2);
    /// Show the treeview only while traversing (focus is on a node or inner element, not inside an applet).
    /// Otherwise the applets get the full width.
    const TREEVIEW_ONLY_WHILE_TRAVERSING: bool = true;

    /// 0 while hidden
    fn treeview_width(&self) -> u32 {
        if self.treeview_shown {
            self.latest_size.width * Self::TREEVIEW_PERCENT / 100
        } else {
            0
        }
    }

    /// Right of the treeview
    fn inner_applet_size(&self) -> DisplayContainerSize {
        DisplayContainerSize::new(
            self.latest_size.width - self.treeview_width(),
            self.latest_size.height.saturating_sub(STATUS_BAR_HEIGHT),
        )
    }

    /// Shows or hides the treeview, resizing the applets
    fn update_treeview_visibility(&mut self) {
        let shown = !Self::TREEVIEW_ONLY_WHILE_TRAVERSING
            || self.holder.tree.is_traversing()
            // eg a drag that started there
            || self.treeview_pointer.is_pressed();
        if shown != self.treeview_shown {
            self.treeview_shown = shown;
            self.surface.damage();
            self.holder
                .tree
                .handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                    self.inner_applet_size(),
                )));
        }
    }

    /// Publishes the window if damaged and the runner read the last one
    fn flush_surface(&mut self) {
        let treeview_width = self.treeview_width();
        let Self {
            holder,
            treeview_pointer,
            status_text,
            rec_shown,
            ..
        } = self;
        self.surface.flush(
            || {
                Self::get_window(
                    &holder.tree,
                    treeview_pointer,
                    treeview_width,
                    status_text,
                    *rec_shown,
                )
            },
            &self.request_sender,
        );
    }

    fn send_request(&self, request: StandardRequest) {
        self.request_sender.send(request).unwrap();
    }

    /// Writes the session if it changed since the last write
    fn save_session(&mut self) {
        let Some(path) = &self.session_path else {
            return;
        };
        let session = Session {
            tree: self.holder.tree.save(),
            focus: self.holder.tree.get_focus_path().0.into_vec(),
        };
        if self.saved_session.as_ref() == Some(&session) {
            return;
        }
        match session::write(path, &session) {
            Ok(()) => self.saved_session = Some(session),
            Err(err) => log::warn!("Couldn't save the session to {}: {err}", path.display()),
        }
    }

    /// Saves first, since the runner may end the process right after
    fn quit(&mut self) {
        self.save_session();
        self.send_request(StandardRequest::Quit);
    }

    /// The saved session if there is one, else `root_initializer` at the root
    #[must_use]
    fn new(
        root_initializer: NodularAppletInitializer,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        session_path: Option<PathBuf>,
        surface: SharedState<UITree>,
        request_sender: Sender<StandardRequest>,
        event_loop: &LoopHandle<'static, Self>,
    ) -> Self {
        let applet_spawners = Arc::new(RwLock::new(applet_spawner_registry));
        let (spawn_client, attach_client) = client_spawners(
            event_loop,
            applet_spawners,
            |root: &mut Self, id, request| {
                root.handle_node_request(id, request);
            },
        );
        let saved_session = session_path.as_deref().and_then(session::load);
        let tree = match &saved_session {
            Some(saved) => {
                let mut tree = NodeTree::restore(saved.tree.clone(), spawn_client, attach_client);
                tree.focus_path(&WorldTreePath(saved.focus.clone().into_boxed_slice()));
                tree
            }
            None => NodeTree::new(root_initializer, spawn_client, attach_client),
        };
        let send = {
            let request_sender = request_sender.clone();
            // the runner may already be gone while shutting down
            Box::new(move |request| {
                let _ = request_sender.send(request);
            })
        };

        Self {
            holder: NodeHolder::new(tree, send, false),
            surface: StatePublisher::new(surface),
            request_sender,
            // Just making stuff up
            latest_size: DisplayContainerSize::new(800, 600),
            treeview_pointer: TreeviewPointer::default(),
            last_drop: None,
            help: None,
            // the root node starts focused
            treeview_shown: true,
            status_text: String::new(),
            capturing: false,
            rec_shown: false,
            session_path,
            saved_session,
        }
    }

    /// Restores the session at `session_path` if there is one (else starts with `root_initializer` at the root),
    /// and keeps it saved there
    pub fn get_initializer(
        root_initializer: NodularAppletInitializer,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        session_path: Option<PathBuf>,
    ) -> Box<dyn RawClientInitializer> {
        struct Initializer {
            root_initializer: NodularAppletInitializer,
            applet_spawner_registry: BTreeMap<String, AppletSpawner>,
            session_path: Option<PathBuf>,
        }
        impl RawClientInitializer for Initializer {
            fn init(
                // smth smth box needs to know size
                self: Box<Self>,
                surface: SharedState<UITree>,
                // the treeview is drawn into the surface
                _treeview: SharedState<Treeview>,
                event_queue: Channel<StandardEvent>,
                // yeah, ik the naming is inconsistent bc I'm not saying "event_receiver" or "event_rx", but it's calm
                // (I am really trying to convince myself this is fine, I am the strawman)
                request_sender: Sender<StandardRequest>,
            ) {
                let mut event_loop = EventLoop::try_new().unwrap();

                event_loop
                    .handle()
                    .insert_source(event_queue, |event, &mut (), root: &mut RootNodeApplet| {
                        let calloop::channel::Event::Msg(event) = event else {
                            return;
                        };
                        root.handle_event(event);
                    })
                    .unwrap();

                let (status_sender, status_receiver) = calloop::channel::channel();
                event_loop
                    .handle()
                    .insert_source(
                        status_receiver,
                        |event, &mut (), root: &mut RootNodeApplet| {
                            if let calloop::channel::Event::Msg(text) = event {
                                root.status_text = text;
                                root.surface.damage();
                            }
                        },
                    )
                    .unwrap();
                status_bar::spawn_ticker(status_sender);
                event_loop
                    .handle()
                    .insert_source(
                        Timer::from_duration(status_bar::REC_FLASH),
                        |_, &mut (), root: &mut RootNodeApplet| {
                            if root.capturing {
                                root.rec_shown = !root.rec_shown;
                                root.surface.damage();
                            }
                            TimeoutAction::ToDuration(status_bar::REC_FLASH)
                        },
                    )
                    .unwrap();
                event_loop
                    .handle()
                    .insert_source(
                        Timer::from_duration(RootNodeApplet::SAVE_INTERVAL),
                        |_, &mut (), root: &mut RootNodeApplet| {
                            root.save_session();
                            TimeoutAction::ToDuration(RootNodeApplet::SAVE_INTERVAL)
                        },
                    )
                    .unwrap();

                let mut root = RootNodeApplet::new(
                    self.root_initializer,
                    self.applet_spawner_registry,
                    self.session_path,
                    surface,
                    request_sender,
                    &event_loop.handle(),
                );

                // after each batch of events, since any of them may move focus or damage
                event_loop
                    .run(None, &mut root, |root| {
                        root.holder.finish();
                        if std::mem::take(&mut root.holder.quit) {
                            root.quit();
                        }
                        root.holder.tree.update_view_root();
                        root.damage_if_tree_changed();
                        root.update_treeview_visibility();
                        root.holder.tree.update_layout();
                        root.holder.update_wl_focus();
                        root.flush_surface();
                    })
                    .unwrap();
            }
        }
        Box::new(Initializer {
            root_initializer,
            applet_spawner_registry,
            session_path,
        })
    }

    /// From the runner
    fn handle_event(&mut self, event: StandardEvent) {
        match event {
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                self.latest_size = size;
                self.surface.damage();
                self.holder
                    .tree
                    .handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                        self.inner_applet_size(),
                    )));
            }
            StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)) => self.handle_mouse(mouse_event),
            StandardEvent::DamageAck(StateKind::Surface) => self.surface.ack(),
            // the treeview is drawn into the surface
            StandardEvent::DamageAck(StateKind::Treeview) => {}
            // the root can't close without ending everything
            StandardEvent::CloseRequest => self.quit(),
            StandardEvent::WlSurfaceRegistered(surface) => {
                // a restored tile, else where the user is working (eg under the command hub that ran it)
                self.holder.add_wl_surface(surface);
            }
            // TODO: surfaces inside projects
            StandardEvent::FocusWlSurface(surface_id) => {
                self.holder.tree.focus_wl_surface(surface_id);
            }
            // its tile closes like an applet closing itself
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                self.holder.close_wl_surface(surface_id);
            }
            StandardEvent::WlSurfaceTitle(surface_id, title) => {
                self.holder.set_wl_surface_title(surface_id, title);
            }
            StandardEvent::Pluck(path) => self.holder.start_pluck(path.as_ref()),
            StandardEvent::Place(path, tree) => {
                let path = path.map(|path| self.redirect_drop(path));
                self.holder.start_place(path.as_ref(), tree);
            }
            StandardEvent::Focus(path) => self.holder.focus(&path),
            StandardEvent::Capturing(capturing) => {
                self.capturing = capturing;
                self.rec_shown = capturing;
                self.surface.damage();
            }
            StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(KeySymbol::Char('?')),
                modifiers: KeyModifiers { logo: true, .. },
                pressed: true,
                ..
            }) => self.toggle_help(),
            StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(KeySymbol::Char('v' | 'V')),
                modifiers: KeyModifiers { logo: true, .. },
                pressed: true,
                ..
            }) => self.holder.tree.toggle_view_root(),
            StandardEvent::UIEvent(_)
            | StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..) => {
                // the root is never a project, so nothing is passed up
                let _ = self.holder.handle_ui_event(event);
            }
        }

        self.damage_if_tree_changed();
    }

    /// Closes the help, or opens it as a child of the focused node (so it shows there), focused
    fn toggle_help(&mut self) {
        let tree = &mut self.holder.tree;
        if let Some(help) = self.help.take()
            && tree.close(help.node)
        {
            self.help_closed(&help);
            return;
        }
        let parent = tree.focused_node();
        let parent_inner = tree.is_inner(parent);
        if let Some(node) = tree.add_child(parent, HelpApplet::get_boxed_initiator()) {
            tree.focus(node, true);
            self.help = Some(OpenHelp {
                node,
                parent,
                parent_inner,
            });
        }
    }

    /// Focus goes back inside the applet it was opened from, if it was there and the help had focus
    fn help_closed(&mut self, help: &OpenHelp) {
        let tree = &mut self.holder.tree;
        if help.parent_inner && tree.focused_node() == help.parent {
            tree.focus(help.parent, true);
        }
    }

    fn handle_mouse(&mut self, mut mouse_event: MouseEvent) {
        // 0 while the treeview is hidden
        let left = self.treeview_width();
        // the bar is above everything else
        let top = f64::from(STATUS_BAR_HEIGHT);
        if mouse_event.position[1] < top {
            return;
        }
        mouse_event.position[1] -= top;
        let in_treeview = mouse_event.position[0] < f64::from(left);
        // a hidden treeview has no rows to hit
        let rows = if self.treeview_shown {
            view_rows(&self.holder.tree)
        } else {
            Vec::new()
        };
        let response = self
            .treeview_pointer
            .handle_mouse(&mouse_event, in_treeview, &rows);

        if response.damaged {
            self.surface.damage();
        }
        match response.action {
            Some(TreeviewAction::Focus(path)) => self.holder.focus(&path),
            Some(TreeviewAction::StartDrag(path)) => {
                self.send_request(StandardRequest::RequestPluck(path));
            }
            Some(TreeviewAction::Drop(path)) => {
                let path = path.unwrap_or_else(|| self.holder.tree.get_focus_path());
                self.last_drop = path
                    .0
                    .first()
                    .and_then(|node_path| self.holder.tree.node_at(node_path))
                    .map(|id| (path.clone(), id));
                self.send_request(StandardRequest::RequestPlace(path));
            }
            None => {}
        }
        if response.consumed {
            return;
        }

        // remap window coords -> tree coords
        mouse_event.position[0] -= f64::from(left);
        self.holder
            .tree
            .handle_event(StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)));
    }

    /// If this is the last drop, the path to the node it was on now.
    /// That node was plucked too if it was dropped onto the dragged part, so it goes to the focus instead.
    fn redirect_drop(&mut self, path: WorldTreePath) -> WorldTreePath {
        match self.last_drop.take() {
            Some((dropped, id)) if dropped == path => self
                .holder
                .full_path(id, inner_path(&path))
                .unwrap_or_else(|| self.holder.tree.get_focus_path()),
            _ => path,
        }
    }

    /// From an applet in the tree
    fn handle_node_request(&mut self, id: NodeId, request: NodeRequest) {
        let closes_help = matches!(request, NodeRequest::Close)
            && self.help.as_ref().is_some_and(|help| help.node == id);
        // nothing is above the root
        if self.holder.handle_node_request(id, request).is_some() {
            log::warn!("The root holder has no node to pass a request to");
        }
        if closes_help && let Some(help) = self.help.take() {
            self.help_closed(&help);
        }
        self.damage_if_tree_changed();
    }

    fn damage_if_tree_changed(&mut self) {
        if self.holder.tree.take_damage() {
            self.surface.damage();
        }
    }

    fn get_window(
        tree: &NodeTree,
        treeview_pointer: &TreeviewPointer,
        // 0 while hidden
        treeview_width: u32,
        status_text: &str,
        rec_shown: bool,
    ) -> UITree {
        let content = if treeview_width == 0 {
            tree.get_window()
        } else {
            Self::get_window_with_treeview(tree, treeview_pointer, treeview_width)
        };
        UITree::new_container_styled(
            sonamu_ui::ui_element::Style {
                display: sonamu_ui::ui_element::Display::Grid,
                grid_template_columns: vec![TrackSize::Fr(1.0)],
                grid_template_rows: vec![
                    TrackSize::Px(STATUS_BAR_HEIGHT as f32),
                    TrackSize::Fr(1.0),
                ],
                ..sonamu_ui::ui_element::Style::full_size()
            },
            vec![status_bar::render(status_text, rec_shown), content],
        )
    }

    fn get_window_with_treeview(
        tree: &NodeTree,
        treeview_pointer: &TreeviewPointer,
        treeview_width: u32,
    ) -> UITree {
        let rows = view_rows(tree);
        let treeview =
            treeview_pointer.render(&rows, treeview_width as f32, &tree.get_focus_path());

        UITree::new_container_styled(
            sonamu_ui::ui_element::Style {
                display: sonamu_ui::ui_element::Display::Grid,
                // in px, like the applets' sizes
                grid_template_columns: vec![
                    TrackSize::Px(treeview_width as f32),
                    TrackSize::Fr(1.0),
                ],
                grid_template_rows: vec![TrackSize::Fr(1.0)],
                ..sonamu_ui::ui_element::Style::full_size()
            },
            vec![treeview, tree.get_window()],
        )
    }
}

/// The treeview's rows: the view root's subtree
fn view_rows(tree: &NodeTree) -> Vec<TreeviewRow> {
    treeview_rows(
        &tree.get_view_treeview(),
        &tree.view_root_path(),
        tree.view_parent(),
    )
}

/// The part of the path inside the node's applet
fn inner_path(path: &WorldTreePath) -> WorldTreePath {
    WorldTreePath(path.0.get(1..).unwrap_or_default().into())
}
