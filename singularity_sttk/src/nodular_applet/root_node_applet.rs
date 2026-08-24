use crate::{
    basic_applet::{BasicApplet, BasicRunnerHook},
    nodular_applet::{
        AppletSpawner, NodularRunnerHook, StandardApplet,
        recursive_node_applet::RecursiveNodeApplet,
    },
};
use calloop::{
    EventLoop, LoopHandle,
    channel::{Channel, Sender, channel},
};
use singularity_common::{
    sap::{
        packets::{StandardEvent, StandardRequest},
        raw_client_initializer::RawClientInitializer,
    },
    sync::EncapsulatedLock,
    utils::tree::world_tree::WorldTreePath,
};
use sonamu_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
    ui_element::UIElement,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

/// Holds the recursive_node_applet, is held by a Basic Applet runner (applet runner).
pub struct RootNodeApplet {
    /// NOTE: this could be generic
    applet: RecursiveNodeApplet,
    // inner_event_queue: Sender<NodularEvent>,
    // window: Arc<Mutex<UIElement>>,
    // hook: Arc<Mutex<Box<dyn BasicRunnerHook>>>,
    // applet_spawner_registry: RwLock<BTreeMap<String, AppletSpawner>>,
    content: EncapsulatedLock<UIElement>,
    content_dirty: bool,
    request_sender: Sender<StandardRequest>,

    latest_size: DisplayContainerSize,

    applet_spawner_registry: Arc<RwLock<BTreeMap<String, AppletSpawner>>>,
}
impl RootNodeApplet {
    fn inner_applet_size(&self) -> DisplayContainerSize {
        self.latest_size
            .find_subsize(DisplayArea::new((0.2, 0.0), (1.0, 1.0)).size())
    }

    /// Redraws if necessary/possible
    fn try_redraw(&mut self) {
        if !self.content_dirty {
            self.content.set(self.get_window(self.latest_size));
            self.request_sender
                .send(StandardRequest::DamageSurface)
                .unwrap();
            self.content_dirty = true;
        }
    }

    #[must_use]
    pub fn new(
        inner_initializer: Box<dyn FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet>,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        content: EncapsulatedLock<UIElement>,
        request_sender: Sender<StandardRequest>,
        event_loop: &LoopHandle<'_, Self>,
    ) -> Self {
        // let (inner_event_queue_tx, inner_event_queue_rx) = channel();
        let (inner_request_queue_tx, inner_request_queue_rx) = channel();

        event_loop
            .insert_source(inner_request_queue_rx, |event, &mut (), applet| {
                let calloop::channel::Event::Msg(event) = event else {
                    return;
                };

                match event {
                    StandardRequest::DamageSurface => {
                        applet.try_redraw();
                    }
                    StandardRequest::DamageTreeview => {
                        applet.try_redraw();
                    }
                    StandardRequest::Quit => todo!(),
                }
            })
            .unwrap();

        struct InnerHook {
            outer_request_queue: Sender<StandardRequest>,
            applet_spawner_registry: Arc<RwLock<BTreeMap<String, AppletSpawner>>>,
        }
        impl BasicRunnerHook for InnerHook {
            fn close(&self) {
                self.outer_request_queue
                    .send(StandardRequest::Quit)
                    .unwrap();
                // self.outer_hook.close();
            }

            fn damage_window(&self) {
                // REVIEW
                // self.outer_hook.damage_window();
                self.outer_request_queue
                    .send(StandardRequest::DamageSurface)
                    .unwrap();
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, _treeview: &sonamu_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: super::NodularAppletInitializer) {
                unimplemented!()
            }

            fn damage_treeview(&self) {
                // REVIEW
                // self.outer_hook.damage_window();

                self.outer_request_queue
                    .send(StandardRequest::DamageSurface)
                    .unwrap();
            }

            fn change_focus(
                &self,
                _operation: singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation,
            ) {
                // REVIEW: do I need to do anything here?
            }

            fn register_applet_spawner(&self, name: String, applet_spawner: AppletSpawner) {
                self.applet_spawner_registry
                    .write()
                    .unwrap()
                    .insert(name, applet_spawner);
            }
            fn get_applet_spawners(&self) -> BTreeMap<String, AppletSpawner> {
                self.applet_spawner_registry.read().unwrap().clone()
            }
            fn find_applet_spawner(&self, name: String) -> Option<AppletSpawner> {
                self.applet_spawner_registry
                    .read()
                    .unwrap()
                    .get(&name)
                    .cloned()
            }
        }

        let applet_spawner_registry = Arc::new(RwLock::new(applet_spawner_registry));
        let inner_hook = InnerHook {
            outer_request_queue: inner_request_queue_tx,
            applet_spawner_registry: applet_spawner_registry.clone(),
        };

        Self {
            applet: inner_initializer(Box::new(inner_hook)),
            // inner_event_queue: inner_event_queue_tx,
            content,
            content_dirty: false,
            // Just making stuff up
            latest_size: DisplayContainerSize::new(800, 600),
            request_sender,
            applet_spawner_registry,
        }
    }

    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet
        + Send
        + Sync
        + 'static,
        applet_spawner_registry: BTreeMap<String, AppletSpawner>,
    ) -> Box<dyn RawClientInitializer> {
        struct Initializer {
            inner_initializer:
                Box<dyn FnOnce(Box<dyn NodularRunnerHook>) -> RecursiveNodeApplet + Send + Sync>,
            applet_spawner_registry: BTreeMap<String, AppletSpawner>,
        }
        impl RawClientInitializer for Initializer {
            fn init(
                // smth smth box needs to know size
                self: Box<Self>,
                content: EncapsulatedLock<UIElement>,
                event_queue: Channel<StandardEvent>,
                // yeah, ik the naming is inconsistent bc I'm not saying "event_receiver" or "event_rx", but it's calm
                // (I am really trying to convince myself this is fine, I am the strawman)
                request_sender: Sender<StandardRequest>,
            ) {
                let mut event_loop = EventLoop::try_new().unwrap();

                event_loop
                    .handle()
                    .insert_source(
                        event_queue,
                        |event, &mut (), applet: &mut RootNodeApplet| {
                            let calloop::channel::Event::Msg(event) = event else {
                                return;
                            };
                            match event {
                                StandardEvent::UIEvent(
                                    sonamu_ui::ui_event::UIEvent::WindowResized(size),
                                ) => {
                                    applet.latest_size = size;
                                    applet.applet.handle_ui_event(
                                        sonamu_ui::ui_event::UIEvent::WindowResized(
                                            applet.inner_applet_size(),
                                        ),
                                    );
                                }
                                StandardEvent::UIEvent(ui_event) => {
                                    applet.applet.handle_ui_event(ui_event);
                                }
                                StandardEvent::CloseRequest => todo!(),
                                StandardEvent::SurfaceDamageAck => {
                                    applet.content_dirty = false;
                                }
                                StandardEvent::FocusChanged(..)
                                | StandardEvent::Highlighted(..)
                                | StandardEvent::TreeviewDamageAck
                                | StandardEvent::WlSurfaceRegistered { .. } => {
                                    applet.applet.handle_standard_event(event);
                                }
                            }
                        },
                    )
                    .unwrap();

                let mut applet = RootNodeApplet::new(
                    self.inner_initializer,
                    self.applet_spawner_registry,
                    content,
                    request_sender,
                    &event_loop.handle(),
                );

                event_loop.run(None, &mut applet, |_| {}).unwrap();
            }
        }
        Box::new(Initializer {
            inner_initializer: Box::new(inner_initializer),
            applet_spawner_registry,
        })
    }

    fn get_treeview_display(&self) -> UIElement {
        let focused_path = self.applet.get_focus_path();
        let treeview = self.applet.get_treeview();

        println!("Focused path: {focused_path:?}");

        // CharGrid::from(treeview.outer_world_to_string())
        //     .element()
        //     .bordered(Color::LIGHT_GREEN)
        //     .fill_bg(Color::BLACK)

        // This is the horizontal split
        // UIElement::combine_displays((0..(focused_path.0.len() + 1)).map(|world_level_index| {
        UIElement::combine_displays((0..focused_path.0.len()).map(|world_level_index| {
            let world_path = WorldTreePath(
                focused_path.0[0..world_level_index]
                    .to_vec()
                    .into_boxed_slice(),
            );
            UIElement::from(
                treeview
                    .safe_get(&world_path)
                    .unwrap()
                    .outer_world_to_string(focused_path.0.get(world_level_index).cloned()),
            )
            .bordered(Color::LIGHT_GREEN)
        }))
        .fill_bg(Color::BLACK)
    }
}
impl BasicApplet for RootNodeApplet {
    fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
        self.applet.handle_ui_event(ui_event);
    }

    fn get_window(&self, container_size: DisplayContainerSize) -> UIElement {
        UIElement::Container(vec![
            self.get_treeview_display()
                .contain(DisplayArea::new((0.0, 0.0), (0.2, 1.0))),
            self.applet
                .layout_builder()
                .bordered(Color::LIGHT_GREEN)
                .contained(DisplayArea::new((0.2, 0.0), (1.0, 1.0)))
                .get_ui_element(container_size),
        ])
    }
}

