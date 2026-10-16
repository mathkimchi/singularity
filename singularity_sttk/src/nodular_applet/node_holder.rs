//! What the root and projects share: holding a `NodeTree` of clients, and pluck and place, node requests
//! and wl surfaces on top of it.

use crate::nodular_applet::{
    AppletClient, AppletSpawnerRegistry, NodeRequest, WrapInitializer,
    node_tree::{AttachClient, NodeId, NodeTree, SpawnClient},
};
use calloop::{LoopHandle, channel::Sender};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        detached::{DetachedClient, DetachedTree},
        packets::{StandardEvent, StandardRequest, WlSurfaceEvent, WlSurfaceId},
    },
    utils::tree::world_tree::WorldTreePath,
};

/// A pluck or place the holder is waiting on a node's applet for
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    Pluck(NodeId),
    Place(NodeId),
}

/// Runs and attaches applets on `event_loop`; their requests go to `on_request`, tagged with their node
pub fn client_spawners<Data: 'static>(
    event_loop: &LoopHandle<'static, Data>,
    applet_spawners: AppletSpawnerRegistry,
    on_request: fn(&mut Data, NodeId, NodeRequest),
) -> (SpawnClient, AttachClient) {
    let spawn_client: SpawnClient = {
        let event_loop = event_loop.clone();
        Box::new(move |id, initializer| {
            let client = Box::new(AppletClient {
                initializer,
                applet_spawners: applet_spawners.clone(),
            });
            ClientHandle::spawn_new_client(client, &event_loop, move |request, data: &mut Data| {
                on_request(data, id, request);
            })
        })
    };
    let attach_client: AttachClient = {
        let event_loop = event_loop.clone();
        Box::new(move |id, client: DetachedClient| {
            client.attach(&event_loop, move |request, data: &mut Data| {
                on_request(data, id, request);
            })
        })
    };
    (spawn_client, attach_client)
}

/// Holds a tree of nodes, for the root (whose node is the runner) or an applet like a project (whose node is in another tree).
/// Its node doesn't know: it only sees the holder's published state and requests.
///
/// Paths it is given and sends are of its tree.
pub struct NodeHolder {
    pub tree: NodeTree,
    /// Sends to the holder's node
    send: Box<dyn Fn(StandardRequest)>,
    /// Its tree is a project's contents, whose root (a container) stands for the project itself
    project: bool,

    /// At most one at a time, since the holder's node sends one at a time
    pending: Option<Pending>,
    /// Plucked after the event batch: detaching a client inside its own request callback panics
    to_pluck: Option<NodeId>,
    /// Wrapped after the event batch, for the same reason
    to_wrap: Option<(NodeId, WrapInitializer)>,
    /// Last sent with `StandardRequest::WlKeyboardFocus`
    focused_wl_surface: Option<WlSurfaceId>,
    /// A project's root was closed, so the project should give its contents back
    pub dissolve: bool,
}
impl NodeHolder {
    pub fn new(tree: NodeTree, send: Box<dyn Fn(StandardRequest)>, project: bool) -> Self {
        Self {
            tree,
            send,
            project,
            pending: None,
            to_pluck: None,
            to_wrap: None,
            focused_wl_surface: None,
            dissolve: false,
        }
    }

    fn send(&self, request: StandardRequest) {
        (self.send)(request);
    }

    /// A path inside the node's applet, as a path of the holder's treeview
    pub fn full_path(&self, id: NodeId, inner: WorldTreePath) -> Option<WorldTreePath> {
        let mut path = vec![self.tree.path_of(id)?];
        path.extend(inner.0);
        Some(WorldTreePath(path.into_boxed_slice()))
    }

    pub fn focus(&mut self, path: &WorldTreePath) {
        self.tree.focus_path(path);
    }

