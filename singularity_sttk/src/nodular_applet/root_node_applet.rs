use crate::nodular_applet::{
    AppletClient, AppletSpawner, NodeRequest, NodularAppletInitializer,
    recursive_node_applet::{NodeEnv, NodeId, RecursiveNode},
};
use calloop::{
    EventLoop, LoopHandle,
    channel::{Channel, Sender},
};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        packets::{StandardEvent, StandardRequest, StateKind, WlSurfaceId},
        raw_client_initializer::RawClientInitializer,
        shared_state::{SharedState, StatePublisher},
    },
    utils::tree::world_tree::{WorldTree, WorldTreePath},
};
use sonamu_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
    ui_element::{TrackSize, UIElement, UITree},
    ui_event::UIEvent,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

/// Holds the tree of nodes, is held by a Basic Applet runner (applet runner).
///
/// Every applet in the tree is a client of this, on its own thread.
pub struct RootNodeApplet {
    tree: RecursiveNode,
    env: NodeEnv,
    surface: StatePublisher<UITree>,
    request_sender: Sender<StandardRequest>,

    latest_size: DisplayContainerSize,
    /// Last sent with `StandardRequest::WlKeyboardFocus`
    focused_wl_surface: Option<WlSurfaceId>,
}
impl RootNodeApplet {
    const TREEVIEW_SPLIT: f32 = 0.2;
    const INNER_APPLET_AREA: DisplayArea =
        DisplayArea::new_proportional([[Self::TREEVIEW_SPLIT, 0.0], [1.0, 1.0]]);

    fn inner_applet_size(&self) -> DisplayContainerSize {
        self.latest_size
            .find_subsize(Self::INNER_APPLET_AREA.size())
    }

    /// Tells the runner if Sonamu focus moved onto or off a wl surface
    fn update_wl_focus(&mut self) {
        let focused_wl_surface = self.tree.focused_wl_surface();
        if focused_wl_surface != self.focused_wl_surface {
            self.focused_wl_surface = focused_wl_surface;
            self.request_sender
                .send(StandardRequest::WlKeyboardFocus(focused_wl_surface))
                .unwrap();
        }
    }

