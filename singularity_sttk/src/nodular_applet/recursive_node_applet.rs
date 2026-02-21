use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
    applet_holder::SubAppletHolder,
};
use singularity_common::{
    sync::EncapsulatedLock,
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::TreeNodePath,
        world_tree::{WorldTree, WorldTreePath},
    },
};
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayUnits},
    ui_element::UIElement,
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};
use std::sync::{
    Arc, RwLock, Weak,
    atomic::{AtomicBool, AtomicUsize},
};

/// The main divided applet holds an Arc to this and applets hold Weak to this.
/// REVIEW: rename
/// TODO: generalize
struct MultiAppletHolder {
    applets: RwLock<Vec<Arc<SubAppletHolder>>>,

    /// NOTE: I am kinda using semantic value of 0 for root is focused,
    /// even though Optional is more elegant in theory.
    /// TODO: migrate to Optional later
    focus_index: AtomicUsize,
    hook: Box<dyn NodularRunnerHook>,

    window_damaged: AtomicBool,
    treeview_damaged: AtomicBool,
}
impl MultiAppletHolder {
    fn add_child(
        shared_resource: Weak<Self>,
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
                window: EncapsulatedLock<UIElement>,
                treeview: EncapsulatedLock<WorldTree<String>>,
                // outer_focused_child_index: Arc<Mutex<usize>>,
                shared_resource: Weak<MultiAppletHolder>,

                // the index of this hook's corresponding app in the shared resource list of applets
                index: usize,
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

                fn add_child(&self, initializer: Box<NodularAppletInitializer>) {
                    MultiAppletHolder::add_child(self.shared_resource.clone(), initializer);
                }
            }

            let inner_hook = InnerHook {
                window: inner_applet_window.clone(),
                index: shared_resource
                    .upgrade()
                    .unwrap()
                    .applets
                    .read()
                    .unwrap()
                    .len(),
                shared_resource: shared_resource.clone(),
                treeview: inner_applet_treeview.clone(),
            };

