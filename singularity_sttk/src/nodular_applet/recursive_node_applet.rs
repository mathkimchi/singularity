use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
    caching_applet::CachingApplet,
};
use singularity_common::{
    sync::EncapsulatedLock,
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TreeNodePath, TreeTraverseOperation},
        world_tree::{WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation},
    },
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::UIElement,
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};
use std::sync::{Arc, RwLock, Weak, atomic::AtomicBool};

#[derive(Debug, Clone, Copy)]
enum FocusIndex {
    Focusing,
    Inner,
    Child(usize),
}

/// Information needed for the main applet and child applet hooks
/// REVIEW: rename
struct SharedResource {
    children: RwLock<Vec<Arc<CachingApplet>>>,

    focus_index: EncapsulatedLock<FocusIndex>,
    hook: Box<dyn NodularRunnerHook>,

    window_damaged: AtomicBool,
    treeview_damaged: AtomicBool,
}
impl SharedResource {
    pub fn new(hook: Box<dyn NodularRunnerHook>) -> Arc<Self> {
        let hook = hook;
        let children = RwLock::new(Vec::new());
        let focus_index = EncapsulatedLock::new(FocusIndex::Focusing);
        let window_damaged = AtomicBool::new(true);
        let treeview_damaged = AtomicBool::new(true);

        Arc::new(Self {
            children,
            focus_index,
            hook,
            window_damaged,
            treeview_damaged,
        })
    }

