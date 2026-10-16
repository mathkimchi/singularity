//! Clients with no holder, for moving them between holders (pluck and place)

use crate::{
    sap::{
        client_handle::ClientHandle,
        packets::{StandardEvent, WlSurfaceId},
        shared_state::SharedState,
    },
    utils::tree::{recursive_tree::RecursiveTreeNode, world_tree::WorldTree},
};
use calloop::{LoopHandle, channel::Channel};
use sonamu_ui::ui_element::UITree;
use std::any::Any;

/// A client taken out of its holder's event loop (`ClientHandle::detach`).
///
/// Its thread keeps running; its requests queue up until it is attached again.
pub struct DetachedClient {
    pub(crate) event_queue: calloop::channel::Sender<StandardEvent>,
    pub(crate) surface: SharedState<UITree>,
    pub(crate) treeview: SharedState<WorldTree<String>>,
    /// The client's `Channel<Request>`; type-erased so holders that can't name `Request` can still hold it
    pub(crate) requests: Box<dyn Any + Send>,
}
impl DetachedClient {
    /// Queued requests are handled once the event loop runs.
    ///
    /// # Panics
    ///
    /// If `Request` isn't the type the client was spawned with.
    pub fn attach<Request: Send + 'static, Data: 'static>(
        self,
        event_loop: &LoopHandle<'static, Data>,
        on_request: impl FnMut(Request, &mut Data) + 'static,
    ) -> ClientHandle {
        let requests = *self
            .requests
            .downcast::<Channel<Request>>()
            .expect("attached with a different request type");
        ClientHandle::attach(
            self.event_queue,
            self.surface,
            self.treeview,
            requests,
            event_loop,
            on_request,
        )
    }

    /// The last published treeview, without acking it
    pub fn peek_treeview(&self) -> WorldTree<String> {
        self.treeview.get()
    }
}
impl std::fmt::Debug for DetachedClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetachedClient").finish_non_exhaustive()
    }
}

/// A plucked part of a tree: a client with its children
#[derive(Debug)]
pub struct DetachedTree {
    pub main: DetachedClient,
    /// Set if `main` shows this wl surface
    pub wl_surface: Option<WlSurfaceId>,
    pub children: Vec<Self>,
}
impl DetachedTree {
    /// Like a holder's treeview: each client's treeview, arranged as the tree
    pub fn peek_treeview(&self) -> WorldTree<String> {
        let mut node = RecursiveTreeNode::from_value(self.main.peek_treeview());
        for child in &self.children {
            match child.peek_treeview() {
                WorldTree::World(child_node) => node.push_child_node(*child_node),
                base => node.push_child_node(RecursiveTreeNode::from_value(base)),
            }
        }
        WorldTree::World(Box::new(node))
    }
}
