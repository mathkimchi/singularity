use crate::nodular_applet::{
    NodularApplet, NodularAppletInitializer, NodularEvent, NodularRunnerHook,
    applet_holder::SubAppletHolder,
};
use singularity_common::sync::EncapsulatedLock;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayUnits},
    ui_element::UIElement,
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};
use std::sync::{Arc, RwLock, Weak, atomic::AtomicUsize};

/// The main divided applet holds an Arc to this and applets hold Weak to this.
/// REVIEW: rename
struct MultiAppletHolder {
    applets: RwLock<Vec<Arc<SubAppletHolder>>>,
    focus_index: AtomicUsize,
    hook: Box<dyn NodularRunnerHook>,
}
impl MultiAppletHolder {
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

    fn get_display(shared_resource: &Weak<Self>) -> UIElement {
        let mut applet_displays = Vec::new();

        for applet in shared_resource
            .upgrade()
            .unwrap()
            .applets
            .read()
            .unwrap()
            .iter()
        {
            applet_displays.push(applet.get_window());
        }

        Self::combine_displays(applet_displays)
    }

    fn add_child(
        shared_resource: Weak<Self>,
        child_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
    ) {
        let child_holder = {
            let inner_applet_window = EncapsulatedLock::new(UIElement::Nothing);
            let inner_applet_treeview = EncapsulatedLock::new(UIElement::Nothing);

            struct InnerHook {
                // outer_children: Arc<Mutex<Vec<SubAppletHolder>>>,
                // // outer_hook: Arc<Mutex<Box<dyn NodularRunnerHook>>>,
                // outer_hook: Arc<Box<dyn BasicRunnerHook>>,
                window: EncapsulatedLock<UIElement>,
                treeview: EncapsulatedLock<UIElement>,
                // outer_focused_child_index: Arc<Mutex<usize>>,
                shared_resource: Weak<MultiAppletHolder>,

                // the index of this hook's corresponding app in the shared resource list of applets
                index: usize,
            }
            impl BasicRunnerHook for InnerHook {
                fn update_display(&self, display: &UIElement) {
                    self.window.set(display.clone());

                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .update_display(&MultiAppletHolder::get_display(&self.shared_resource));
                }

                fn close(&self) {
                    // FIXME: right now, just closes the entire node including all children as well
                    self.shared_resource.upgrade().unwrap().hook.close();
                }
            }
            impl NodularRunnerHook for InnerHook {
                fn update_treeview(&self, treeview: &UIElement) {
                    self.treeview.set(treeview.clone());

                    self.shared_resource
                        .upgrade()
                        .unwrap()
                        .hook
                        .update_treeview(&MultiAppletHolder::get_display(&self.shared_resource));
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
                todo!(),
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

        shared_resource
            .upgrade()
            .unwrap()
            .hook
            .update_display(&Self::get_display(&shared_resource));
    }

    fn new(hook: Box<dyn NodularRunnerHook>) -> Self {
        let hook = hook;
        let applets = RwLock::new(Vec::new());
        let focus_index = AtomicUsize::new(0);

        Self {
            applets,
            focus_index,
            hook,
        }
    }
}

/// Has a list of inner applets and displays them in vertical or horizontal division.
pub struct DividedApplet {
    shared_resource: Arc<MultiAppletHolder>,
}
impl DividedApplet {
    fn new(
        inner_initiator: impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>,
        // hook: Box<dyn NodularRunnerHook>,
        hook: Box<dyn NodularRunnerHook>,
    ) -> Self {
        let hook = hook;
        let applets = RwLock::new(Vec::new());
        let focus_index = AtomicUsize::new(0);

        let s = Self {
            shared_resource: Arc::new(MultiAppletHolder {
                applets,
                focus_index,
                hook,
            }),
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
}
impl BasicApplet for DividedApplet {
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
            self.shared_resource
                .focus_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.shared_resource.focus_index.fetch_min(
                self.shared_resource.applets.read().unwrap().len() - 1,
                std::sync::atomic::Ordering::Relaxed,
            );
            return;
        }

        let applet_holder = self.shared_resource.applets.read().unwrap()[self
            .shared_resource
            .focus_index
            .load(std::sync::atomic::Ordering::Relaxed)]
        .clone();
        applet_holder.immut_handle_ui_event(ui_event);
    }
}
impl NodularApplet for DividedApplet {
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
}

// /// Holds the recursive_node_applet, is held by a Basic Applet runner (applet runner).
// pub struct RootNodeApplet {
//     applet: RecursiveNodeApplet,
//     window: Arc<Mutex<UIElement>>,

//     hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
// }
// impl RootNodeApplet {}
// impl BasicApplet for RootNodeApplet {
//     fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
//         ui_event;
//     }
// }
