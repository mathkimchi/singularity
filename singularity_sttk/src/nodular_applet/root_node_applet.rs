use crate::{
    help::HelpApplet,
    nodular_applet::{
        AppletClient, AppletSpawner, NodeRequest, NodularAppletInitializer,
        node_tree::{NodeId, NodeTree},
        treeview::{TreeviewAction, TreeviewPointer, TreeviewRow, treeview_rows},
    },
};
use calloop::{
    EventLoop, LoopHandle,
    channel::{Channel, Sender},
};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        detached::{DetachedClient, DetachedTree},
        packets::{StandardEvent, StandardRequest, StateKind, WlSurfaceId},
        raw_client_initializer::RawClientInitializer,
        shared_state::{SharedState, StatePublisher},
    },
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{TrackSize, UITree},
    ui_event::{KeyModifiers, KeySymbol, MouseEvent, UIEvent},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

/// A pluck or place the root is waiting on a node's applet for
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    Pluck(NodeId),
    Place(NodeId),
}

/// Holds the tree of nodes, is held by a Basic Applet runner (applet runner).
///
/// Every applet in the tree is a client of this, on its own thread.
pub struct RootNodeApplet {
    tree: NodeTree,
    surface: StatePublisher<UITree>,
    request_sender: Sender<StandardRequest>,

    latest_size: DisplayContainerSize,
    /// Last sent with `StandardRequest::WlKeyboardFocus`
    focused_wl_surface: Option<WlSurfaceId>,

