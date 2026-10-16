use crate::{
    creatable_applet::CreatableNodularApplet, nodular_applet::NodularAppletInitializer,
    wl_surface_applet::WlSurfaceApplet,
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
use slotmap::SlotMap;
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_event::{KeyTrait, MouseEvent, MouseEventKind, UIEvent},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum FocusIndex {
    Focusing,
    Inner,
    Child(usize),
}

slotmap::new_key_type! {
    /// Identifies a node, so requests from its applet still find it after it moves
    pub struct NodeId;
}

/// Shows its main applet, or the focused child.
struct Node {
    /// Dropping it ends the applet
    main: ClientHandle,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    focus: FocusIndex,
}
impl Node {
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
}

/// The outer tree: every node's main is an applet running as a client of the tree's holder.
///
/// Only touched by the holder's thread; applets talk to it through `NodeRequest`s.
pub struct NodeTree {
    root: NodeId,
    nodes: SlotMap<NodeId, Node>,
    /// Which node shows each wl surface (its main is a `WlSurfaceApplet`)
    wl_surfaces: HashMap<WlSurfaceId, NodeId>,

    /// Every node is displayed at this size
    latest_size: Option<DisplayContainerSize>,
    /// Runs an applet as a client whose requests are tagged with the given node
    spawn_applet: Box<dyn Fn(NodeId, NodularAppletInitializer) -> ClientHandle>,
    /// Set when the tree changed in a way that needs a redraw
    damaged: bool,
}
impl NodeTree {
    pub fn new(
        root_initializer: NodularAppletInitializer,
        spawn_applet: Box<dyn Fn(NodeId, NodularAppletInitializer) -> ClientHandle>,
    ) -> Self {
        let mut tree = Self {
            root: NodeId::default(),
            nodes: SlotMap::with_key(),
            wl_surfaces: HashMap::new(),
            latest_size: None,
            spawn_applet,
            damaged: true,
        };
        tree.root = tree.insert_node(None, root_initializer);
        tree
    }

    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Whether the tree changed since the last call
    pub const fn take_damage(&mut self) -> bool {
        std::mem::replace(&mut self.damaged, false)
    }

    fn insert_node(
        &mut self,
        parent: Option<NodeId>,
        initializer: NodularAppletInitializer,
    ) -> NodeId {
        let spawn_applet = &self.spawn_applet;
        let id = self.nodes.insert_with_key(|id| Node {
            main: spawn_applet(id, initializer),
            parent,
            children: Vec::new(),
            focus: FocusIndex::Focusing,
        });
        if let Some(size) = self.latest_size {
            self.nodes[id]
                .main
                .send_event(StandardEvent::UIEvent(UIEvent::WindowResized(size)));
        }
        id
    }

    /// Returns `None` if `parent` isn't in the tree (eg it closed since)
    pub fn add_child(
        &mut self,
        parent: NodeId,
        initializer: NodularAppletInitializer,
    ) -> Option<NodeId> {
        if !self.nodes.contains_key(parent) {
            return None;
        }
        let id = self.insert_node(Some(parent), initializer);
        self.nodes[parent].children.push(id);
        self.damaged = true;
        Some(id)
    }

    pub fn add_wl_surface(
        &mut self,
        parent: NodeId,
        surface_id: WlSurfaceId,
        wl_event_queue: calloop::channel::Sender<(WlSurfaceId, WlSurfaceEvent)>,
    ) {
        if let Some(id) = self.add_child(
            parent,
            WlSurfaceApplet::get_boxed_initiator((surface_id, wl_event_queue)),
        ) {
            self.wl_surfaces.insert(surface_id, id);
        }
    }

    /// Closes the node; its parent adopts its children in its place.
    /// If the focused child closed, the parent becomes focused.
    ///
    /// Returns whether something closed.
    pub fn close(&mut self, id: NodeId) -> bool {
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        let Some(parent_id) = node.parent else {
            log::warn!("Tried to close root");
            return false;
        };

        let closed = self.nodes.remove(id).unwrap();
        for &orphan in &closed.children {
            self.nodes[orphan].parent = Some(parent_id);
        }
        let parent = &mut self.nodes[parent_id];
        let index = parent
            .children
            .iter()
            .position(|&child| child == id)
            .unwrap();
        parent
            .children
            .splice(index..=index, closed.children.iter().copied());
        parent.focus = match parent.focus {
            FocusIndex::Child(focused) if focused == index => FocusIndex::Focusing,
            FocusIndex::Child(focused) if focused > index => {
                FocusIndex::Child(focused + closed.children.len() - 1)
            }
            focus => focus,
        };

        self.wl_surfaces.retain(|_, node_id| *node_id != id);
        self.damaged = true;
        true
    }

    /// Closes the node showing the surface
    pub fn close_wl_surface(&mut self, surface_id: WlSurfaceId) {
        if let Some(&id) = self.wl_surfaces.get(&surface_id) {
            self.close(id);
        }
    }

    /// Changes focus from the perspective of the node,
    /// passing the operation on to its ancestors as needed.
    pub fn change_focus(&mut self, id: NodeId, operation: WorldTreeTraversalOperation) {
        let mut id = id;
        let mut operation = Some(operation);
        while let Some(current_operation) = operation
            && let Some(node) = self.nodes.get_mut(id)
        {
            let (new_focus, parent_call) = node.calculate_new_focus(current_operation);
            if let Some(new_focus) = new_focus {
                node.focus = new_focus;
                self.damaged = true;
            }
            operation = parent_call;

            // the root has no parent to pass it to
            let Some(parent) = node.parent else {
                break;
            };
            id = parent;
        }
    }

    /// Focuses down to the node showing the surface.
    pub fn focus_wl_surface(&mut self, surface_id: WlSurfaceId) {
        let Some(&id) = self.wl_surfaces.get(&surface_id) else {
            return;
        };
        self.nodes[id].focus = FocusIndex::Inner;

        let mut child = id;
        while let Some(parent) = self.nodes[child].parent {
            let index = self.nodes[parent]
                .children
                .iter()
                .position(|&id| id == child)
                .unwrap();
            self.nodes[parent].focus = FocusIndex::Child(index);
            child = parent;
        }
        self.damaged = true;
    }

    /// Follows focused children down to the node that is focusing or focused on its main
    fn focused_node(&self) -> NodeId {
        let mut id = self.root;
        while let FocusIndex::Child(child_index) = self.nodes[id].focus {
            id = self.nodes[id].children[child_index];
        }
        id
    }

    /// `None` while a node itself is focused (`Focusing`), since it takes the keys then
    pub fn focused_wl_surface(&self) -> Option<WlSurfaceId> {
        let id = self.focused_node();
        if matches!(self.nodes[id].focus, FocusIndex::Focusing) {
            return None;
        }
        // TODO: wl surfaces inside an applet's inner tree
        self.wl_surfaces
            .iter()
            .find_map(|(&surface_id, &node_id)| (node_id == id).then_some(surface_id))
    }

    pub fn handle_event(&mut self, event: StandardEvent) {
        if let StandardEvent::UIEvent(UIEvent::WindowResized(size)) = event {
            // every applet is displayed at this size, not just the focused one
            self.latest_size = Some(size);
            for node in self.nodes.values() {
                node.main
                    .send_event(StandardEvent::UIEvent(UIEvent::WindowResized(size)));
            }
            return;
        }

        let id = self.focused_node();
        let node = &mut self.nodes[id];
        if matches!(node.focus, FocusIndex::Inner) {
            node.main.send_event(event);
            return;
        }

        // `Focusing`, so we can intercept
        // REVIEW: should focusing just auto go to inner?
        let StandardEvent::UIEvent(ui_event) = event else {
            node.main.send_event(event);
            return;
        };
        match &ui_event {
            UIEvent::Key {
                symbol,
                pressed: true,
                ..
            } => {
                if let Some(key) = symbol
                    && let Some(key_char) = key.to_char()
                    && let Some(operation) = WorldTreeTraversalOperation::from_char(key_char)
                {
                    self.change_focus(id, operation);
                } else {
                    // if key press not a traversal, then just set focus to inner and forward input
                    node.focus = FocusIndex::Inner;
                    node.main.send_event(event);
                    self.damaged = true;
                }
            }
            UIEvent::Key { pressed: false, .. } => {}
            UIEvent::Mouse(MouseEvent {
                kind: MouseEventKind::Button { pressed: true, .. },
                ..
            }) => {
                // set focus to inner and forward mouse press
                node.focus = FocusIndex::Inner;
                node.main.send_event(event);
                self.damaged = true;
            }
            UIEvent::WindowResized(_) | UIEvent::Mouse(_) => {
                // forward without changing focus (otherwise hovering would steal focus)
                node.main.send_event(event);
            }
        }
    }

    /// The focused applet's window
    pub fn get_window(&self) -> sonamu_ui::ui_element::UITree {
        self.nodes[self.focused_node()].main.get_surface()
    }

    pub fn get_treeview(&self) -> WorldTree<String> {
        self.get_treeview_of(self.root)
    }

    fn get_treeview_of(&self, id: NodeId) -> WorldTree<String> {
        let node = &self.nodes[id];
        let mut raw_treeview = RecursiveTreeNode::from_value(node.main.get_treeview());

        for &child in &node.children {
            match self.get_treeview_of(child) {
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
        self.get_focus_path_of(self.root)
    }

    fn get_focus_path_of(&self, id: NodeId) -> WorldTreePath {
        let node = &self.nodes[id];
        match node.focus {
            FocusIndex::Focusing => WorldTreePath::new_into(),
            FocusIndex::Inner => {
                // TODO: the focus path inside the main applet's inner tree; for now its root
                let mut path_tail = WorldTreePath::new_into().0.to_vec();

                path_tail.insert(0, TreeNodePath::new_root());

                WorldTreePath(path_tail.into_boxed_slice())
            }
            FocusIndex::Child(child_index) => {
                let mut path_tail = self
                    .get_focus_path_of(node.children[child_index])
                    .0
                    .to_vec();

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
