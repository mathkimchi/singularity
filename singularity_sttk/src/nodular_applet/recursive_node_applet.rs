use crate::{
    basic_applet::{BasicApplet, BasicRunnerHook},
    nodular_applet::{
        NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
        caching_applet::CachingApplet,
    },
};
use singularity_common::{
    sync::EncapsulatedLock,
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TreeNodePath, TreeTraverseOperation},
        world_tree::{WorldTree, WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation},
    },
};
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
        // breaks when adding the first child
        // shared_resource.upgrade().unwrap().applets.read().unwrap()[shared_resource
        //     .upgrade()
        //     .unwrap()
        //     .focus_index
        //     .load(std::sync::atomic::Ordering::Relaxed)]
        // .immut_handle_nodular_event(NodularEvent::Focused(false));

        let child_holder = {
            let inner_applet_window = EncapsulatedLock::new(UIElement::Nothing);
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
                // the index of this hook's corresponding app in the shared resource list of applets
                // index: usize,
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
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
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
                // index: shared_resource.children.read().unwrap().len(),
                shared_resource: Arc::downgrade(shared_resource),
                // treeview: inner_applet_treeview.clone(),
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
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
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

    // fn get_focused_applet(&self) -> Arc<&SubAppletHolder> {
    //     match self.shared_resource.focus_index.get() {
    //         FocusIndex::Focusing | FocusIndex::Inner => Arc::new(&self.main_applet),
    //         FocusIndex::Child(child_index) => {
    //             &self.shared_resource.children.read().unwrap()[child_index]
    //         }
    //     }
    // }

    fn get_display(&self, container_size: DisplayContainerSize) -> UIElement {
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

            // raw_treeview.push_child_node(child_treeview);
        }

        WorldTree::World(Box::new(raw_treeview))
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
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