    /// NOTE: this can't take `&self` because we need to create a weak reference to shared resource,
    /// so we need to get this already wrapped in an Arc.
    pub fn add_child(
        shared_resource: &Arc<Self>,
        child_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) {
        let child_holder = {
            let inner_applet_window = EncapsulatedLock::new(UIElement::Nothing);
            let inner_applet_treeview =
                EncapsulatedLock::new(WorldTree::new_base(String::from("Hi")));

            struct InnerHook {
                shared_resource: Weak<SharedResource>,
            }
            impl BasicRunnerHook for InnerHook {
                fn damage_window(&self) {
                    if !self
                        .shared_resource
                        .upgrade()
                        .unwrap()
                        .window_damaged
                        .swap(true, std::sync::atomic::Ordering::Relaxed)
                    {
                        self.shared_resource.upgrade().unwrap().hook.damage_window();
                    }
                }

                fn close(&self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn damage_treeview(&self) {
                    if !self
                        .shared_resource
                        .upgrade()
                        .unwrap()
                        .treeview_damaged
                        .swap(true, std::sync::atomic::Ordering::Relaxed)
                    {
                        self.shared_resource
                            .upgrade()
                            .unwrap()
                            .hook
                            .damage_treeview();
                    }
                }

                fn add_child(&self, initializer: NodularAppletInitializer) {
                    SharedResource::add_child(
                        &self.shared_resource.upgrade().unwrap(),
                        initializer,
                    );
                }

                fn change_focus(&self, operation: WorldTreeTraversalOperation) {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .change_focus(operation);
                }

                fn register_applet_spawner(
                    &self,
                    name: String,
                    applet_spawner: super::AppletSpawner,
                ) {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .register_applet_spawner(name, applet_spawner);
                }
                fn get_applet_spawners(
                    &self,
                ) -> std::collections::BTreeMap<String, super::AppletSpawner> {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .get_applet_spawners()
                }
                fn find_applet_spawner(&self, name: String) -> Option<super::AppletSpawner> {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .find_applet_spawner(name)
                }
            }

            let inner_hook = InnerHook {
                shared_resource: Arc::downgrade(shared_resource),
            };

            Arc::new(CachingApplet::new(
                child_initializer,
                Box::new(inner_hook),
                inner_applet_window,
                inner_applet_treeview,
            ))
        };

        shared_resource.children.write().unwrap().push(child_holder);

        if !shared_resource
            .treeview_damaged
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            shared_resource.hook.damage_treeview();
        }
        if !shared_resource
            .window_damaged
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            shared_resource.hook.damage_window();
        }
    }

    /// Doesn't actually change anything.
    /// Just returns the new focus and what to call for the parent.
    fn calculate_new_focus(
        &self,
        operation: WorldTreeTraversalOperation,
    ) -> (Option<FocusIndex>, Option<WorldTreeTraversalOperation>) {
        match self.focus_index.get() {
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
                    if child_index < self.children.read().unwrap().len() {
                        (Some(FocusIndex::Child(child_index)), None)
                    } else {
                        (None, None)
                    }
                }
                WorldTreeTraversalOperation::Layerwise(TreeTraverseOperation::LastChild) => {
                    let num_children = self.children.read().unwrap().len();
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
                                    .clamp(0, self.children.read().unwrap().len() as isize - 1)
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

    /// FIXME: Bruh, this needs to differentiate between if a child is calling this
    /// or if its being called from self.
    /// Right now, this is only being called by children,
    /// but it would make more sense if these were from the perspective of itself.
    /// It is technically redundant but makes sense to change behavior
    /// based on the current focus path.
    pub fn change_focus(&self, operation: WorldTreeTraversalOperation) {
        let (new_focus, parent_call) = self.calculate_new_focus(operation);

        if let Some(new_focus) = new_focus {
            self.focus_index.set(new_focus);
        }
        if let Some(parent_call) = parent_call {
            self.hook.change_focus(parent_call);
        }

        self.hook.damage_window();
        self.hook.damage_treeview();
    }
}

/// Holds a main Applet (at index 0) and also children.
/// Displays the main child
pub struct RecursiveNodeApplet {
    main_applet: CachingApplet,
    shared_resource: Arc<SharedResource>,
}
impl RecursiveNodeApplet {
    fn new(
        main_applet_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn NodularRunnerHook>,
    ) -> Self {
        let shared_resource = SharedResource::new(hook);

        let main_applet = {
            let inner_applet_window = EncapsulatedLock::new(UIElement::Nothing);
            let inner_applet_treeview =
                EncapsulatedLock::new(WorldTree::new_base(String::from("Hi")));

            struct InnerHook {
                shared_resource: Weak<SharedResource>,
            }
            impl BasicRunnerHook for InnerHook {
                fn damage_window(&self) {
                    if !self
                        .shared_resource
                        .upgrade()
                        .unwrap()
                        .window_damaged
                        .swap(true, std::sync::atomic::Ordering::Relaxed)
                    {
                        self.shared_resource.upgrade().unwrap().hook.damage_window();
                    }
                }

                fn close(&self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn damage_treeview(&self) {
                    if !self
                        .shared_resource
                        .upgrade()
                        .unwrap()
                        .treeview_damaged
                        .swap(true, std::sync::atomic::Ordering::Relaxed)
                    {
                        self.shared_resource
                            .upgrade()
                            .unwrap()
                            .hook
                            .damage_treeview();
                    }
                }

                fn add_child(&self, initializer: NodularAppletInitializer) {
                    SharedResource::add_child(
                        &self.shared_resource.upgrade().unwrap(),
                        initializer,
                    );
                }

                fn change_focus(&self, operation: WorldTreeTraversalOperation) {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .change_focus(operation);
                }

                fn register_applet_spawner(
                    &self,
                    name: String,
                    applet_spawner: super::AppletSpawner,
                ) {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .register_applet_spawner(name, applet_spawner);
                }
                fn get_applet_spawners(
                    &self,
                ) -> std::collections::BTreeMap<String, super::AppletSpawner> {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .get_applet_spawners()
                }
                fn find_applet_spawner(&self, name: String) -> Option<super::AppletSpawner> {
                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .find_applet_spawner(name)
                }
            }

            let inner_hook = InnerHook {
                // window: inner_applet_window.clone(),
                shared_resource: Arc::downgrade(&shared_resource),
                // treeview: inner_applet_treeview.clone(),
            };

            CachingApplet::new(
                main_applet_initiator,
                Box::new(inner_hook),
                inner_applet_window,
                inner_applet_treeview,
            )
        };

        Self {
            main_applet,
            shared_resource,
        }
    }

    /// Partial application
    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        move |hook: Box<dyn NodularRunnerHook>| Self::new(inner_initializer, hook)
    }

    pub fn get_boxed_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet> {
        move |hook| Box::new(Self::new(inner_initializer, hook))
    }

    /// Ik, horrible name
    pub fn boxed_get_boxed_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> Box<impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>> {
        Box::new(Self::get_boxed_initializer(inner_initializer))
    }

    fn get_display(&self, container_size: DisplayContainerSize) -> UIElement {
        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing | FocusIndex::Inner => self.main_applet.get_window(container_size),
            FocusIndex::Child(child_index) => {
                self.shared_resource.children.read().unwrap()[child_index]
                    .get_window(container_size)
            }
        }
    }
}
impl BasicApplet for RecursiveNodeApplet {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing => {
                // We can intercept