    /// At most one at a time, since the runner sends one at a time
    pending: Option<Pending>,
    /// Plucked after the event batch: detaching a client inside its own request callback panics
    to_pluck: Option<NodeId>,
    treeview_pointer: TreeviewPointer,
    /// The last treeview drop and the node it was on.
    /// A drop can come before its drag's pluck finishes, which can shift the paths.
    last_drop: Option<(WorldTreePath, NodeId)>,
    /// The help Super+? opened, to close it on the next one
    help: Option<OpenHelp>,
    /// See `TREEVIEW_ONLY_WHILE_TRAVERSING`
    treeview_shown: bool,
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
    /// Show the treeview only while traversing (a node itself is focused, not inside its applet).
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
            self.latest_size.height,
        )
    }

    /// Shows or hides the treeview, resizing the applets
    fn update_treeview_visibility(&mut self) {
        let shown = !Self::TREEVIEW_ONLY_WHILE_TRAVERSING
            || !self.tree.is_inner(self.tree.focused_node())
            // eg a drag that started there
            || self.treeview_pointer.is_pressed();
        if shown != self.treeview_shown {
            self.treeview_shown = shown;
            self.surface.damage();
            self.tree
                .handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                    self.inner_applet_size(),
                )));
        }
    }

    /// Tells the runner if Sonamu focus moved onto or off a wl surface
    fn update_wl_focus(&mut self) {
        let focused_wl_surface = self.tree.focused_wl_surface();
        if focused_wl_surface != self.focused_wl_surface {
            self.focused_wl_surface = focused_wl_surface;
            self.send_request(StandardRequest::WlKeyboardFocus(focused_wl_surface));
        }
    }

    /// Publishes the window if damaged and the runner read the last one
    fn flush_surface(&mut self) {
        let treeview_width = self.treeview_width();
        let Self {
            tree,
            treeview_pointer,
            ..
        } = self;
        self.surface.flush(
            || Self::get_window(tree, treeview_pointer, treeview_width),
            &self.request_sender,
        );
    }

    fn send_request(&self, request: StandardRequest) {
        self.request_sender.send(request).unwrap();
    }

    #[must_use]
    fn new(
        root_initializer: NodularAppletInitializer,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        surface: SharedState<UITree>,
        request_sender: Sender<StandardRequest>,
        event_loop: &LoopHandle<'static, Self>,
    ) -> Self {
        let applet_spawners = Arc::new(RwLock::new(applet_spawner_registry));
        let spawn_applet = {
            let event_loop = event_loop.clone();
            Box::new(move |id, initializer| {
                ClientHandle::spawn_new_client(
                    Box::new(AppletClient {
                        initializer,
                        applet_spawners: applet_spawners.clone(),
                    }),
                    &event_loop,
                    move |request, root: &mut Self| root.handle_node_request(id, request),
                )
            })
        };
        let attach_applet = {
            let event_loop = event_loop.clone();
            Box::new(move |id, client: DetachedClient| {
                client.attach(&event_loop, move |request, root: &mut Self| {
                    root.handle_node_request(id, request);
                })
            })
        };

        Self {
            tree: NodeTree::new(root_initializer, spawn_applet, attach_applet),
            surface: StatePublisher::new(surface),
            request_sender,
            // Just making stuff up
            latest_size: DisplayContainerSize::new(800, 600),
            focused_wl_surface: None,
            pending: None,
            to_pluck: None,
            treeview_pointer: TreeviewPointer::default(),
            last_drop: None,
            help: None,
            // the root node starts focused
            treeview_shown: true,
        }
    }

    pub fn get_initializer(
        root_initializer: NodularAppletInitializer,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
    ) -> Box<dyn RawClientInitializer> {
        struct Initializer {
            root_initializer: NodularAppletInitializer,
            applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        }
        impl RawClientInitializer for Initializer {
            fn init(
                // smth smth box needs to know size
                self: Box<Self>,
                surface: SharedState<UITree>,
                // the treeview is drawn into the surface
                _treeview: SharedState<WorldTree<String>>,
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

                let mut root = RootNodeApplet::new(
                    self.root_initializer,
                    self.applet_spawner_registry,
                    surface,
                    request_sender,
                    &event_loop.handle(),
                );

                // after each batch of events, since any of them may move focus or damage
                event_loop
                    .run(None, &mut root, |root| {
                        root.finish_pluck();
                        root.tree.update_view_root();
                        root.damage_if_tree_changed();
                        root.update_treeview_visibility();
                        root.tree.update_layout();
                        root.update_wl_focus();
                        root.flush_surface();
                    })
                    .unwrap();
            }
        }
        Box::new(Initializer {
            root_initializer,
            applet_spawner_registry,
        })
    }

    /// From the runner
    fn handle_event(&mut self, event: StandardEvent) {
        match event {
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                self.latest_size = size;
                self.surface.damage();
                self.tree
                    .handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(
                        self.inner_applet_size(),
                    )));
            }
            StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)) => self.handle_mouse(mouse_event),
            StandardEvent::DamageAck(StateKind::Surface) => self.surface.ack(),
            // the treeview is drawn into the surface
            StandardEvent::DamageAck(StateKind::Treeview) => {}
            // the root can't close without ending everything
            StandardEvent::CloseRequest => self.send_request(StandardRequest::Quit),
            StandardEvent::WlSurfaceRegistered {
                surface_id,
                wl_event_queue,
            } => {
                // where the user is working (eg under the command hub that ran it)
                self.tree
                    .add_wl_surface(self.tree.focused_node(), surface_id, wl_event_queue);
            }
            StandardEvent::FocusWlSurface(surface_id) => self.tree.focus_wl_surface(surface_id),
            // its tile closes like an applet closing itself
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                self.tree.close_wl_surface(surface_id);
            }
            StandardEvent::Pluck(path) => {
                let path = path.unwrap_or_else(|| self.tree.get_focus_path());
                self.start_pluck(&path);
            }
            StandardEvent::Place(path, tree) => {
                let path = match path {
                    Some(path) => self.redirect_drop(path),
                    None => self.tree.get_focus_path(),
                };
                self.start_place(&path, tree);
            }
            StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(KeySymbol::Char('?')),
                modifiers: KeyModifiers { logo: true, .. },
                pressed: true,
                ..
            }) => self.toggle_help(),
            StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(KeySymbol::Char('p' | 'P')),
                modifiers: KeyModifiers { logo: true, .. },
                pressed: true,
                ..
            }) => self.tree.toggle_project(self.tree.focused_node()),
            StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(KeySymbol::Char('v' | 'V')),
                modifiers: KeyModifiers { logo: true, .. },
                pressed: true,
                ..
            }) => self.tree.toggle_view_root(),
            StandardEvent::UIEvent(_)
            | StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..) => self.tree.handle_event(event),
        }

        self.damage_if_tree_changed();
    }

    /// Closes the help, or opens it as a child of the focused node (so it shows there), focused
    fn toggle_help(&mut self) {
        if let Some(help) = self.help.take()
            && self.tree.close(help.node)
        {
            self.help_closed(&help);
            return;
        }
        let parent = self.tree.focused_node();
        let parent_inner = self.tree.is_inner(parent);
        if let Some(node) = self
            .tree
            .add_child(parent, HelpApplet::get_boxed_initiator())
        {
            self.tree.focus(node, true);
            self.help = Some(OpenHelp {
                node,
                parent,
                parent_inner,
            });
        }
    }

    /// Focus goes back inside the applet it was opened from, if it was there and the help had focus
    fn help_closed(&mut self, help: &OpenHelp) {
        if help.parent_inner && self.tree.focused_node() == help.parent {
            self.tree.focus(help.parent, true);
        }
    }

    fn handle_mouse(&mut self, mut mouse_event: MouseEvent) {
        // 0 while the treeview is hidden
        let left = self.treeview_width();
        let in_treeview = mouse_event.position[0] < f64::from(left);
        // a hidden treeview has no rows to hit
        let rows = if self.treeview_shown {
            view_rows(&self.tree)
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
            Some(TreeviewAction::Focus(path)) => self.tree.focus_path(&path),
            Some(TreeviewAction::StartDrag(path)) => {
                self.send_request(StandardRequest::RequestPluck(path));
            }
            Some(TreeviewAction::Drop(path)) => {
                let path = path.unwrap_or_else(|| self.tree.get_focus_path());
                self.last_drop = path
                    .0
                    .first()
                    .and_then(|node_path| self.tree.node_at(node_path))
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
        self.tree
            .handle_event(StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)));
    }

    /// If this is the last drop, the path to the node it was on now.
    /// That node was plucked too if it was dropped onto the dragged part, so it goes to the focus instead.
    fn redirect_drop(&mut self, path: WorldTreePath) -> WorldTreePath {
        match self.last_drop.take() {
            Some((dropped, id)) if dropped == path => self
                .full_path(id, inner_path(&path))
                .unwrap_or_else(|| self.tree.get_focus_path()),
            _ => path,
        }
    }

    /// `path[0]` is the node, the rest is inside its applet
    fn start_pluck(&mut self, path: &WorldTreePath) {
        let id = match path.0.first() {
            None => Some(self.tree.root()),
            Some(node_path) => self.tree.node_at(node_path),
        };
        let Some(id) = id else {
            self.send_request(StandardRequest::Plucked(None));
            return;
        };

        // the applet may have something closer; it answers with `Plucked`.
        // Without a running applet (eg a split) it would never answer.
        if path.0.len() > 1
            && self
                .tree
                .try_send_event(id, StandardEvent::Pluck(Some(inner_path(path))))
                .is_ok()
        {
            self.pending = Some(Pending::Pluck(id));
        } else {
            self.to_pluck = Some(id);
        }
    }

    /// Plucks the node chosen by `start_pluck` or its applet's `Plucked(None)`
    fn finish_pluck(&mut self) {
        if let Some(id) = self.to_pluck.take() {
            // `None` for the root
            let plucked = self.tree.pluck(id);
            self.send_request(StandardRequest::Plucked(plucked));
            self.damage_if_tree_changed();
        }
    }

    /// Places at the deepest node along `path[0]`, asking its applet first if the path goes into it
    fn start_place(&mut self, path: &WorldTreePath, mut tree: DetachedTree) {
        let (id, exact) = path
            .0
            .first()
            .map_or((self.tree.root(), true), |node_path| {
                self.tree.closest_node_at(node_path)
            });

        if exact && path.0.len() > 1 {
            match self
                .tree
                .try_send_event(id, StandardEvent::Place(Some(inner_path(path)), tree))
            {
                Ok(()) => {
                    self.pending = Some(Pending::Place(id));
                    return;
                }
                // no running applet (eg a split), so the node takes it
                Err(StandardEvent::Place(_, given_back)) => tree = given_back,
                Err(_) => unreachable!("the event is given back as is"),
            }
        }
        self.tree.place(id, tree);
        self.send_request(StandardRequest::Placed(None));
    }

    /// From an applet in the tree
    fn handle_node_request(&mut self, id: NodeId, request: NodeRequest) {
        self.apply_node_request(id, request);
        self.damage_if_tree_changed();
    }

    fn damage_if_tree_changed(&mut self) {
        if self.tree.take_damage() {
            self.surface.damage();
        }
    }

    fn apply_node_request(&mut self, id: NodeId, request: NodeRequest) {
        match request {
            // the treeview is drawn into the surface, so both damage it
            NodeRequest::Standard(StandardRequest::Damage(_)) => self.surface.damage(),
            // eg Ctrl+Shift+Q; quits everything
            NodeRequest::Standard(StandardRequest::Quit) => {
                self.send_request(StandardRequest::Quit);
            }
            // only sent by the root itself
            NodeRequest::Standard(StandardRequest::WlKeyboardFocus(_)) => {}
            NodeRequest::Standard(StandardRequest::Plucked(plucked)) => {
                if self.pending != Some(Pending::Pluck(id)) {
                    log::warn!("Unexpected `Plucked` from {id:?}; dropping it");
                    return;
                }
                self.pending = None;
                match plucked {
                    Some(plucked) => self.send_request(StandardRequest::Plucked(Some(plucked))),
                    // nothing inside, so the node itself
                    None => self.to_pluck = Some(id),
                }
            }
            NodeRequest::Standard(StandardRequest::Placed(given_back)) => {
                if self.pending != Some(Pending::Place(id)) {
                    log::warn!("Unexpected `Placed` from {id:?}; dropping it");
                    return;
                }
                self.pending = None;
                // the applet couldn't, so it becomes the node's child
                if let Some(tree) = given_back {
                    self.tree.place(id, tree);
                }
                self.send_request(StandardRequest::Placed(None));
            }
            // the applet's path is inside its inner tree
            NodeRequest::Standard(StandardRequest::RequestPluck(path)) => {
                if let Some(path) = self.full_path(id, path) {
                    self.send_request(StandardRequest::RequestPluck(path));
                }
            }
            NodeRequest::Standard(StandardRequest::RequestPlace(path)) => {
                if let Some(path) = self.full_path(id, path) {
                    self.send_request(StandardRequest::RequestPlace(path));
                }
            }
            NodeRequest::AddChild(initializer) => {
                self.tree.add_child(id, initializer);
            }
            NodeRequest::ChangeFocus(operation) => self.tree.change_focus(id, operation),
            NodeRequest::Close => {
                if self.tree.close(id)
                    && let Some(help) = self.help.take_if(|help| help.node == id)
                {
                    self.help_closed(&help);
                }
            }
            NodeRequest::Split { axis, new_pane } => self.tree.split(id, axis, new_pane),
            NodeRequest::SetProject(project) => self.tree.set_project(id, project),
        }
    }

    /// A path inside the node's applet, as a path of the root's treeview
    fn full_path(&self, id: NodeId, inner: WorldTreePath) -> Option<WorldTreePath> {
        let mut path = vec![self.tree.path_of(id)?];
        path.extend(inner.0);
        Some(WorldTreePath(path.into_boxed_slice()))
    }

    fn get_window(
        tree: &NodeTree,
        treeview_pointer: &TreeviewPointer,
        // 0 while hidden
        treeview_width: u32,
    ) -> UITree {
        if treeview_width == 0 {
            return tree.get_window();
        }
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
