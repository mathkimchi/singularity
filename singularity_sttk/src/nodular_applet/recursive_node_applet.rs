use crate::{
    basic_applet::{BasicApplet, BasicRunnerHook},
    creatable_applet::CreatableNodularApplet,
    nodular_applet::{
        NodularAppletInitializer, NodularRunnerHook, StandardApplet, caching_applet::CachingApplet,
    },
    wl_surface_applet::WlSurfaceApplet,
};
use singularity_common::{
    sap::packets::{StandardEvent, WlSurfaceId},
    sync::EncapsulatedLock,
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
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::{RwLock, atomic::AtomicBool},
};

#[derive(Debug, Clone, Copy)]
enum FocusIndex {
    Focusing,
    Inner,
    Child(usize),
}

/// A child of a `RecursiveNodeApplet`; can be moved to another node.
pub struct NodeChild {
    applet: CachingApplet,
    slot: Rc<ChildSlot>,
}

/// Shared between a `NodeChild` and its hook
struct ChildSlot {
    parent: RefCell<Weak<SharedResource>>,
    /// `Some` once closed, holding the children for the parent to adopt.
    /// Removal is deferred since the parent may be holding a lock on `children`.
    closed: RefCell<Option<Vec<NodeChild>>>,
}

/// Information needed for the main applet and child applet hooks
/// REVIEW: rename
struct SharedResource {
    children: RwLock<Vec<NodeChild>>,

    focus_index: EncapsulatedLock<FocusIndex>,
    hook: Box<dyn NodularRunnerHook>,

    window_damaged: AtomicBool,
    treeview_damaged: AtomicBool,

    /// So newly added children can be given the size
    latest_size: Cell<Option<DisplayContainerSize>>,
}
impl SharedResource {
    pub fn new(hook: Box<dyn NodularRunnerHook>) -> Rc<Self> {
        let hook = hook;
        let children = RwLock::new(Vec::new());
        let focus_index = EncapsulatedLock::new(FocusIndex::Focusing);
        let window_damaged = AtomicBool::new(true);
        let treeview_damaged = AtomicBool::new(true);

        Rc::new(Self {
            children,
            focus_index,
            hook,
            window_damaged,
            treeview_damaged,
            latest_size: Cell::new(None),
        })
    }

    fn create_hold_holder(
        shared_resource: &Rc<Self>,
        child_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
    ) -> NodeChild {
        let inner_applet_window = EncapsulatedLock::new(UITree::leaf(UIElement::Nothing));
        let inner_applet_treeview = EncapsulatedLock::new(WorldTree::new_base(String::from("Hi")));

        struct InnerHook {
            // outer_children: Arc<Mutex<Vec<SubAppletHolder>>>,
            // // outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
            // outer_hook: Arc<Box<dyn BasicRunnerHook>>,
            // window: EncapsulatedLock<UIElement>,
            // treeview: EncapsulatedLock<WorldTree<String>>,
            // outer_focused_child_index: Arc<Mutex<usize>>,
            slot: Rc<ChildSlot>,
            // the index of this hook's corresponding app in the shared resource list of applets
            // index: usize,
        }
        impl InnerHook {
            fn parent(&self) -> Rc<SharedResource> {
                self.slot.parent.borrow().upgrade().unwrap()
            }
        }
        impl BasicRunnerHook for InnerHook {
            // fn update_display(&self, display: &UIElement) {
            //     self.window.set(display.clone());

            //     self.shared_resource
            //         .upgrade()
            //         .unwrap()
            //         .hook
            //         .update_display(&MultiAppletHolder::get_display(&self.shared_resource));
            // }

            fn damage_window(&self) {
                if !self
                    .parent()
                    .window_damaged
                    .swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    self.parent().hook.damage_window();
                }
            }

            fn close(&self) {
                self.close_with_children(Vec::new());
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, treeview: &UIElement) {
            //     self.treeview.set(treeview.clone());

            //     self.shared_resource
            //         .upgrade()
            //         .unwrap()
            //         .hook
            //         .update_treeview(&MultiAppletHolder::get_display(&self.shared_resource));
            // }

            fn damage_treeview(&self) {
                if !self
                    .parent()
                    .treeview_damaged
                    .swap(true, std::sync::atomic::Ordering::Relaxed)
                {
                    self.parent().hook.damage_treeview();
                }
            }

            fn add_child(&self, initializer: NodularAppletInitializer) {
                SharedResource::add_child(&self.parent(), initializer);
            }

            fn change_focus(&self, operation: WorldTreeTraversalOperation) {
                self.parent().change_focus(operation);
            }

            fn close_with_children(&self, children: Vec<NodeChild>) {
                self.slot
                    .closed
                    .borrow_mut()
                    .get_or_insert_default()
                    .extend(children);
                self.damage_window();
                self.damage_treeview();
            }

            fn quit(&self) {
                self.parent().hook.quit();
            }

            fn register_applet_spawner(&self, name: String, applet_spawner: super::AppletSpawner) {
                self.parent()
                    .hook
                    .register_applet_spawner(name, applet_spawner);
            }
            fn get_applet_spawners(
                &self,
            ) -> std::collections::BTreeMap<String, super::AppletSpawner> {
                self.parent().hook.get_applet_spawners()
            }
            fn find_applet_spawner(&self, name: String) -> Option<super::AppletSpawner> {
                self.parent().hook.find_applet_spawner(name)
            }
        }

        let slot = Rc::new(ChildSlot {
            parent: RefCell::new(Rc::downgrade(shared_resource)),
            closed: RefCell::new(None),
        });
        let inner_hook = InnerHook {
            // window: inner_applet_window.clone(),
            // index: shared_resource.children.read().unwrap().len(),
            slot: slot.clone(),
            // treeview: inner_applet_treeview.clone(),
        };

        NodeChild {
            applet: CachingApplet::new(
                child_initializer,
                Box::new(inner_hook),
                inner_applet_window,
                inner_applet_treeview,
            ),
            slot,
        }
    }

