//! This is where the hierarchy stuff is implemented.

use calloop::{EventLoop, LoopHandle, LoopSignal, channel::Sender};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        detached::{Axis, DetachedClient, DetachedTree},
        packets::{StandardEvent, StandardRequest, StateKind},
        raw_client_initializer::RawClientInitializer,
        shared_state::{SharedState, StatePublisher},
    },
    utils::tree::world_tree::{
        WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation,
    },
};
use sonamu_ui::{display_units::DisplayContainerSize, ui_element::UITree, ui_event::UIEvent};
use std::{
    cell::Cell,
    collections::BTreeMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

pub mod node_tree;
pub mod root_node_applet;
mod treeview;

// pub struct RootApplet<Applet: BasicApplet> {
//     inner: Applet,
//     children: NodeApplet<Box<dyn BasicApplet>>,
// }
// impl<Applet: BasicApplet> BasicApplet for RootApplet<Applet> {
//     type InitializingData = Applet::InitializingData;

//     fn initialize(
//         initializing_data: Self::InitializingData,
//         hook: Box<dyn singularity_sar::applet::RunnerHook>,
//     ) -> Self {
//         todo!()
//     }

//     fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
//         todo!()
//     }
// }

/// An applet; runs on its own thread as a client of the node holding it.
pub trait StandardApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent);

    /// Called when the surface is damaged (`AppletContext::damage_window`) or resized
    fn get_window_standard_applet(&self, container_size: DisplayContainerSize) -> UITree;

    /// The applet's inner tree; its node shows it as is.
    ///
    /// TODO: also publish the focus path inside it, and let focus cross into it.
    /// For now the node assumes the inner tree's root is focused.
    fn get_treeview(&self) -> WorldTree<String>;

    /// Takes the closest pluckable element at `path` (in the inner tree) out of the applet.
    /// `None` makes its node pluck itself, with the applet and its children.
    fn pluck(&mut self, _path: &WorldTreePath) -> Option<DetachedTree> {
        None
    }

    /// Takes ownership of `tree` at `path` (in the inner tree).
    /// `Err` gives it back, and the node adds it as its last child.
    fn place(&mut self, _path: &WorldTreePath, tree: DetachedTree) -> Result<(), DetachedTree> {
        Err(tree)
    }

    /// A request from an applet this applet embedded (`AppletContext::embed`)
    fn handle_embedded_request(&mut self, _id: EmbeddedId, _request: NodeRequest) {}

    /// Called on the applet's thread after an `AppletWaker` woke it (eg a worker thread finished)
    fn wake(&mut self) {}
}

/// Wakes an applet from another thread (`StandardApplet::wake`); from `AppletContext::waker`
#[derive(Clone)]
pub struct AppletWaker(Sender<()>);
impl AppletWaker {
    pub fn wake(&self) {
        // the applet may have ended
        let _ = self.0.send(());
    }
}

/// An applet embedded in another applet's inner tree (`AppletContext::embed`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmbeddedId(u64);

/// Makes the applet, on the applet's own thread
pub type NodularAppletInitializer =
    Box<dyn FnOnce(AppletContext) -> Box<dyn StandardApplet> + Send>;

pub trait AppletSpawnerTrait: Send + Sync {
    fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer>;

    /// Clone on its own is not dyn compatible because it outputs -> Self.
    /// Using the AppletSpawnerTrait, AppletSpawner workaround for that.
    fn duplicate(&self) -> AppletSpawner;
}
pub type AppletSpawner = Box<dyn AppletSpawnerTrait>;

impl Clone for AppletSpawner {
    fn clone(&self) -> Self {
        self.duplicate()
    }
}

pub type AppletSpawnerRegistry = Arc<RwLock<BTreeMap<String, AppletSpawner>>>;

