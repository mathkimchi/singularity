//! A project: an applet holding the project's contents as its inner tree, like the root holds the outer tree.
//! Its node doesn't know it is a project; it only sees an applet with an inner tree.

use crate::nodular_applet::{
    Answer, AppletContext, EmbeddedId, NodeRequest, NodularAppletInitializer, StandardApplet,
    WrapInitializer,
    node_holder::NodeHolder,
    node_tree::{AttachClient, NodeId, NodeTree, SpawnClient, root_label_mut},
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
    display_units::DisplayContainerSize,
    ui_element::{UIElement, UITree},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Its inner tree root's label
const LABEL: &str = "Project";

/// Shows its contents' panes; the root's treeview shows its contents as its node's inner tree.
/// Its inner tree's root is a container (no main), so focus inside the project at its root (`[[project], []]`)
/// can add children to the project, apart from focus on the project's node (`[[project]]`).
///
/// Closing it (`CloseRequest`, eg Ctrl+W on its node) gives its contents back to its node, which they replace.
pub struct ProjectApplet {
    holder: NodeHolder,
    hook: AppletContext,
    /// The node each embedded applet (its contents' mains) is the main of
    nodes: Rc<RefCell<HashMap<EmbeddedId, NodeId>>>,
    /// Its contents were given back, so its tree is empty
    dissolved: bool,
}
impl ProjectApplet {
    /// Makes a project holding a subtree (`NodeRequest::Wrap`), under its inner root
    pub fn wrapper() -> WrapInitializer {
        Box::new(|subtree| {
            Box::new(move |hook| Box::new(Self::new(subtree, hook)) as Box<dyn StandardApplet>)
        })
    }

    /// Makes a saved project again, holding its saved contents
    pub fn restorer(contents: SavedTree) -> NodularAppletInitializer {
        Box::new(move |hook| {
            Box::new(Self::with_tree(hook, |spawn_client, attach_client| {
                NodeTree::restore(contents, spawn_client, attach_client)
            })) as Box<dyn StandardApplet>
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
        }
    }
}
impl StandardApplet for ProjectApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        if self.dissolved {
            return;
        }
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

    fn get_window_standard_applet(&self, _container_size: DisplayContainerSize) -> UITree {
        if self.dissolved {
            return UITree::leaf(UIElement::Nothing);
        }
        self.holder.tree.get_window()
    }

    /// Its tree, whose root (a container) is labeled as the project
    fn get_treeview(&self) -> WorldTree<String> {
        if self.dissolved {
            return WorldTree::Base(LABEL.to_string());
        }
        let mut treeview = self.holder.tree.get_treeview();
        *root_label_mut(&mut treeview) = LABEL.to_string();
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
        (!self.dissolved).then(|| SavedApplet::Project(Box::new(self.holder.tree.save())))
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