    /// Plucks at `path` (`None`: at the focus) and answers with `Plucked`.
    /// `path[0]` is the node, the rest is inside its applet.
    pub fn start_pluck(&mut self, path: Option<&WorldTreePath>) {
        let path = path.cloned().unwrap_or_else(|| self.tree.get_focus_path());
        let id = match path.0.first() {
            None => Some(self.tree.root()),
            Some(node_path) => self.tree.node_at(node_path),
        };
        let Some(id) = id else {
            self.send(StandardRequest::Plucked(None));
            return;
        };

        // the applet may have something closer; it answers with `Plucked`.
        // Without a running applet (eg a split) it would never answer.
        if path.0.len() > 1
            && self
                .tree
                .try_send_event(id, StandardEvent::Pluck(Some(inner_path(&path))))
                .is_ok()
        {
            self.pending = Some(Pending::Pluck(id));
        } else {
            self.to_pluck = Some(id);
        }
    }

    /// Run after each batch of events
    pub fn finish(&mut self) {
        if let Some(id) = self.to_pluck.take() {
            // `None` for the root
            let plucked = self.tree.pluck(id);
            self.send(StandardRequest::Plucked(plucked));
        }
        if let Some((id, wrapper)) = self.to_wrap.take() {
            self.tree.wrap(id, wrapper);
        }
    }

    /// A UI event for the tree; a standard keybind on the selected node is done as if its applet asked,
    /// except that its applet is asked to close rather than closed (eg a wl client may refuse).
    /// Returns what the holder's own node should do instead, like `handle_node_request`.
    pub fn handle_ui_event(&mut self, event: StandardEvent) -> Option<NodeRequest> {
        self.tree.handle_event(event);
        let (id, request) = self.tree.take_keybind_request()?;
        if matches!(request, NodeRequest::Close) && self.tree.has_main(id) {
            self.tree.send_event(id, StandardEvent::CloseRequest);
            return None;
        }
        self.handle_node_request(id, request)
    }

