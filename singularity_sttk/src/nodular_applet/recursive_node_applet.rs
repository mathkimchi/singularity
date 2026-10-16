use crate::{
    nodular_applet::{NodeRequest, NodularAppletInitializer},
    standard_keybinds::standard_keybind_request,
};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        packets::{StandardEvent, WlSurfaceEvent, WlSurfaceId},
    },
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TreeNodePath, TreeTraverseOperation},
        world_tree::{WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation},
    },
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{UIElement, UITree},
    ui_event::{KeyTrait, MouseEvent, MouseEventKind, UIEvent},
};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy)]
enum FocusIndex {
    Focusing,
    Inner,
    Child(usize),
}

/// Identifies a node, so requests from its applet still find it after it moves
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(u64);
impl NodeId {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// What a node shows and forwards events to
pub enum NodeMain {
    /// Dropping the handle ends the applet
    Applet(ClientHandle),
    Wayland(WlSurfaceId),
}

/// What nodes need from whoever holds the tree
pub struct NodeEnv {
    /// Runs an applet as a client whose requests are tagged with the given node
    pub spawn_applet: Box<dyn Fn(NodeId, NodularAppletInitializer) -> ClientHandle>,
    /// Set once the first wl surface is registered
    pub wl_event_queue: Option<calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>>,
    /// Requests nodes make on behalf of their main (eg standard keybinds on wl tiles),
    /// for the holder to handle like applet requests
    pub pending_requests: Vec<(NodeId, NodeRequest)>,
    /// Set when the tree changed in a way that needs a redraw
    pub damaged: bool,
}

/// Holds a main applet (or wl surface) and also children.
/// Displays the focused one.
///
/// Only touched by the thread holding the tree; applets talk to it through `NodeRequest`s.
pub struct RecursiveNode {
    id: NodeId,
    main: NodeMain,
    children: Vec<Self>,
    focus: FocusIndex,
    /// So newly added children can be given the size
    latest_size: Option<DisplayContainerSize>,
}
impl RecursiveNode {
    fn new(id: NodeId, main: NodeMain) -> Self {
        Self {
            id,
            main,
            children: Vec::new(),
            focus: FocusIndex::Focusing,
            latest_size: None,
        }
    }

    pub fn new_applet(initializer: NodularAppletInitializer, env: &NodeEnv) -> Self {
        let id = NodeId::new();
        Self::new(id, NodeMain::Applet((env.spawn_applet)(id, initializer)))
    }

    pub fn new_wayland(surface_id: WlSurfaceId) -> Self {
        Self::new(NodeId::new(), NodeMain::Wayland(surface_id))
    }