                if let UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: false,
                        caps_lock: false,
                        logo: false,
                    },
                ) = &ui_event
                    && let Some(key_char) = key.to_char()
                    && let Some(operation) = WorldTreeTraversalOperation::from_char(key_char)
                {
                    self.shared_resource.change_focus(operation);
                } else {
                    // if not a traversal, then just set focus to inner and forward input
                    self.shared_resource
                        .change_focus(WorldTreeTraversalOperation::NextLayer);
                    self.handle_ui_event(ui_event);
                }
            }
            FocusIndex::Inner => self.main_applet.immut_handle_ui_event(ui_event),
            FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                [child_index]
                .immut_handle_ui_event(ui_event),
        }
    }

    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement {
        // TODO: return cached if damaged is already false?
        // REVIEW: idk why this is necessary if I already have caching browser; I'm not touching it rn
        self.shared_resource
            .window_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        self.get_display(container_size)
    }
}
impl NodularApplet for RecursiveNodeApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        match nodular_event {
            NodularEvent::Highlighted(_) => todo!(),
            NodularEvent::Focused(state) => {
                // self.shared_resource.applets.read().unwrap()[self
                //     .shared_resource
                //     .focus_index
                //     .load(std::sync::atomic::Ordering::Relaxed)]
                // .immut_handle_nodular_event(NodularEvent::Focused(state));

                // get_focused_applet!(
                //     self,
                //     immut_handle_nodular_event(NodularEvent::Focused(state))
                // );

                match self.shared_resource.focus_index.get() {
                    FocusIndex::Focusing | FocusIndex::Inner => self
                        .main_applet
                        .immut_handle_nodular_event(NodularEvent::Focused(state)),
                    FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                        [child_index]
                        .immut_handle_nodular_event(NodularEvent::Focused(state)),
                }
            }
        }
    }

    fn get_treeview(&self) -> WorldTree<String> {
        self.shared_resource
            .treeview_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut raw_treeview = RecursiveTreeNode::from_value(self.main_applet.get_treeview());

        for child_index in 0..(self.shared_resource.children.read().unwrap().len()) {
            let child_treeview =
                self.shared_resource.children.read().unwrap()[child_index].get_treeview();

            match child_treeview {
                WorldTree::World(recursive_tree_node) => {
                    raw_treeview.push_child_node(*recursive_tree_node);
                }
                _ => {
                    // REVIEW: should this ever even happen?
                    raw_treeview.push_child_node(RecursiveTreeNode::from_value(child_treeview));
                }
            }
        }

        WorldTree::World(Box::new(raw_treeview))
    }

    fn get_focus_path(&self) -> WorldTreePath {
        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing => WorldTreePath::new_into(),
            FocusIndex::Inner => {
                let mut path_tail = self.main_applet.get_focus_path().0.to_vec();

                path_tail.insert(0, TreeNodePath::new_root());

                WorldTreePath(path_tail.into_boxed_slice())
            }
            FocusIndex::Child(child_index) => {
                let mut path_tail = self.shared_resource.children.read().unwrap()[child_index]
                    .get_focus_path()
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
