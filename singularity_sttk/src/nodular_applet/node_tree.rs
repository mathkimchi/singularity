use crate::{
    creatable_applet::CreatableNodularApplet, nodular_applet::NodularAppletInitializer,
    wl_surface_applet::WlSurfaceApplet,
};
use singularity_common::{
    sap::{
        client_handle::ClientHandle,
        detached::{Axis, DetachedClient, DetachedTree, Layout},
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
    color::Color,
    display_units::DisplayContainerSize,
    ui_element::{Length, Style, UIElement, UITree},
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

/// Shows its main applet or the focused child, or all its children at once (a split).
struct Node {
    /// Dropping it ends the applet. `None` for splits.
    main: Option<ClientHandle>,
    layout: Layout,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    focus: FocusIndex,
    /// Last size sent to `main`
    size: Option<DisplayContainerSize>,
    /// Becomes the view root when focus moves into it
    project: bool,
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
                // a split has no main to go into, so into its first pane
                WorldTreeTraversalOperation::NextLayer if self.main.is_none() => {
                    ((!self.children.is_empty()).then_some(FocusIndex::Child(0)), None)
                }
                WorldTreeTraversalOperation::NextLayer => (Some(FocusIndex::Inner), None),
                // TODO: no key makes these (`q`/`e` are the layer keys)
                WorldTreeTraversalOperation::Layerwise(
                    TreeTraverseOperation::BfsPrev | TreeTraverseOperation::BfsNext,
                ) => (None, None),
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
                WorldTreeTraversalOperation::GlobalRoot => {
                    (Some(FocusIndex::Focusing), Some(operation))
                }
                WorldTreeTraversalOperation::PrevLayer => (Some(FocusIndex::Focusing), None),
                // TODO: focus inside the applet's inner tree; for now only `PrevLayer` leaves it
                WorldTreeTraversalOperation::NextLayer
                | WorldTreeTraversalOperation::Layerwise(_) => (None, None),
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
                // focus is already below this node; only a stale request (eg from an applet whose
                // node lost focus meanwhile) gets these here
                WorldTreeTraversalOperation::NextLayer => (None, None),
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
                        TreeTraverseOperation::BfsPrev
                        | TreeTraverseOperation::BfsNext
                        | TreeTraverseOperation::Child(_)
                        | TreeTraverseOperation::LastChild => (None, None),
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

    /// The whole tree is displayed at this size
    latest_size: Option<DisplayContainerSize>,
    /// What is on screen, from the last `update_layout`
    panes: Vec<Pane>,
    /// The pane a mouse button was pressed in; it gets the pointer until the release
    pointer_grab: Option<NodeId>,
    /// Runs an applet as a client whose requests are tagged with the given node
    spawn_applet: Box<dyn Fn(NodeId, NodularAppletInitializer) -> ClientHandle>,
    /// Like `spawn_applet`, for a placed client that is already running
    attach_applet: Box<dyn Fn(NodeId, DetachedClient) -> ClientHandle>,
    /// Set when the tree changed in a way that needs a redraw
    damaged: bool,

    /// Only its subtree is shown (panes and treeview), as if it were the root.
    /// Focus always stays inside it: when focus leaves, it widens to the ancestor that has the focus.
    view_root: NodeId,
    /// The focused node at the last `update_view_root`, to tell when focus moves into a project
    last_focused: NodeId,
}

/// A node shown on screen, at `rect` (`[x, y, width, height]`, border included) of the tree's area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pane {
    node: NodeId,
    rect: [u32; 4],
}
impl Pane {
    /// Inside the border
    const fn inner_size(&self) -> DisplayContainerSize {
        DisplayContainerSize::new(
            self.rect[2].saturating_sub(2 * PANE_BORDER),
            self.rect[3].saturating_sub(2 * PANE_BORDER),
        )
    }

    fn contains(&self, [x, y]: [f64; 2]) -> bool {
        let [left, top, width, height] = self.rect.map(f64::from);
        (left..left + width).contains(&x) && (top..top + height).contains(&y)
    }
}

const PANE_BORDER: u32 = 1;
/// Focus is inside the pane's applet (keys go to it)
const BORDER_INNER: Color = Color::LIGHT_GREEN;
/// Focus is on the pane's node itself (traversal keys move it)
const BORDER_FOCUSING: Color = Color::LIGHT_YELLOW;
/// Rect colors are drawn as linear, so this shows as about #3a3f4a
const BORDER_UNFOCUSED: Color = Color([11, 13, 17, 0xFF]);

impl NodeTree {
    pub fn new(
        root_initializer: NodularAppletInitializer,
        spawn_applet: Box<dyn Fn(NodeId, NodularAppletInitializer) -> ClientHandle>,
        attach_applet: Box<dyn Fn(NodeId, DetachedClient) -> ClientHandle>,
    ) -> Self {
        let mut tree = Self {
            root: NodeId::default(),
            nodes: SlotMap::with_key(),
            wl_surfaces: HashMap::new(),
            latest_size: None,
            panes: Vec::new(),
            pointer_grab: None,
            spawn_applet,
            attach_applet,
            damaged: true,
            view_root: NodeId::default(),
            last_focused: NodeId::default(),
        };
        tree.root = tree.insert_node(None, root_initializer);
        tree.view_root = tree.root;
        tree.last_focused = tree.root;
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
            main: Some(spawn_applet(id, initializer)),
            layout: Layout::Single,
            parent,
            children: Vec::new(),
            focus: FocusIndex::Focusing,
            size: None,
            project: false,
        });
        self.send_initial_size(id);
        id
    }

    /// Attaches the tree's clients under `parent` (not added to its children), with new ids
    fn attach_subtree(&mut self, parent: NodeId, tree: DetachedTree) -> NodeId {
        let attach_applet = &self.attach_applet;
        let id = self.nodes.insert_with_key(|id| Node {
            main: tree.main.map(|main| attach_applet(id, main)),
            layout: tree.layout,
            parent: Some(parent),
            children: Vec::new(),
            focus: FocusIndex::Focusing,
            size: None,
            project: tree.project,
        });
        self.send_initial_size(id);
        if let Some(surface_id) = tree.wl_surface {
            self.wl_surfaces.insert(surface_id, id);
        }
        for child in tree.children {
            let child = self.attach_subtree(id, child);
            self.nodes[id].children.push(child);
        }
        id
    }

    /// Detaches the node and its descendants (already removed from its parent's children)
    fn detach_subtree(&mut self, id: NodeId) -> DetachedTree {
        let node = self.nodes.remove(id).unwrap();
        let wl_surface = self
            .wl_surfaces
            .iter()
            .find_map(|(&surface_id, &node_id)| (node_id == id).then_some(surface_id));
        if let Some(surface_id) = wl_surface {
            self.wl_surfaces.remove(&surface_id);
        }
        DetachedTree {
            main: node.main.map(ClientHandle::detach),
            layout: node.layout,
            wl_surface,
            project: node.project,
            children: node
                .children
                .into_iter()
                .map(|child| self.detach_subtree(child))
                .collect(),
        }
    }

    /// A guess until `update_layout` shows it: the whole tree's size
    fn send_initial_size(&mut self, id: NodeId) {
        if let Some(size) = self.latest_size {
            self.resize(id, size);
        }
    }

    fn resize(&mut self, id: NodeId, size: DisplayContainerSize) {
        let node = &mut self.nodes[id];
        if node.size != Some(size) {
            node.size = Some(size);
            if let Some(main) = &node.main {
                main.send_event(StandardEvent::UIEvent(UIEvent::WindowResized(size)));
            }
        }
    }

    pub fn send_event(&self, id: NodeId, event: StandardEvent) {
        if let Some(main) = self.nodes.get(id).and_then(|node| node.main.as_ref()) {
            main.send_event(event);
        }
    }

    /// Gives the event back if the node has no running applet to send it to
    /// (splits have none, and an applet may have ended, eg by panicking)
    // the error is just the event, as big as what was passed in
    #[allow(clippy::result_large_err)]
    pub fn try_send_event(&self, id: NodeId, event: StandardEvent) -> Result<(), StandardEvent> {
        match self.nodes.get(id).and_then(|node| node.main.as_ref()) {
            Some(main) => main.try_send_event(event),
            None => Err(event),
        }
    }

    /// The node at `path` of the node tree
    pub fn node_at(&self, path: &TreeNodePath) -> Option<NodeId> {
        let mut id = self.root;
        for &index in &path.0 {
            id = *self.nodes[id].children.get(index)?;
        }
        Some(id)
    }

    /// The deepest node along `path`, and whether it is the node at `path`
    pub fn closest_node_at(&self, path: &TreeNodePath) -> (NodeId, bool) {
        let mut id = self.root;
        for &index in &path.0 {
            let Some(&child) = self.nodes[id].children.get(index) else {
                return (id, false);
            };
            id = child;
        }
        (id, true)
    }

    /// The node's path in the node tree, `None` if it isn't in the tree
    pub fn path_of(&self, id: NodeId) -> Option<TreeNodePath> {
        let mut indices = Vec::new();
        let mut child = id;
        while let Some(parent) = self.nodes.get(child)?.parent {
            indices.push(self.child_index(parent, child));
            child = parent;
        }
        indices.reverse();
        Some(TreeNodePath(indices))
    }

    fn child_index(&self, parent: NodeId, child: NodeId) -> usize {
        self.nodes[parent]
            .children
            .iter()
            .position(|&id| id == child)
            .unwrap()
    }

    /// Takes the node out with its children.
    /// If it was its parent's focused child, the parent becomes focused.
    ///
    /// `None` for the root or a node not in the tree.
    ///
    /// # Panics
    ///
    /// If called while the loop is dispatching a request from one of the plucked applets.
    pub fn pluck(&mut self, id: NodeId) -> Option<DetachedTree> {
        let parent_id = self.nodes.get(id)?.parent?;
        let index = self.child_index(parent_id, id);
        let parent = &mut self.nodes[parent_id];
        parent.children.remove(index);
        parent.focus = match parent.focus {
            FocusIndex::Child(focused) if focused == index => FocusIndex::Focusing,
            FocusIndex::Child(focused) if focused > index => FocusIndex::Child(focused - 1),
            focus => focus,
        };
        self.damaged = true;
        if self.contains(id, self.view_root) {
            self.view_root = parent_id;
        }
        let plucked = self.detach_subtree(id);
        self.collapse_split(parent_id);
        Some(plucked)
    }

    /// Whether `descendant` is `ancestor` or below it
    fn contains(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut id = Some(descendant);
        while let Some(current) = id {
            if current == ancestor {
                return true;
            }
            id = self.nodes.get(current).and_then(|node| node.parent);
        }
        false
    }

    /// Replaces the node with a split showing it and a new applet, which gets focus
    pub fn split(&mut self, id: NodeId, axis: Axis, initializer: NodularAppletInitializer) {
        let Some(node) = self.nodes.get(id) else {
            return;
        };
        let (parent, project) = (node.parent, node.project);
        // the split takes its place, so it is the project now (and the view root), until it collapses back
        let split = self.nodes.insert(Node {
            main: None,
            layout: Layout::Split(axis),
            parent,
            children: vec![id],
            focus: FocusIndex::Child(0),
            size: None,
            project,
        });
        match parent {
            Some(parent) => {
                let index = self.child_index(parent, id);
                self.nodes[parent].children[index] = split;
            }
            None => self.root = split,
        }
        if self.view_root == id {
            self.view_root = split;
        }
        let node = &mut self.nodes[id];
        node.parent = Some(split);
        node.project = false;

        let new = self.insert_node(Some(split), initializer);
        self.nodes[split].children.push(new);
        self.focus_node(new);
        self.nodes[new].focus = FocusIndex::Inner;
    }

    /// A split left with one pane is replaced by that pane
    fn collapse_split(&mut self, id: NodeId) {
        let node = &self.nodes[id];
        if !matches!(node.layout, Layout::Split(_)) || node.children.len() != 1 {
            return;
        }
        let child = node.children[0];
        let parent = node.parent;
        match parent {
            Some(parent) => {
                let index = self.child_index(parent, id);
                self.nodes[parent].children[index] = child;
            }
            None => self.root = child,
        }
        if self.view_root == id {
            self.view_root = child;
        }
        let split = self.nodes.remove(id).unwrap();
        let child = &mut self.nodes[child];
        child.parent = parent;
        child.project |= split.project;
        self.damaged = true;
    }

    /// Adds the tree as the parent's last child (the root's if the parent is gone) and focuses it
    pub fn place(&mut self, parent: NodeId, tree: DetachedTree) -> NodeId {
        // eg it closed while its applet was being asked to place
        let parent = if self.nodes.contains_key(parent) {
            parent
        } else {
            self.root
        };
        let id = self.attach_subtree(parent, tree);
        self.nodes[parent].children.push(id);
        self.focus_node(id);
        self.nodes[id].focus = FocusIndex::Focusing;
        self.damaged = true;
        id
    }

    /// Points every ancestor's focus down to the node, without changing the node's own focus
    fn focus_node(&mut self, id: NodeId) {
        let mut child = id;
        while let Some(parent) = self.nodes[child].parent {
            let index = self.child_index(parent, child);
            self.nodes[parent].focus = FocusIndex::Child(index);
            child = parent;
        }
        self.damaged = true;
    }

    /// Focuses what `path` of the treeview points at: the node itself if the path ends at it,
    /// otherwise its main (focus inside it is TODO)
    pub fn focus_path(&mut self, path: &WorldTreePath) {
        let Some(id) = path.0.first().map_or(Some(self.root), |path| self.node_at(path)) else {
            return;
        };
        self.focus_node(id);
        self.nodes[id].focus = if path.0.len() > 1 {
            FocusIndex::Inner
        } else {
            FocusIndex::Focusing
        };
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

    /// Adds a tile for the surface as the parent's last child, and focuses inside it (so it's shown and gets keys)
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
            self.focus(id, true);
        }
    }

    /// Focuses the node itself, or inside its applet if `inner`
    pub fn focus(&mut self, id: NodeId, inner: bool) {
        self.focus_node(id);
        self.nodes[id].focus = if inner {
            FocusIndex::Inner
        } else {
            FocusIndex::Focusing
        };
    }

    /// Marking a node as a project makes it the view root, since focus is usually inside it then
    pub fn set_project(&mut self, id: NodeId, project: bool) {
        let Some(node) = self.nodes.get_mut(id) else {
            return;
        };
        node.project = project;
        if project {
            self.view_root = id;
        }
        self.damaged = true;
    }

    pub fn toggle_project(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get(id) {
            self.set_project(id, !node.project);
        }
    }

    /// Views only the focused node's subtree, or everything if it's already the view root
    pub fn toggle_view_root(&mut self) {
        let focused = self.focused_node();
        self.view_root = if focused == self.view_root {
            self.root
        } else {
            focused
        };
        self.damaged = true;
    }

    /// Widens the view root if focus left it, or narrows it to a project focus just moved into.
    /// Run after each batch of changes.
    pub fn update_view_root(&mut self) {
        let old = self.view_root;
        // pluck, close and split move it themselves; just in case
        if !self.nodes.contains_key(self.view_root) {
            self.view_root = self.root;
        }
        let focused = self.focused_node();
        // eg traversing to the parent past it
        while !self.contains(self.view_root, focused)
            && let Some(parent) = self.nodes[self.view_root].parent
        {
            self.view_root = parent;
        }
        // the outermost project inside the view that focus just moved into
        if focused != self.last_focused {
            let mut entered = None;
            let mut id = focused;
            while id != self.view_root {
                let node = &self.nodes[id];
                if node.project && !self.contains(id, self.last_focused) {
                    entered = Some(id);
                }
                let Some(parent) = node.parent else {
                    break;
                };
                id = parent;
            }
            if let Some(project) = entered {
                self.view_root = project;
            }
            self.last_focused = focused;
        }
        if self.view_root != old {
            self.damaged = true;
        }
    }

    /// The view root's path in the node tree
    pub fn view_root_path(&self) -> TreeNodePath {
        self.path_of(self.view_root)
            .unwrap_or_else(TreeNodePath::new_root)
    }

    /// The view root's parent: its path and label
    pub fn view_parent(&self) -> Option<(TreeNodePath, String)> {
        let parent = self.nodes[self.view_root].parent?;
        Some((self.path_of(parent)?, self.label(parent)))
    }

    /// Its applet's treeview label (or its layout's), marked if it's a project
    fn label(&self, id: NodeId) -> String {
        let mut treeview = self.applet_treeview(id);
        std::mem::take(root_label_mut(&mut treeview))
    }

    fn applet_treeview(&self, id: NodeId) -> WorldTree<String> {
        let node = &self.nodes[id];
        let mut treeview = node.main.as_ref().map_or_else(
            || WorldTree::Base(node.layout.label().to_string()),
            ClientHandle::get_treeview,
        );
        if node.project {
            root_label_mut(&mut treeview).insert_str(0, "◆ ");
        }
        treeview
    }

    /// Whether focus is inside the node's applet
    pub fn is_inner(&self, id: NodeId) -> bool {
        self.nodes
            .get(id)
            .is_some_and(|node| matches!(node.focus, FocusIndex::Inner))
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

        if self.view_root == id {
            self.view_root = parent_id;
        }
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
        self.collapse_split(parent_id);
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
        self.focus_node(id);
    }

    /// Follows focused children down to the node that is focusing or focused on its main
    pub fn focused_node(&self) -> NodeId {
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
        match event {
            // shown panes get their sizes in `update_layout`
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                self.latest_size = Some(size);
                self.damaged = true;
                return;
            }
            StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)) => {
                self.handle_mouse(mouse_event);
                return;
            }
            _ => {}
        }

        let id = self.focused_node();
        let node = &mut self.nodes[id];
        // a split has nothing to send to
        let Some(main) = &node.main else {
            if let StandardEvent::UIEvent(UIEvent::Key {
                symbol: Some(key),
                pressed: true,
                ..
            }) = event
                && let Some(key_char) = key.to_char()
                && let Some(operation) = WorldTreeTraversalOperation::from_char(key_char)
            {
                self.change_focus(id, operation);
            }
            return;
        };
        if matches!(node.focus, FocusIndex::Inner) {
            main.send_event(event);
            return;
        }

        // `Focusing`, so we can intercept
        // REVIEW: should focusing just auto go to inner?
        let StandardEvent::UIEvent(ui_event) = event else {
            main.send_event(event);
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
                    main.send_event(event);
                    self.damaged = true;
                }
            }
            UIEvent::Key { pressed: false, .. } => {}
            // handled above
            UIEvent::WindowResized(_) | UIEvent::Mouse(_) => {}
        }
    }

    /// Sends the pane under the pointer (or the one holding the pointer grab) the event, relative to it.
    /// A press focuses the pane's applet.
    fn handle_mouse(&mut self, mut mouse_event: MouseEvent) {
        let pane = match self.pointer_grab {
            Some(id) => self.panes.iter().find(|pane| pane.node == id),
            None => self
                .panes
                .iter()
                .find(|pane| pane.contains(mouse_event.position)),
        };
        let Some(&pane) = pane else {
            return;
        };

        if let MouseEventKind::Button { pressed, .. } = mouse_event.kind {
            self.pointer_grab = pressed.then_some(pane.node);
            // panes are from the last `update_layout`, so the node may have closed since
            if pressed
                && self
                    .nodes
                    .get(pane.node)
                    .is_some_and(|node| node.main.is_some())
            {
                self.focus_node(pane.node);
                self.nodes[pane.node].focus = FocusIndex::Inner;
            }
        }
        mouse_event.position[0] -= f64::from(pane.rect[0] + PANE_BORDER);
        mouse_event.position[1] -= f64::from(pane.rect[1] + PANE_BORDER);
        self.send_event(pane.node, StandardEvent::UIEvent(UIEvent::Mouse(mouse_event)));
    }

    /// The node `id` shows: following focused children, but stopping at splits, which show every pane
    fn displayed_node(&self, mut id: NodeId) -> NodeId {
        loop {
            let node = &self.nodes[id];
            match (node.layout, node.focus) {
                (Layout::Single, FocusIndex::Child(index)) => id = node.children[index],
                _ => return id,
            }
        }
    }

    /// Works out what is on screen, and resizes the shown applets to their panes
    pub fn update_layout(&mut self) {
        let Some(size) = self.latest_size else {
            return;
        };
        let mut panes = Vec::new();
        self.push_panes(self.view_root, [0, 0, size.width, size.height], &mut panes);
        for pane in &panes {
            self.resize(pane.node, pane.inner_size());
        }
        if self.pointer_grab.is_some_and(|id| !panes.iter().any(|pane| pane.node == id)) {
            self.pointer_grab = None;
        }
        self.panes = panes;
    }

    fn push_panes(&self, id: NodeId, rect: [u32; 4], panes: &mut Vec<Pane>) {
        let id = self.displayed_node(id);
        let node = &self.nodes[id];
        let Layout::Split(axis) = node.layout else {
            panes.push(Pane { node: id, rect });
            return;
        };

        let [x, y, width, height] = rect;
        let count = node.children.len() as u32;
        let length = match axis {
            Axis::Horizontal => width,
            Axis::Vertical => height,
        };
        for (index, &child) in (0..).zip(&node.children) {
            // the last pane takes the rounding leftovers
            let start = length * index / count;
            let end = length * (index + 1) / count;
            let child_rect = match axis {
                Axis::Horizontal => [x + start, y, end - start, height],
                Axis::Vertical => [x, y + start, width, end - start],
            };
            self.push_panes(child, child_rect, panes);
        }
    }

    /// Every pane, bordered; the focused one's border shows whether focus is inside it
    pub fn get_window(&self) -> UITree {
        let focused = self.focused_node();
        let panes = self
            .panes
            .iter()
            .map(|pane| {
                let node = &self.nodes[pane.node];
                let border = match node.focus {
                    _ if pane.node != focused => BORDER_UNFOCUSED,
                    FocusIndex::Inner => BORDER_INNER,
                    _ => BORDER_FOCUSING,
                };
                let surface = node.main.as_ref().map_or_else(
                    || UITree::leaf(UIElement::Nothing),
                    ClientHandle::get_surface,
                );
                let [x, y, width, height] = pane.rect.map(|value| value as f32);
                UITree::new_styled(
                    UIElement::Bordered(border),
                    Style {
                        border: Length::uniform_rect(Length::Px(PANE_BORDER as f32)),
                        ..Style::absolute([x, y], [width, height])
                    },
                    vec![surface],
                )
            })
            .collect();
        UITree::new_container(panes)
    }

    /// The view root's subtree; its root is at `view_root_path`
    pub fn get_view_treeview(&self) -> WorldTree<String> {
        self.get_treeview_of(self.view_root)
    }

    fn get_treeview_of(&self, id: NodeId) -> WorldTree<String> {
        let node = &self.nodes[id];
        let mut raw_treeview = RecursiveTreeNode::from_value(self.applet_treeview(id));

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

fn root_label_mut(treeview: &mut WorldTree<String>) -> &mut String {
    match treeview {
        WorldTree::Base(label) => label,
        WorldTree::World(node) => root_label_mut(node.get_value_mut()),
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