    pub fn add_child(&mut self, mut child: Self, env: &mut NodeEnv) {
        if let Some(size) = self.latest_size {
            child.handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(size)), env);
        }
        self.children.push(child);
        env.damaged = true;
    }

    pub fn find_mut(&mut self, id: NodeId) -> Option<&mut Self> {
        let path = self.path_to(id)?;
        Some(self.node_at_mut(&path))
    }

    /// Child indices from this node down to the node with `id`
    fn path_to(&self, id: NodeId) -> Option<Vec<usize>> {
        if self.id == id {
            return Some(Vec::new());
        }
        self.children
            .iter()
            .enumerate()
            .find_map(|(child_index, child)| {
                let mut path = child.path_to(id)?;
                path.insert(0, child_index);
                Some(path)
            })
    }

    fn node_at_mut(&mut self, path: &[usize]) -> &mut Self {
        path.iter()
            .fold(self, |node, &child_index| &mut node.children[child_index])
    }

    /// Closes the node with `id`; its parent adopts its children in its place.
    /// If the focused child closed, the parent becomes focused.
    ///
    /// Returns whether something closed.
    pub fn close(&mut self, id: NodeId) -> bool {
        let Some(mut path) = self.path_to(id) else {
            return false;
        };
        let Some(index) = path.pop() else {
            log::warn!("Tried to close root");
            return false;
        };

        let parent = self.node_at_mut(&path);
        let closed = parent.children.remove(index);
        let num_orphans = closed.children.len();
        parent.children.splice(index..index, closed.children);
        parent.focus = match parent.focus {
            FocusIndex::Child(focused) if focused == index => FocusIndex::Focusing,
            FocusIndex::Child(focused) if focused > index => {
                FocusIndex::Child(focused + num_orphans - 1)
            }
            focus => focus,
        };
        true
    }

    /// Doesn't actually change anything.
    /// Just returns the new focus and what to call for the parent.
    fn calculate_new_focus(
        &self,
        operation: WorldTreeTraversalOperation,
    ) -> (Option<FocusIndex>, Option<WorldTreeTraversalOperation>) {
        match self.focus {
            FocusIndex::Focusing => match operation {
                WorldTreeTraversalOperation::GlobalRoot
                | WorldTreeTraversalOperation::PrevLayer
                | WorldTreeTraversalOperation::Layerwise(
                    TreeTraverseOperation::Parent | TreeTraverseOperation::RelShiftSibling(_),
                ) => {
                    // With PrevLayer, propagate the prev layer until the lowest level of prev layer gets this message
                    (None, Some(operation))
                }
                WorldTreeTraversalOperation::NextLayer => (Some(FocusIndex::Inner), None),
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::BfsPrev) => todo!(),
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::BfsNext) => todo!(),
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::Child(
                    child_index,
                )) => {
                    if child_index < self.children.len() {
                        (Some(FocusIndex::Child(child_index)), None)
                    } else {
                        (None, None)
                    }
                }
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::LastChild) => {
                    let num_children = self.children.len();
                    if num_children > 0 {
                        (Some(FocusIndex::Child(num_children - 1)), None)
                    } else {
                        (None, None)
                    }
                }
            },
            FocusIndex::Inner => match operation {
                WorldTreeTraversalOperation::GlobalRoot => todo!(),
                WorldTreeTraversalOperation::PrevLayer => (Some(FocusIndex::Focusing), None),
                WorldTreeTraversalOperation::NextLayer => todo!(),
                WorldTreeTraversalOperation::Layerwise(tree_traverse_operation) => {
                    match tree_traverse_operation {
                        TreeTraverseOperation::Parent => todo!(),
                        TreeTraverseOperation::RelShiftSibling(_) => todo!(),
                        TreeTraverseOperation::BfsPrev => todo!(),
                        TreeTraverseOperation::BfsNext => todo!(),
                        TreeTraverseOperation::Child(_) => todo!(),
                        TreeTraverseOperation::LastChild => todo!(),
                    }
                }
            },
            FocusIndex::Child(child_index) => match operation {
                WorldTreeTraversalOperation::GlobalRoot
                | WorldTreeTraversalOperation::PrevLayer => {
                    (Some(FocusIndex::Focusing), Some(operation))
                }
                // WorldTreeTraversalOperation::PrevLayer => {
                //     // If I don't change this guy's focus,
                //     // then it will automatically focus to
                //     // the previous spot when it comes here next time
                //     // self.focus_index.set(FocusIndex::Focusing);
                //     (None, Some(operation))
                // }
                WorldTreeTraversalOperation::NextLayer => todo!(),
                WorldTreeTraversalOperation::Layerwise(tree_traverse_operation) => {
                    match tree_traverse_operation {
                        TreeTraverseOperation::Parent => (
                            // the child wants to focus on parent, so just focus on self
                            Some(FocusIndex::Focusing),
                            None,
                        ),
                        TreeTraverseOperation::RelShiftSibling(shift) => (
                            Some(FocusIndex::Child(
                                (child_index as isize + shift)
                                    .clamp(0, self.children.len() as isize - 1)
                                    .cast_unsigned(),
                            )),
                            None,
                        ),
                        TreeTraverseOperation::BfsPrev => todo!(),
                        TreeTraverseOperation::BfsNext => todo!(),
                        TreeTraverseOperation::Child(_) => todo!(),
                        TreeTraverseOperation::LastChild => todo!(),
                    }
                }
            },
        }
    }

    /// Changes focus from the perspective of the node with `id`,
    /// passing the operation on to its ancestors as needed.
    ///
    /// Returns whether `id` is in this tree.
    pub fn change_focus(&mut self, id: NodeId, operation: WorldTreeTraversalOperation) -> bool {
        let Some(mut path) = self.path_to(id) else {
            return false;
        };

        let mut operation = Some(operation);
        while let Some(current_operation) = operation {
            let node = self.node_at_mut(&path);
            let (new_focus, parent_call) = node.calculate_new_focus(current_operation);
            if let Some(new_focus) = new_focus {
                node.focus = new_focus;
            }
            operation = parent_call;

            // the root has no parent to pass it to
            if path.pop().is_none() {
                break;
            }
        }
        true
    }

    fn send_to_main(&self, event: StandardEvent, env: &mut NodeEnv) {
        match &self.main {
            NodeMain::Applet(client) => client.send_event(event),
            NodeMain::Wayland(surface_id) => {
                let StandardEvent::UIEvent(ui_event) = event else {
                    return;
                };

                // wl clients can't send node requests, so their node does the standard keybinds
                let wl_event = match standard_keybind_request(&ui_event) {
                    // the tile closes once the client destroys its toplevel (`WlSurfaceDestroyed`),
                    // so the client may refuse (eg to ask about unsaved changes)
                    Some(NodeRequest::Close) => WlSurfaceEvent::Close,
                    Some(request) => {
                        env.pending_requests.push((self.id, request));
                        return;
                    }
                    None => Self::to_wl_event(ui_event),
                };
                if let Some(wl_event_queue) = &env.wl_event_queue {
                    wl_event_queue.send((*surface_id, wl_event)).unwrap();
                }
            }
        }
    }

    fn to_wl_event(ui_event: UIEvent) -> WlSurfaceEvent {
        match ui_event {
            UIEvent::Key {
                raw_keycode,
                pressed,
                ..
            } => WlSurfaceEvent::Key {
                // evdev -> xkb
                keycode: raw_keycode + 8,
                pressed,
            },
            UIEvent::Mouse(mouse_event) => WlSurfaceEvent::Mouse(mouse_event),
            UIEvent::WindowResized(new_size) => WlSurfaceEvent::Resize(new_size),
        }
    }

    pub fn handle_event(&mut self, event: StandardEvent, env: &mut NodeEnv) {
        match event {
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                // every applet in this node is displayed at this size, not just the focused one
                self.latest_size = Some(size);
                self.send_to_main(event, env);
                for child in &mut self.children {
                    child.handle_event(StandardEvent::UIEvent(UIEvent::WindowResized(size)), env);
                }
            }
            StandardEvent::UIEvent(ui_event) => match self.focus {
                FocusIndex::Focusing => {
                    // We can intercept
                    match &ui_event {
                        UIEvent::Key {
                            symbol,
                            pressed: true,
                            ..
                        } => {
                            if let Some(key) = symbol
                                && let Some(key_char) = key.to_char()
                                && let Some(operation) =
                                    WorldTreeTraversalOperation::from_char(key_char)
                            {
                                // may need the ancestors, so the tree's holder does it
                                env.pending_requests
                                    .push((self.id, NodeRequest::ChangeFocus(operation)));
                            } else {
                                // if key press not a traversal, then just set focus to inner and forward input
                                self.focus = FocusIndex::Inner;
                                env.damaged = true;
                                self.send_to_main(event, env);
                            }
                        }
                        UIEvent::Key { pressed: false, .. } => {}
                        UIEvent::Mouse(MouseEvent {
                            kind: MouseEventKind::Button { pressed: true, .. },
                            ..
                        }) => {
                            // set focus to inner and forward mouse press
                            self.focus = FocusIndex::Inner;
                            env.damaged = true;
                            self.send_to_main(event, env);
                        }
                        UIEvent::WindowResized(_) | UIEvent::Mouse(_) => {
                            // forward without changing focus (otherwise hovering would steal focus)
                            self.send_to_main(event, env);
                        }
                    }
                }
                FocusIndex::Inner => self.send_to_main(event, env),
                FocusIndex::Child(child_index) => {
                    self.children[child_index].handle_event(event, env);
                }
            },
            StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..)
            | StandardEvent::CloseRequest => match self.focus {
                // REVIEW: should focusing just auto go to inner?
                FocusIndex::Focusing | FocusIndex::Inner => self.send_to_main(event, env),
                FocusIndex::Child(child_index) => {
                    self.children[child_index].handle_event(event, env);
                }
            },
            // the tree's holder handles these
            StandardEvent::DamageAck(_)
            | StandardEvent::WlSurfaceRegistered { .. }
            | StandardEvent::FocusWlSurface(_)
            | StandardEvent::WlSurfaceDestroyed(_) => {}
        }
    }

    /// Focuses down to the node showing the surface.
    ///
    /// Returns whether it's in this tree.
    pub fn focus_wl_surface(&mut self, surface_id: WlSurfaceId) -> bool {
        let new_focus = if matches!(self.main, NodeMain::Wayland(id) if id == surface_id) {
            FocusIndex::Inner
        } else if let Some(child_index) = self
            .children
            .iter_mut()
            .position(|child| child.focus_wl_surface(surface_id))
        {
            FocusIndex::Child(child_index)
        } else {
            return false;
        };
        self.focus = new_focus;
        true
    }

    /// The node showing the surface
    pub fn find_wl_surface(&self, surface_id: WlSurfaceId) -> Option<NodeId> {
        if matches!(self.main, NodeMain::Wayland(id) if id == surface_id) {
            return Some(self.id);
        }
        self.children
            .iter()
            .find_map(|child| child.find_wl_surface(surface_id))
    }

    /// `None` while this node itself is focused (`Focusing`), since it takes the keys then
    pub fn focused_wl_surface(&self) -> Option<WlSurfaceId> {
        match self.focus {
            FocusIndex::Focusing => None,
            FocusIndex::Inner => match self.main {
                NodeMain::Wayland(surface_id) => Some(surface_id),
                // TODO: wl surfaces inside an applet's inner tree
                NodeMain::Applet(_) => None,
            },
            FocusIndex::Child(child_index) => self.children[child_index].focused_wl_surface(),
        }
    }

    /// The focused applet's window
    pub fn get_window(&self) -> UITree {
        match self.focus {
            FocusIndex::Focusing | FocusIndex::Inner => match &self.main {
                NodeMain::Applet(client) => client.get_surface(),
                NodeMain::Wayland(surface_id) => {
                    UITree::leaf(UIElement::Subsurface(surface_id.as_u64()))
                }
            },
            FocusIndex::Child(child_index) => self.children[child_index].get_window(),
        }
    }

    pub fn get_treeview(&self) -> WorldTree<String> {
        let main_treeview = match &self.main {
            NodeMain::Applet(client) => client.get_treeview(),
            // TODO: proper wl title
            NodeMain::Wayland(_) => WorldTree::Base("Wl Surface".to_string()),
        };
        let mut raw_treeview = RecursiveTreeNode::from_value(main_treeview);

        for child in &self.children {
            match child.get_treeview() {
                WorldTree::World(recursive_tree_node) => {
                    raw_treeview.push_child_node(*recursive_tree_node);
                }
                child_treeview => {
                    // REVIEW: should this ever even happen?
                    raw_treeview.push_child_node(RecursiveTreeNode::from_value(child_treeview));
                }
            }
        }

        WorldTree::World(Box::new(raw_treeview))
    }

    pub fn get_focus_path(&self) -> WorldTreePath {
        match self.focus {
            FocusIndex::Focusing => WorldTreePath::new_into(),
            FocusIndex::Inner => {
                // TODO: the focus path inside the main applet's inner tree; for now its root
                let mut path_tail = WorldTreePath::new_into().0.to_vec();

                path_tail.insert(0, TreeNodePath::new_root());

                WorldTreePath(path_tail.into_boxed_slice())
            }
            FocusIndex::Child(child_index) => {
                let mut path_tail = self.children[child_index].get_focus_path().0.to_vec();

                if path_tail.is_empty() {
                    log::warn!("Warning: this shouldn't happen");
                    path_tail.push(TreeNodePath::new_root());
                }

                // child of index `focus - 1` is focused
                path_tail[0].0.insert(0, child_index);

                WorldTreePath(path_tail.into_boxed_slice())
            }
        }
    }
}