    /// Publishes the window if damaged and the runner read the last one
    fn flush_surface(&mut self) {
        let tree = &self.tree;
        self.surface
            .flush(|| Self::get_window(tree), &self.request_sender);
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
        let event_loop = event_loop.clone();
        let env = NodeEnv {
            spawn_applet: Box::new(move |id, initializer| {
                ClientHandle::spawn_new_client(
                    Box::new(AppletClient {
                        initializer,
                        applet_spawners: applet_spawners.clone(),
                    }),
                    &event_loop,
                    move |request, root: &mut Self| root.handle_node_request(id, request),
                )
            }),
            wl_event_queue: None,
            pending_requests: Vec::new(),
            damaged: false,
        };

        Self {
            tree: RecursiveNode::new_applet(root_initializer, &env),
            env,
            surface: StatePublisher::new(surface),
            request_sender,
            // Just making stuff up
            latest_size: DisplayContainerSize::new(800, 600),
            focused_wl_surface: None,
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
                self.tree.handle_event(
                    StandardEvent::UIEvent(UIEvent::WindowResized(self.inner_applet_size())),
                    &mut self.env,
                );
            }
            StandardEvent::UIEvent(UIEvent::Mouse(mut mouse_event)) => {
                // remap window coords -> inner applet coords
                // TODO: there's also the 1px border on every side
                let [left, top] = Self::INNER_APPLET_AREA.map_onto_px_size(self.latest_size).0[0];
                mouse_event.position[0] -= f64::from(left);
                mouse_event.position[1] -= f64::from(top);
                self.tree.handle_event(
                    StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)),
                    &mut self.env,
                );
            }
            StandardEvent::DamageAck(StateKind::Surface) => self.surface.ack(),
            // the treeview is drawn into the surface
            StandardEvent::DamageAck(StateKind::Treeview) => {}
            StandardEvent::CloseRequest => todo!(),
            StandardEvent::WlSurfaceRegistered {
                surface_id,
                wl_event_queue,
            } => {
                self.env.wl_event_queue = Some(wl_event_queue);
                // TODO: change behavior based on the focus
                self.tree
                    .add_child(RecursiveNode::new_wayland(surface_id), &mut self.env);
            }
            StandardEvent::FocusWlSurface(surface_id) => {
                if self.tree.focus_wl_surface(surface_id) {
                    self.surface.damage();
                }
            }
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                // its tile closes like an applet closing itself
                if let Some(id) = self.tree.find_wl_surface(surface_id) {
                    self.env.pending_requests.push((id, NodeRequest::Close));
                }
            }
            StandardEvent::UIEvent(_)
            | StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..) => self.tree.handle_event(event, &mut self.env),
        }

        self.process_pending();
    }

    /// From an applet in the tree
    fn handle_node_request(&mut self, id: NodeId, request: NodeRequest) {
        self.apply_node_request(id, request);
        self.process_pending();
    }

    /// Handles requests nodes made themselves, then redraws if the tree changed
    fn process_pending(&mut self) {
        for (id, request) in std::mem::take(&mut self.env.pending_requests) {
            self.apply_node_request(id, request);
        }
        if std::mem::take(&mut self.env.damaged) {
            self.surface.damage();
        }
    }

    fn apply_node_request(&mut self, id: NodeId, request: NodeRequest) {
        match request {
            // the treeview is drawn into the surface, so both damage it
            NodeRequest::Standard(StandardRequest::Damage(_)) => self.surface.damage(),
            // eg Ctrl+Shift+Q; quits everything
            NodeRequest::Standard(StandardRequest::Quit) => {
                self.request_sender.send(StandardRequest::Quit).unwrap();
            }
            // only sent by the root itself
            NodeRequest::Standard(StandardRequest::WlKeyboardFocus(_)) => {}
            NodeRequest::AddChild(initializer) => {
                // the node may have closed since
                if let Some(node) = self.tree.find_mut(id) {
                    let child = RecursiveNode::new_applet(initializer, &self.env);
                    node.add_child(child, &mut self.env);
                }
            }
            NodeRequest::ChangeFocus(operation) => {
                if self.tree.change_focus(id, operation) {
                    self.surface.damage();
                }
            }
            NodeRequest::Close => {
                if self.tree.close(id) {
                    self.surface.damage();
                }
            }
        }
    }

    fn get_treeview_display(tree: &RecursiveNode) -> UITree {
        let focused_path = tree.get_focus_path();
        let treeview = tree.get_treeview();

        println!("Focused path: {focused_path:?}");

        // CharGrid::from(treeview.outer_world_to_string())
        //     .element()
        //     .bordered(Color::LIGHT_GREEN)
        //     .fill_bg(Color::BLACK)

        // This is the horizontal split
        // UIElement::combine_displays((0..(focused_path.0.len() + 1)).map(|world_level_index| {
        UITree::combine_displays(
            (0..focused_path.0.len())
                .map(|world_level_index| {
                    let world_path = WorldTreePath(
                        focused_path.0[0..world_level_index]
                            .to_vec()
                            .into_boxed_slice(),
                    );
                    UIElement::from(
                        treeview
                            .safe_get(&world_path)
                            .unwrap()
                            .outer_world_to_string(focused_path.0.get(world_level_index).cloned()),
                    )
                    .as_tree()
                    .bordered(Color::LIGHT_GREEN)
                })
                .collect(),
        )
        .fill_bg(Color::BLACK)
    }

    fn get_window(tree: &RecursiveNode) -> UITree {
        UITree::new_container_styled(
            sonamu_ui::ui_element::Style {
                display: sonamu_ui::ui_element::Display::Grid,
                grid_template_columns: vec![
                    TrackSize::Percent(Self::TREEVIEW_SPLIT),
                    TrackSize::Fr(1.0),
                ],
                grid_template_rows: vec![TrackSize::Fr(1.0)],
                ..sonamu_ui::ui_element::Style::full_size()
            },
            vec![
                Self::get_treeview_display(tree),
                tree.get_window().bordered(Color::LIGHT_GREEN),
            ],
        )
    }
}