/// Applet to its node
pub enum NodeRequest {
    Standard(StandardRequest),
    /// The child gets its own node under the applet's node
    AddChild(NodularAppletInitializer),
    /// From the perspective of the applet's node
    ChangeFocus(WorldTreeTraversalOperation),
    /// Closes the applet's node; its children move to the parent
    Close,
    /// Replaces the applet's node with a split showing it and a new applet next to it
    Split {
        axis: Axis,
        new_pane: NodularAppletInitializer,
    },
    /// Makes the applet's node a project (its subtree is viewed on its own while focus is in it), or not
    SetProject(bool),
}
impl From<StandardRequest> for NodeRequest {
    fn from(request: StandardRequest) -> Self {
        Self::Standard(request)
    }
}

/// What an applet uses to talk to its node (replaces the old runner hooks).
///
/// Can be cloned, but stays on the applet's thread (it holds the applet's loop, for `embed`).
#[derive(Clone)]
pub struct AppletContext {
    request_sender: Sender<NodeRequest>,
    /// To the applet's own loop, which publishes the damaged state
    damage_sender: Sender<StateKind>,
    /// To the applet's own loop, which calls `StandardApplet::wake`
    wake_sender: Sender<()>,
    applet_spawners: AppletSpawnerRegistry,
    /// The applet's own loop, which embedded applets' requests are handled on
    event_loop: LoopHandle<'static, AppletState>,
    next_embedded_id: Rc<Cell<u64>>,
}
impl AppletContext {
    /// Runs an applet that this applet holds itself, in its inner tree.
    /// Its requests go to `StandardApplet::handle_embedded_request`; dropping the handle ends it.
    pub fn embed(&self, initializer: NodularAppletInitializer) -> (EmbeddedId, ClientHandle) {
        let id = self.new_embedded_id();
        let handle = ClientHandle::spawn_new_client(
            Box::new(AppletClient {
                initializer,
                applet_spawners: self.applet_spawners.clone(),
            }),
            &self.event_loop,
            move |request, state: &mut AppletState| {
                state.applet.handle_embedded_request(id, request);
            },
        );
        (id, handle)
    }

    /// Like `embed`, for a placed client that is already running
    pub fn embed_detached(&self, client: DetachedClient) -> (EmbeddedId, ClientHandle) {
        let id = self.new_embedded_id();
        let handle = client.attach(&self.event_loop, move |request, state: &mut AppletState| {
            state.applet.handle_embedded_request(id, request);
        });
        (id, handle)
    }

    fn new_embedded_id(&self) -> EmbeddedId {
        let id = self.next_embedded_id.get();
        self.next_embedded_id.set(id + 1);
        EmbeddedId(id)
    }

    pub fn request(&self, request: NodeRequest) {
        // the node may already be gone while closing
        let _ = self.request_sender.send(request);
    }

    pub fn damage_window(&self) {
        let _ = self.damage_sender.send(StateKind::Surface);
    }

    pub fn damage_treeview(&self) {
        let _ = self.damage_sender.send(StateKind::Treeview);
    }

    /// For other threads, since the context stays on the applet's thread
    pub fn waker(&self) -> AppletWaker {
        AppletWaker(self.wake_sender.clone())
    }

    pub fn close(&self) {
        self.request(NodeRequest::Close);
    }

    pub fn add_child(&self, initializer: NodularAppletInitializer) {
        self.request(NodeRequest::AddChild(initializer));
    }

    pub fn change_focus(&self, operation: WorldTreeTraversalOperation) {
        self.request(NodeRequest::ChangeFocus(operation));
    }

    /// Quits everything
    pub fn quit(&self) {
        self.request(StandardRequest::Quit.into());
    }

    /// NOTE: look at 2026-03-08 for more detail on future things
    pub fn register_applet_spawner(&self, name: String, applet_spawner: AppletSpawner) {
        self.applet_spawners
            .write()
            .unwrap()
            .insert(name, applet_spawner);
    }
    pub fn get_applet_spawners(&self) -> BTreeMap<String, AppletSpawner> {
        self.applet_spawners.read().unwrap().clone()
    }
    pub fn find_applet_spawner(&self, name: &str) -> Option<AppletSpawner> {
        self.applet_spawners.read().unwrap().get(name).cloned()
    }
}