/*
// /// Has a list of inner applets and displays them in vertical or horizontal division.
// pub struct DividedApplet {
//     shared_resource: Arc<MultiAppletHolder>,
// }
// impl DividedApplet {
//     fn new(
//         inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
//         // hook: Box<dyn NodularRunnerHook>,
//         hook: Box<dyn NodularRunnerHook>,
//     ) -> Self {
//         let s = Self {
//             shared_resource: Arc::new(MultiAppletHolder::new(hook)),
//         };

//         MultiAppletHolder::add_child(Arc::downgrade(&s.shared_resource), inner_initiator);

//         s
//     }

//     /// Partial application
//     pub fn get_initializer(
//         inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
//     ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
//         move |hook: Box<dyn NodularRunnerHook>| Self::new(inner_initializer, hook)
//     }

//     /// Takes in a list of full-size elements and returns a combined ui element where they are equally spaced
//     /// across the horizontal axis and take full height.
//     fn combine_displays(subdisplays: Vec<UIElement>) -> UIElement {
//         // proportional units so widths out of 1
//         let widths = 1. / subdisplays.len() as f32;
//         UIElement::Container(
//             subdisplays
//                 .into_iter()
//                 .enumerate()
//                 .map(|(i, subdisplay)| {
//                     subdisplay
//                         .bordered(Color::LIGHT_GREEN)
//                         .contain(DisplayArea::new(
//                             (widths * (i as f32), 0.),
//                             (DisplayUnits::from_mixed(-1, widths * ((i + 1) as f32)), 1.),
//                         ))
//                 })
//                 .collect(),
//         )
//     }

//     fn get_display(&self) -> UIElement {
//         let mut applet_displays = Vec::new();

//         for applet in self.shared_resource.applets.read().unwrap().iter() {
//             applet_displays.push(applet.get_window());
//         }

//         Self::combine_displays(applet_displays)
//     }
// }
// impl BasicApplet for DividedApplet {
//     fn handle_ui_event(&mut self, ui_event: UIEvent) {
//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: true,
//                 alt: false,
//                 shift: false,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.to_char() == Some('\t')
//         {
//             self.shared_resource.applets.read().unwrap()[self
//                 .shared_resource
//                 .focus_index
//                 .load(std::sync::atomic::Ordering::Relaxed)]
//             .immut_handle_nodular_event(NodularEvent::Focused(false));

//             self.shared_resource
//                 .focus_index
//                 .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//             self.shared_resource.focus_index.fetch_min(
//                 self.shared_resource.applets.read().unwrap().len() - 1,
//                 std::sync::atomic::Ordering::Relaxed,
//             );

//             self.shared_resource.applets.read().unwrap()[self
//                 .shared_resource
//                 .focus_index
//                 .load(std::sync::atomic::Ordering::Relaxed)]
//             .immut_handle_nodular_event(NodularEvent::Focused(true));

//             return;
//         }

//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: true,
//                 alt: false,
//                 shift: true,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.raw_code == 15
//         {
//             // Ctrl+Shift+Tab
//             self.shared_resource.applets.read().unwrap()[self
//                 .shared_resource
//                 .focus_index
//                 .load(std::sync::atomic::Ordering::Relaxed)]
//             .immut_handle_nodular_event(NodularEvent::Focused(false));

//             self.shared_resource
//                 .focus_index
//                 .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
//             self.shared_resource.focus_index.fetch_min(
//                 self.shared_resource.applets.read().unwrap().len() - 1,
//                 std::sync::atomic::Ordering::Relaxed,
//             );

//             self.shared_resource.applets.read().unwrap()[self
//                 .shared_resource
//                 .focus_index
//                 .load(std::sync::atomic::Ordering::Relaxed)]
//             .immut_handle_nodular_event(NodularEvent::Focused(true));
//             return;
//         }

//         let applet_holder = self.shared_resource.applets.read().unwrap()[self
//             .shared_resource
//             .focus_index
//             .load(std::sync::atomic::Ordering::Relaxed)]
//         .clone();
//         applet_holder.immut_handle_ui_event(ui_event);
//     }

//     fn get_window(&self) -> UIElement {
//         // TODO: return cached if damaged is already false?
//         self.shared_resource
//             .window_damaged
//             .store(false, std::sync::atomic::Ordering::Relaxed);

//         self.get_display()
//     }
// }
// impl NodularApplet for DividedApplet {
//     fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
//         match nodular_event {
//             NodularEvent::Highlighted(_) => todo!(),
//             NodularEvent::Focused(state) => {
//                 self.shared_resource.applets.read().unwrap()[self
//                     .shared_resource
//                     .focus_index
//                     .load(std::sync::atomic::Ordering::Relaxed)]
//                 .immut_handle_nodular_event(NodularEvent::Focused(state));
//             }
//         }
//     }

//     fn get_treeview(&self) -> RootedTree<String> {
//         self.shared_resource
//             .treeview_damaged
//             .store(false, std::sync::atomic::Ordering::Relaxed);

//         // RootedTree::
//         todo!()
//     }
// }

*/

// /// Ideally, this would've been a function,
// /// but I couldn't get the types working.
// macro_rules! get_focused_applet {
//     ($s:expr, $f:literal) => {
//         match $s.shared_resource.focus_index.get() {
//             FocusIndex::Focusing | FocusIndex::Inner => ($f)($s.main_applet),
//             FocusIndex::Child(child_index) => {
//                 ($f)($s.shared_resource.children.read().unwrap()[child_index])
//             }
//         }
//     };
// }