    /// NOTE: this can't take `&self` because we need to create a weak reference to shared resource,
    /// so we need to get this already wrapped in an Arc.
    pub fn add_child(
        shared_resource: &Rc<Self>,
        child_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
    ) {
        // breaks when adding the first child
        // shared_resource.upgrade().unwrap().applets.read().unwrap()[shared_resource
        //     .upgrade()
        //     .unwrap()
        //     .focus_index
        //     .load(std::sync::atomic::Ordering::Relaxed)]
        // .immut_handle_nodular_event(NodularEvent::Focused(false));

        let child_holder = Self::create_hold_holder(shared_resource, child_initializer);

        if let Some(size) = shared_resource.latest_size.get() {
            child_holder
                .applet
                .immut_handle_ui_event(UIEvent::WindowResized(size));
        }
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

    /// Replaces each closed child with its children.
    /// If the focused child closed, this node becomes focused.
    fn remove_closed_children(shared_resource: &Rc<Self>) {
        let mut children = shared_resource.children.write().unwrap();
        if children
            .iter()
            .all(|child| child.slot.closed.borrow().is_none())
        {
            return;
        }

        let old_focus = shared_resource.focus_index.get();
        let mut new_focus = old_focus;
        let mut new_children = Vec::with_capacity(children.len());
        for (i, child) in std::mem::take(&mut *children).into_iter().enumerate() {
            let is_focused = matches!(old_focus, FocusIndex::Child(focused) if focused == i);
            match child.slot.closed.take() {
                None => {
                    if is_focused {
                        new_focus = FocusIndex::Child(new_children.len());
                    }
                    new_children.push(child);
                }
                Some(orphans) => {
                    if is_focused {
                        new_focus = FocusIndex::Focusing;
                    }
                    for orphan in orphans {
                        *orphan.slot.parent.borrow_mut() = Rc::downgrade(shared_resource);
                        new_children.push(orphan);
                    }
                }
            }
        }
        *children = new_children;
        drop(children);

        shared_resource.focus_index.set(new_focus);
        shared_resource.hook.damage_window();
        shared_resource.hook.damage_treeview();
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
    shared_resource: Rc<SharedResource>,
}
impl RecursiveNodeApplet {
    fn new(
        main_applet_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn NodularRunnerHook>,
    ) -> Self {
        let shared_resource = SharedResource::new(hook);

        let main_applet = {
            let inner_applet_window = EncapsulatedLock::new(UITree::leaf(UIElement::Nothing));
            let inner_applet_treeview =
                EncapsulatedLock::new(WorldTree::new_base(String::from("Hi")));

            struct InnerHook {
                // outer_children: Arc<Mutex<Vec<SubAppletHolder>>>,
                // // outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
                // outer_hook: Arc<Box<dyn BasicRunnerHook>>,
                // window: EncapsulatedLock<UIElement>,
                // treeview: EncapsulatedLock<WorldTree<String>>,
                // outer_focused_child_index: Arc<Mutex<usize>>,
                shared_resource: Weak<SharedResource>,
            }
            impl BasicRunnerHook for InnerHook {
                // fn update_display(&self, display: &UIElement) {
                //     self.window.set(display.clone());

                //     self.shared_resource
                //         .upgrade()
                //         .unwrap()
                //         .hook
                //         .update_display(&MultiAppletHolder::get_display(&self.shared_resource));
                // }

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
                    self.close_with_children(Vec::new());
                }
            }
            impl NodularRunnerHook for InnerHook {
                // fn update_treeview(&self, treeview: &UIElement) {
                //     self.treeview.set(treeview.clone());

                //     self.shared_resource
                //         .upgrade()
                //         .unwrap()
                //         .hook
                //         .update_treeview(&MultiAppletHolder::get_display(&self.shared_resource));
                // }

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

                /// Closing the main applet closes this node, so the parent adopts this node's children
                fn close_with_children(&self, children: Vec<NodeChild>) {
                    let shared_resource = self.shared_resource.upgrade().unwrap();
                    let mut orphans =
                        std::mem::take(&mut *shared_resource.children.write().unwrap());
                    orphans.extend(children);
                    shared_resource.hook.close_with_children(orphans);
                }

                fn quit(&self) {
                    self.shared_resource.upgrade().unwrap().hook.quit();
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
                shared_resource: Rc::downgrade(&shared_resource),
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
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        move |hook: Box<dyn NodularRunnerHook>| Self::new(inner_initializer, hook)
    }

    pub fn get_boxed_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet> {
        move |hook| Box::new(Self::new(inner_initializer, hook))
    }

    /// Ik, horrible name
    pub fn boxed_get_boxed_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>,
    ) -> Box<impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn StandardApplet>> {
        Box::new(Self::get_boxed_initializer(inner_initializer))
    }

    // fn get_focused_applet(&self) -> Arc<&SubAppletHolder> {
    //     match self.shared_resource.focus_index.get() {
    //         FocusIndex::Focusing | FocusIndex::Inner => Arc::new(&self.main_applet),
    //         FocusIndex::Child(child_index) => {
    //             &self.shared_resource.children.read().unwrap()[child_index]
    //         }
    //     }
    // }

    fn get_display(&self, container_size: DisplayContainerSize) -> UITree {
        // let mut applet_displays = Vec::new();

        // for applet in self.shared_resource.applets.read().unwrap().iter() {
        //     applet_displays.push(applet.get_window());
        // }

        // Self::combine_displays(applet_displays)

        // NOTE: above was implementation for equally divided

        // get_focused_applet!(self, |f| f.get_window())
        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing | FocusIndex::Inner => self.main_applet.get_window(container_size),
            FocusIndex::Child(child_index) => {
                // log::debug!(
                //     "This title: {}, returning {}th child named {}'s window",
                //     self.get_treeview().get_root_value(),
                //     child_index,
                //     self.shared_resource.children.read().unwrap()[child_index]
                //         .get_treeview()
                //         .get_root_value()
                // );
                self.shared_resource.children.read().unwrap()[child_index]
                    .applet
                    .get_window(container_size)
            }
        }
    }
}
impl StandardApplet for RecursiveNodeApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        // TODO: change the order of the matches and stuff; currently just closest to previous implementation
        match standard_event {
            StandardEvent::UIEvent(UIEvent::WindowResized(size)) => {
                // every applet in this node is displayed at this size, not just the focused one
                self.shared_resource.latest_size.set(Some(size));
                self.main_applet
                    .immut_handle_ui_event(UIEvent::WindowResized(size));
                for child in self.shared_resource.children.read().unwrap().iter() {
                    child
                        .applet
                        .immut_handle_ui_event(UIEvent::WindowResized(size));
                }
            }
            StandardEvent::UIEvent(ui_event) => {
                match self.shared_resource.focus_index.get() {
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
                                    self.shared_resource.change_focus(operation);
                                } else {
                                    // if key press not a traversal, then just set focus to inner and forward input
                                    self.shared_resource
                                        .change_focus(WorldTreeTraversalOperation::NextLayer);
                                    self.handle_ui_event(ui_event);
                                }
                            }
                            UIEvent::Key { pressed: false, .. } => {}
                            UIEvent::Mouse(MouseEvent {
                                kind: MouseEventKind::Button { pressed: true, .. },
                                ..
                            }) => {
                                // set focus to inner and forward mouse press
                                self.shared_resource
                                    .change_focus(WorldTreeTraversalOperation::NextLayer);
                                self.handle_ui_event(ui_event);
                            }
                            UIEvent::WindowResized(_) | UIEvent::Mouse(_) => {
                                // forward without changing focus (otherwise hovering would steal focus)
                                self.main_applet.handle_ui_event(ui_event);
                            }
                        }

                        // match key.to_char() {
                        //     Some('q') => {
                        //         self.shared_resource
                        //             .hook
                        //             .change_focus(WorldTreeTraversalOperation::PrevLayer);
                        //         self.shared_resource.hook.damage_treeview();
                        //     }
                        //     Some('e') => {
                        //         self.shared_resource.focus_index.set(FocusIndex::Inner);
                        //         self.shared_resource.hook.damage_treeview();
                        //     }
                        //     Some('a') => {
                        //         self
                        //     }
                        //     Some('s') => {
                        //         self.shared_resource.hook.change_focus(
                        //             WorldTreeTraversalOperation::Layerwise(
                        //                 TreeTraverseOperation::RelShiftSibling(1),
                        //             ),
                        //         );
                        //         self.shared_resource.hook.damage_treeview();
                        //     }
                        //     Some('d') => {
                        //         if !self.shared_resource.children.read().unwrap().is_empty() {
                        //             self.shared_resource.focus_index.set(FocusIndex::Child(0));
                        //             self.shared_resource.hook.damage_treeview();
                        //         }
                        //     }
                        //     Some('0'..='9') => {
                        //         if (key.to_digit().unwrap() as usize)
                        //             < self.shared_resource.children.read().unwrap().len()
                        //         {
                        //             self.shared_resource
                        //                 .focus_index
                        //                 .set(FocusIndex::Child(key.to_digit().unwrap() as usize));
                        //             self.shared_resource.hook.damage_treeview();
                        //         }
                        //     }
                        //     _ => {}
                        // }
                    }
                    FocusIndex::Inner => self.main_applet.immut_handle_ui_event(ui_event),
                    FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                        [child_index]
                        .applet
                        .immut_handle_ui_event(ui_event),
                }
            }
            StandardEvent::FocusWlSurface(surface_id) => {
                let new_focus = if self.main_applet.contains_wl_surface(surface_id) {
                    FocusIndex::Inner
                } else if let Some(child_index) = self
                    .shared_resource
                    .children
                    .read()
                    .unwrap()
                    .iter()
                    .position(|child| child.applet.contains_wl_surface(surface_id))
                {
                    FocusIndex::Child(child_index)
                } else {
                    return;
                };
                self.shared_resource.focus_index.set(new_focus);

                // the focused applet focuses further down
                match new_focus {
                    FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                        [child_index]
                        .applet
                        .immut_handle_standard_event(standard_event),
                    _ => self.main_applet.immut_handle_standard_event(standard_event),
                }

                self.shared_resource.hook.damage_window();
                self.shared_resource.hook.damage_treeview();
            }
            StandardEvent::WlSurfaceDestroyed(surface_id) => {
                // forward to whichever applet contains it, without changing focus
                if self.main_applet.contains_wl_surface(surface_id) {
                    self.main_applet.immut_handle_standard_event(standard_event);
                } else if let Some(child) = self
                    .shared_resource
                    .children
                    .read()
                    .unwrap()
                    .iter()
                    .find(|child| child.applet.contains_wl_surface(surface_id))
                {
                    child.applet.immut_handle_standard_event(standard_event);
                }
            }
            StandardEvent::WlSurfaceRegistered {
                surface_id,
                wl_event_queue,
            } => {
                // TODO: change behavior based on `match self.shared_resource.focus_index.get()`
                SharedResource::add_child(
                    &self.shared_resource,
                    RecursiveNodeApplet::boxed_get_boxed_initializer(
                        WlSurfaceApplet::get_boxed_initiator((surface_id, wl_event_queue)),
                    ),
                );
            }
            StandardEvent::FocusChanged(..)
            | StandardEvent::Highlighted(..)
            | StandardEvent::CloseRequest
            | StandardEvent::SurfaceDamageAck
            | StandardEvent::TreeviewDamageAck => {
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
                    // REVIEW: should focusing just auto go to inner?
                    FocusIndex::Focusing | FocusIndex::Inner => {
                        self.main_applet.immut_handle_standard_event(standard_event)
                    }
                    FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                        [child_index]
                        .applet
                        .immut_handle_standard_event(standard_event),
                }
            }
        }

        SharedResource::remove_closed_children(&self.shared_resource);
    }

    fn get_window_standard_applet(&self, container_size: DisplayContainerSize) -> UITree {
        // TODO: return cached if damaged is already false?
        // REVIEW: idk why this is necessary if I already have caching browser; I'm not touching it rn
        self.shared_resource
            .window_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        self.get_display(container_size)
    }

    fn get_treeview(&self) -> WorldTree<String> {
        self.shared_resource
            .treeview_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut raw_treeview = RecursiveTreeNode::from_value(self.main_applet.get_treeview());

        for child_index in 0..(self.shared_resource.children.read().unwrap().len()) {
            let child_treeview = self.shared_resource.children.read().unwrap()[child_index]
                .applet
                .get_treeview();

            match child_treeview {
                WorldTree::World(recursive_tree_node) => {
                    raw_treeview.push_child_node(*recursive_tree_node);
                }
                _ => {
                    // REVIEW: should this ever even happen?
                    raw_treeview.push_child_node(RecursiveTreeNode::from_value(child_treeview));
                }
            }

            // raw_treeview.push_child_node(child_treeview);
        }

        WorldTree::World(Box::new(raw_treeview))
    }

    fn contains_wl_surface(&self, surface_id: WlSurfaceId) -> bool {
        self.main_applet.contains_wl_surface(surface_id)
            || self
                .shared_resource
                .children
                .read()
                .unwrap()
                .iter()
                .any(|child| child.applet.contains_wl_surface(surface_id))
    }

    /// `None` while this node itself is focused (`Focusing`), since it takes the keys then
    fn focused_wl_surface(&self) -> Option<WlSurfaceId> {
        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing => None,
            FocusIndex::Inner => self.main_applet.focused_wl_surface(),
            FocusIndex::Child(child_index) => self.shared_resource.children.read().unwrap()
                [child_index]
                .applet
                .focused_wl_surface(),
        }
    }

    fn get_focus_path(&self) -> WorldTreePath {
        // log::debug!(
        //     "My title is {} and my focus is {:?}",
        //     self.get_treeview().get_root_value(),
        //     self.shared_resource.focus_index.get(),
        // );

        match self.shared_resource.focus_index.get() {
            FocusIndex::Focusing => WorldTreePath::new_into(),
            FocusIndex::Inner => {
                let mut path_tail = self.main_applet.get_focus_path().0.to_vec();

                path_tail.insert(0, TreeNodePath::new_root());

                // println!("Focused: {}", focus_index);
                // println!("Path: {:?}", path_tail);

                WorldTreePath(path_tail.into_boxed_slice())
            }
            FocusIndex::Child(child_index) => {
                let mut path_tail = self.shared_resource.children.read().unwrap()[child_index]
                    .applet
                    .get_focus_path()
                    .0
                    .to_vec();

                if path_tail.is_empty() {
                    log::warn!("Warning: this shouldn't happen");
                    path_tail.push(TreeNodePath::new_root());
                }

                // child of index `focus - 1` is focused
                path_tail[0].0.insert(0, child_index);

                // println!("Focused: {}", focus_index);
                // println!("Path: {:?}", path_tail);

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