/// The applet's event loop data
pub struct AppletState {
    applet: Box<dyn StandardApplet>,
    size: DisplayContainerSize,
    surface: StatePublisher<UITree>,
    treeview: StatePublisher<WorldTree<String>>,
    request_sender: Sender<NodeRequest>,
    loop_signal: LoopSignal,
}

/// Runs a `StandardApplet` as a client of a node
pub(crate) struct AppletClient {
    pub initializer: NodularAppletInitializer,
    pub applet_spawners: AppletSpawnerRegistry,
}
impl RawClientInitializer<NodeRequest> for AppletClient {
    fn init(
        self: Box<Self>,
        surface: SharedState<UITree>,
        treeview: SharedState<WorldTree<String>>,
        event_queue: calloop::channel::Channel<StandardEvent>,
        request_sender: Sender<NodeRequest>,
    ) {
        let mut event_loop: EventLoop<AppletState> = EventLoop::try_new().unwrap();
        let (damage_tx, damage_rx) = calloop::channel::channel();

        event_loop
            .handle()
            .insert_source(event_queue, |event, &mut (), state| match event {
                calloop::channel::Event::Msg(StandardEvent::DamageAck(StateKind::Surface)) => {
                    state.surface.ack();
                }
                calloop::channel::Event::Msg(StandardEvent::DamageAck(StateKind::Treeview)) => {
                    state.treeview.ack();
                }
                // TODO: `None` should be the focus path inside the inner tree; for now its root
                calloop::channel::Event::Msg(StandardEvent::Pluck(path)) => {
                    let path = path.unwrap_or_else(WorldTreePath::new_into);
                    let plucked = state.applet.pluck(&path);
                    let _ = state
                        .request_sender
                        .send(StandardRequest::Plucked(plucked).into());
                }
                calloop::channel::Event::Msg(StandardEvent::Place(path, tree)) => {
                    let path = path.unwrap_or_else(WorldTreePath::new_into);
                    let given_back = state.applet.place(&path, tree).err();
                    let _ = state
                        .request_sender
                        .send(StandardRequest::Placed(given_back).into());
                }
                calloop::channel::Event::Msg(event) => {
                    if let StandardEvent::UIEvent(UIEvent::WindowResized(size)) = event {
                        state.size = size;
                        state.surface.damage();
                    }
                    state.applet.handle_standard_event(event);
                }
                // the node dropped this applet's `ClientHandle`
                calloop::channel::Event::Closed => state.loop_signal.stop(),
            })
            .unwrap();
        event_loop
            .handle()
            .insert_source(damage_rx, |event, &mut (), state| {
                if let calloop::channel::Event::Msg(kind) = event {
                    match kind {
                        StateKind::Surface => state.surface.damage(),
                        StateKind::Treeview => state.treeview.damage(),
                    }
                }
            })
            .unwrap();
        let (wake_tx, wake_rx) = calloop::channel::channel();
        event_loop
            .handle()
            .insert_source(wake_rx, |event, &mut (), state| {
                if let calloop::channel::Event::Msg(()) = event {
                    state.applet.wake();
                }
            })
            .unwrap();

        let context = AppletContext {
            request_sender: request_sender.clone(),
            damage_sender: damage_tx,
            wake_sender: wake_tx,
            applet_spawners: self.applet_spawners,
            event_loop: event_loop.handle(),
            next_embedded_id: Rc::default(),
        };
        let mut state = AppletState {
            applet: (self.initializer)(context),
            // until the node sends a size
            size: DisplayContainerSize::new(800, 600),
            surface: StatePublisher::new(surface),
            treeview: StatePublisher::new(treeview),
            request_sender,
            loop_signal: event_loop.get_signal(),
        };

        // after each batch of events, so multiple damages become one publish
        event_loop
            .run(None, &mut state, |state| {
                state.surface.flush(
                    || state.applet.get_window_standard_applet(state.size),
                    &state.request_sender,
                );
                state
                    .treeview
                    .flush(|| state.applet.get_treeview(), &state.request_sender);
            })
            .unwrap();
    }
}