/*

/// Holds a nodular applet but only supports Basic operations.
/// NOTE: This is really for debugging; for the actual, I'll implement RootNodeApplet.
pub struct NodularHolderApplet<InnerApplet: NodularApplet> {
    inner_applet: InnerApplet,
}
impl<InnerApplet: NodularApplet> NodularHolderApplet<InnerApplet> {
    pub fn new(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> InnerApplet,
        hook: Box<dyn BasicRunnerHook>,
    ) -> Self {
        struct InnerHook {
            outer_hook: Box<dyn BasicRunnerHook>,
        }
        impl BasicRunnerHook for InnerHook {
            // fn update_display(&self, display: &sonamu_ui::ui_element::UIElement) {
            //     self.outer_hook.update_display(display);
            // }

            fn close(&self) {
                self.outer_hook.close();
            }

            fn damage_window(&self) {
                self.outer_hook.damage_window();
            }
        }
        impl NodularRunnerHook for InnerHook {
            // fn update_treeview(&self, _treeview: &sonamu_ui::ui_element::UIElement) {}

            fn add_child(&self, _initializer: Box<super::NodularAppletInitializer>) {}

            fn damage_treeview(&self) {
                todo!()
            }

            fn change_focus(
                &self,
                _operation: singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation,
            ) {
            }
        }

        let inner_hook = InnerHook { outer_hook: hook };

        Self {
            inner_applet: inner_initializer(Box::new(inner_hook)),
        }
    }

    pub fn get_initializer(
        inner_initializer: impl FnOnce(Box<dyn NodularRunnerHook>) -> InnerApplet,
    ) -> impl FnOnce(Box<dyn BasicRunnerHook>) -> Self {
        move |hook| Self::new(inner_initializer, hook)
    }
}
impl<InnerApplet: NodularApplet> BasicApplet for NodularHolderApplet<InnerApplet> {
    fn handle_ui_event(&mut self, ui_event: sonamu_ui::ui_event::UIEvent) {
        self.inner_applet.handle_ui_event(ui_event);
    }

    fn get_window(&self) -> UIElement {
        self.inner_applet.get_window()
    }
}

*/