    /// Places at the deepest node along `path[0]` (`None`: at the focus),
    /// asking its applet first if the path goes into it
    pub fn start_place(&mut self, path: Option<&WorldTreePath>, mut tree: DetachedTree) {
        let path = path.cloned().unwrap_or_else(|| self.tree.get_focus_path());
        let (id, exact) = path
            .0
            .first()
            .map_or((self.tree.root(), true), |node_path| {
                self.tree.closest_node_at(node_path)
            });

        if exact && path.0.len() > 1 {
            match self
                .tree
                .try_send_event(id, StandardEvent::Place(Some(inner_path(&path)), tree))
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
        self.send(StandardRequest::Placed(None));
    }

    /// Into the focused node's main if focus is inside it (it may take it or give it back with `AddWlSurface`),
    /// otherwise under the focused node
    pub fn add_wl_surface(
        &mut self,
        surface_id: WlSurfaceId,
        wl_event_queue: Sender<(WlSurfaceId, WlSurfaceEvent)>,
    ) {
        let id = self.tree.focused_node();
        if self.tree.is_inner(id) {
            self.tree.send_event(
                id,
                StandardEvent::WlSurfaceRegistered {
                    surface_id,
                    wl_event_queue,
                },
            );
        } else {
            self.tree.add_wl_surface(id, surface_id, wl_event_queue);
        }
    }

    /// Closes the surface's tile, here or in whichever main has it inside
    pub fn close_wl_surface(&mut self, surface_id: WlSurfaceId) {
        if !self.tree.close_wl_surface(surface_id) {
            for id in self.tree.with_main() {
                self.tree
                    .send_event(id, StandardEvent::WlSurfaceDestroyed(surface_id));
            }
        }
    }

    /// Passes the title to the surface's tile, here or in whichever main has it inside
    pub fn set_wl_surface_title(&mut self, surface_id: WlSurfaceId, title: String) {
        if !self.tree.set_wl_surface_title(surface_id, title.clone()) {
            for id in self.tree.with_main() {
                self.tree
                    .send_event(id, StandardEvent::WlSurfaceTitle(surface_id, title.clone()));
            }
        }
    }

    /// Tells the holder's node if focus moved onto or off a wl surface. Run after each batch of events.
    pub fn update_wl_focus(&mut self) {
        let focused_wl_surface = self.tree.focused_wl_surface();
        if focused_wl_surface != self.focused_wl_surface {
            self.focused_wl_surface = focused_wl_surface;
            self.send(StandardRequest::WlKeyboardFocus(focused_wl_surface));
        }
    }

    /// From the node's main. Returns what the holder's own node should do instead.
    pub fn handle_node_request(&mut self, id: NodeId, request: NodeRequest) -> Option<NodeRequest> {
        match request {
            NodeRequest::Standard(StandardRequest::Damage(_)) => self.tree.damage(),
            // eg Ctrl+Shift+Q; quits everything
            NodeRequest::Standard(StandardRequest::Quit) => self.send(StandardRequest::Quit),
            // the main is a holder too
            NodeRequest::Standard(StandardRequest::WlKeyboardFocus(surface_id)) => {
                self.tree.set_inner_wl_focus(id, surface_id);
            }
            NodeRequest::Standard(StandardRequest::Plucked(plucked)) => {
                if self.pending != Some(Pending::Pluck(id)) {
                    log::warn!("Unexpected `Plucked` from {id:?}; dropping it");
                    return None;
                }
                self.pending = None;
                match plucked {
                    Some(plucked) => self.send(StandardRequest::Plucked(Some(plucked))),
                    // nothing inside, so the node itself
                    None => self.to_pluck = Some(id),
                }
            }
            NodeRequest::Standard(StandardRequest::Placed(given_back)) => {
                if self.pending != Some(Pending::Place(id)) {
                    log::warn!("Unexpected `Placed` from {id:?}; dropping it");
                    return None;
                }
                self.pending = None;
                // the applet couldn't, so it becomes the node's child
                if let Some(tree) = given_back {
                    self.tree.place(id, tree);
                }
                self.send(StandardRequest::Placed(None));
            }
            // the applet's path is inside its inner tree
            NodeRequest::Standard(StandardRequest::RequestPluck(path)) => {
                if let Some(path) = self.full_path(id, path) {
                    self.send(StandardRequest::RequestPluck(path));
                }
            }
            NodeRequest::Standard(StandardRequest::RequestPlace(path)) => {
                if let Some(path) = self.full_path(id, path) {
                    self.send(StandardRequest::RequestPlace(path));
                }
            }
            NodeRequest::AddChild(initializer) => {
                self.tree.add_child(id, initializer);
            }
            NodeRequest::ChangeFocus(operation) => self.tree.change_focus(id, operation),
            // the root of a project's inner tree stands for the project, which closes into its contents
            NodeRequest::Close if self.project && id == self.tree.root() => self.dissolve = true,
            NodeRequest::Close => {
                self.tree.close(id);
            }
            // the project's node splits, likewise
            request @ NodeRequest::Split { .. } if self.project && id == self.tree.root() => {
                return Some(request);
            }
            NodeRequest::Split { axis, new_pane } => self.tree.split(id, axis, new_pane),
            // it is a project already
            NodeRequest::Wrap(_) if self.project && id == self.tree.root() => {}
            NodeRequest::Wrap(wrapper) => self.to_wrap = Some((id, wrapper)),
            NodeRequest::Unwrap(tree) => self.tree.unwrap(id, tree),
            NodeRequest::AddWlSurface {
                surface_id,
                wl_event_queue,
            } => self.tree.add_wl_surface(id, surface_id, wl_event_queue),
        }
        None
    }
}

/// The part of the path inside the node's applet
fn inner_path(path: &WorldTreePath) -> WorldTreePath {
    WorldTreePath(path.0.get(1..).unwrap_or_default().into())
}