            Arc::new(SubAppletHolder::new(
                child_initializer,
                Box::new(inner_hook),
                inner_applet_window,
                inner_applet_treeview,
            ))
        };

        shared_resource
            .upgrade()
            .unwrap()
            .applets
            .write()
            .unwrap()
            .push(child_holder);

        if !shared_resource
            .upgrade()
            .unwrap()
            .window_damaged
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            shared_resource.upgrade().unwrap().hook.damage_window();
        }
    }

    fn new(hook: Box<dyn NodularRunnerHook>) -> Self {
        let hook = hook;
        let applets = RwLock::new(Vec::new());
        let focus_index = AtomicUsize::new(0);
        let window_damaged = AtomicBool::new(true);
        let treeview_damaged = AtomicBool::new(true);

        Self {
            applets,
            focus_index,
            hook,
            window_damaged,
            treeview_damaged,
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

/// Holds a main Applet (at index 0) and also children.
/// Displays the main child
pub struct RecursiveNodeApplet {
    shared_resource: Arc<MultiAppletHolder>,
}
impl RecursiveNodeApplet {
    fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn NodularRunnerHook>,
    ) -> Self {
        let s = Self {
            shared_resource: Arc::new(MultiAppletHolder::new(hook)),
        };

        MultiAppletHolder::add_child(Arc::downgrade(&s.shared_resource), inner_initiator);

        s
    }

    /// Partial application
    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        move |hook: Box<dyn NodularRunnerHook>| Self::new(inner_initializer, hook)
    }

    /// Takes in a list of full-size elements and returns a combined ui element where they are equally spaced
    /// across the horizontal axis and take full height.
    fn combine_displays(subdisplays: Vec<UIElement>) -> UIElement {
        // proportional units so widths out of 1
        let widths = 1. / subdisplays.len() as f32;
        UIElement::Container(
            subdisplays
                .into_iter()
                .enumerate()
                .map(|(i, subdisplay)| {
                    subdisplay
                        .bordered(Color::LIGHT_GREEN)
                        .contain(DisplayArea::new(
                            (widths * (i as f32), 0.),
                            (DisplayUnits::from_mixed(-1, widths * ((i + 1) as f32)), 1.),
                        ))
                })
                .collect(),
        )
    }

    fn get_display(&self) -> UIElement {
        let mut applet_displays = Vec::new();

        for applet in self.shared_resource.applets.read().unwrap().iter() {
            applet_displays.push(applet.get_window());
        }

        Self::combine_displays(applet_displays)
    }
}
impl BasicApplet for RecursiveNodeApplet {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        if let UIEvent::KeyPress(
            key,
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: false,
                caps_lock: false,
                logo: false,
            },
        ) = &ui_event
            && key.to_char() == Some('\t')
        {
            self.shared_resource.applets.read().unwrap()[self
                .shared_resource
                .focus_index
                .load(std::sync::atomic::Ordering::Relaxed)]
            .immut_handle_nodular_event(NodularEvent::Focused(false));

            self.shared_resource
                .focus_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.shared_resource.focus_index.fetch_min(
                self.shared_resource.applets.read().unwrap().len() - 1,
                std::sync::atomic::Ordering::Relaxed,
            );

            self.shared_resource.applets.read().unwrap()[self
                .shared_resource
                .focus_index
                .load(std::sync::atomic::Ordering::Relaxed)]
            .immut_handle_nodular_event(NodularEvent::Focused(true));

            return;
        }

        if let UIEvent::KeyPress(
            key,
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: true,
                caps_lock: false,
                logo: false,
            },
        ) = &ui_event
            && key.raw_code == 15
        {
            // Ctrl+Shift+Tab
            self.shared_resource.applets.read().unwrap()[self
                .shared_resource
                .focus_index
                .load(std::sync::atomic::Ordering::Relaxed)]
            .immut_handle_nodular_event(NodularEvent::Focused(false));

            self.shared_resource
                .focus_index
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            self.shared_resource.focus_index.fetch_min(
                self.shared_resource.applets.read().unwrap().len() - 1,
                std::sync::atomic::Ordering::Relaxed,
            );

            self.shared_resource.applets.read().unwrap()[self
                .shared_resource
                .focus_index
                .load(std::sync::atomic::Ordering::Relaxed)]
            .immut_handle_nodular_event(NodularEvent::Focused(true));
            return;
        }

        let applet_holder = self.shared_resource.applets.read().unwrap()[self
            .shared_resource
            .focus_index
            .load(std::sync::atomic::Ordering::Relaxed)]
        .clone();
        applet_holder.immut_handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        // TODO: return cached if damaged is already false?
        self.shared_resource
            .window_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        self.get_display()
    }
}
impl NodularApplet for RecursiveNodeApplet {
    fn handle_nodular_event(&mut self, nodular_event: NodularEvent) {
        match nodular_event {
            NodularEvent::Highlighted(_) => todo!(),
            NodularEvent::Focused(state) => {
                self.shared_resource.applets.read().unwrap()[self
                    .shared_resource
                    .focus_index
                    .load(std::sync::atomic::Ordering::Relaxed)]
                .immut_handle_nodular_event(NodularEvent::Focused(state));
            }
        }
    }

    fn get_treeview(&self) -> WorldTree<String> {
        self.shared_resource
            .treeview_damaged
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut raw_treeview = RecursiveTreeNode::from_value(
            self.shared_resource.applets.read().unwrap()[0].get_treeview(),
        );

        for child_index in 0..(self.shared_resource.applets.read().unwrap().len() - 1) {
            let child_treeview =
                self.shared_resource.applets.read().unwrap()[child_index + 1].get_treeview();

            match child_treeview {
                WorldTree::World(recursive_tree_node) => {
                    raw_treeview.push_child_node(*recursive_tree_node);
                }
                _ => {
                    // REVIEW: should this ever even happen?
                    raw_treeview.push_child_node(RecursiveTreeNode::from_value(child_treeview))
                }
            }

            // raw_treeview.push_child_node(child_treeview);
        }

        WorldTree::World(Box::new(raw_treeview))
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        let focus_index = self
            .shared_resource
            .focus_index
            .load(std::sync::atomic::Ordering::Relaxed);

        let mut path_tail = self.shared_resource.applets.read().unwrap()[focus_index]
            .get_focus_path()
            .0
            .to_vec();

        if focus_index == 0 {
            // main is focused
            path_tail.insert(0, TreeNodePath::new_root());
        } else {
            // child of index `focus - 1` is focused
            path_tail[0].0.insert(0, focus_index - 1);
        }

        WorldTreePath(path_tail.into_boxed_slice())
    }
}
