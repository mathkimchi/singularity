use crate::client_handle::ClientHandle;
use calloop::{EventLoop, LoopHandle};
use singularity_common::sap::{
    packets::{StandardEvent, StandardRequest},
    raw_client_initializer::RawClientInitializer,
};
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{
    UIDisplay, display_units::DisplayContainerSize, ui_element::UIElement, ui_event::UIEvent,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};

// /// Wrap this around an Arc
// struct UIConnections {
//     root_ui_element: Mutex<UIElement>,
//     ui_event_queue: Mutex<Vec<UIEvent>>,
//     is_running: AtomicBool,
// }

// /// Because of the "edge" model (look at devlog sometime before 2026-07-26),
// /// we have main runner loop share a state with UI and with root applet,
// /// but they should be different locks (ie: UI and root applet can not lock each other)
// /// so, for redundant information like whether the app runs or not,
// /// I will duplicate that information and let the middleman keep them synchronized with each other
// enum RootAppletState {
//     Running { root_window_damaged: bool },
//     Ended,
// }

/// For the runner/main app being UI'd to hold
///
/// Instead of storing the event receiver in here,
/// it is registered in the calloop.
struct UIHandle {
    is_running: Arc<AtomicBool>,
    ui_content: EncapsulatedLock<UIElement>,
}
impl UIHandle {
    pub fn init_ui(
        initial_content: UIElement,
        runner_event_loop: &LoopHandle<'_, AppletRunner>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let ui_content = EncapsulatedLock::new(initial_content);
        let (tx, rx) = calloop::channel::channel();

        // I don't think order matters,
        // but just in-case I should start listening before I make the display
        runner_event_loop
            .insert_source(rx, |event, &mut (), runner| {
                let calloop::channel::Event::Msg(event) = event else {
                    // Means the UI closed; haven't thought abt what to do in this case
                    panic!()
                };
                runner.handle_ui_event(event);
            })
            .unwrap();

        {
            let is_running = is_running.clone();
            let ui_content = ui_content.clone();
            thread::spawn(|| {
                UIDisplay::run_display(is_running, tx, ui_content);
            });
        }

        Self {
            is_running,
            ui_content,
        }
    }

    pub fn set_ui_content(&self, new_content: UIElement) {
        self.ui_content.set(new_content);
    }
}

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner {
    event_loop: LoopHandle<'static, Self>,

    ui_handle: UIHandle,
    root_client: ClientHandle,

    window_size: DisplayContainerSize,
}
impl AppletRunner {
    fn new(
        event_loop: LoopHandle<'static, Self>,
        root_applet_initializer: Box<dyn RawClientInitializer>,
    ) -> Self {
        let root_client = ClientHandle::spawn_new_client(
            root_applet_initializer,
            UIElement::Nothing,
            &event_loop,
        );

        Self {
            ui_handle: UIHandle::init_ui(UIElement::Nothing, &event_loop),
            event_loop,
            root_client,
            // idk if there's a way to actually get this
            window_size: DisplayContainerSize {
                width: 100,
                height: 100,
            },
        }
    }

    /// Blocks until end.
    pub fn run(root_applet_initializer: Box<dyn RawClientInitializer>) {
        let mut event_loop = EventLoop::try_new().unwrap();

        let mut runner = Self::new(event_loop.handle(), root_applet_initializer);

        event_loop
            .run(None, &mut runner, |_| {
                // I think this is run between events, but I don't need this rn
            })
            .unwrap();

        // let self_sync_node = SyncNode::default();
        // let ui_sync_node = SyncNode::default();
        // let root_applet_sync_node = SyncNode::default();
        // // first is for self, second is for ui
        // let [self_ui_shared_data, ui_loop_ui_shared_data] = SyncEdge::new(
        //     UIState::Running {
        //         root_element: UIElement::Nothing,
        //         ui_event_queue: VecDeque::new(),
        //     },
        //     [&self_sync_node, &ui_sync_node],
        // );
        // let [
        //     self_root_applet_shared_data,
        //     root_applet_shared_data_for_applet,
        // ] = SyncEdge::new(
        //     RootAppletState::Running {
        //         root_window_damaged: false,
        //     },
        //     [&self_sync_node, &root_applet_sync_node],
        // );
        // // let root_window = Arc::new(Mutex::new(UIElement::Nothing));
        // // let root_window_damaged = Arc::new(AtomicBool::new(true));
        // // let (ui_event_queue_tx, ui_event_queue_rx) = mpsc::channel();
        // // let is_running = Arc::new(AtomicBool::new(true));

        // std::thread::spawn(move || {
        //     UIDisplay::run_display(ui_sync_node, ui_loop_ui_shared_data);
        // });
        // // {
        // //     // Debug:
        // //     self_sync_node.lock();
        // //     dbg!(self_ui_shared_data.try_lock().is_some());
        // // }

        // let applet = {
        //     // anon implementation
        //     struct AppletRunnerHook {
        //         // // UI and root applet should NOT have any direct locks shared
        //         // ui_shared_data: SharedData<UIState>,
        //         sync_node: SyncNode,
        //         applet_shared_data: SyncEdge<RootAppletState>,
        //         // root_window_damaged: Arc<AtomicBool>,
        //     }
        //     impl BasicRunnerHook for AppletRunnerHook {
        //         // fn update_display(&self, display: &UIElement) {
        //         //     // TODO: send reminder as well

        //         //     *self.root_ui_element.lock().unwrap() = display.clone();
        //         // }

        //         fn damage_window(&self) {
        //             // self.root_window_damaged
        //             //     .store(true, std::sync::atomic::Ordering::Relaxed);
        //             if let RootAppletState::Running {
        //                 root_window_damaged,
        //             } = &mut *self.applet_shared_data.wait_lock(self.sync_node.lock())
        //             {
        //                 *root_window_damaged = true;
        //             }
        //             // if it was ended, we should leave it like that
        //         }

        //         fn close(&self) {
        //             dbg!("Closing");
        //             // FIXME: Culprit: this is being called in main loop and mainloop is locking lock_state
        //             *self.applet_shared_data.wait_lock(self.sync_node.lock()) =
        //                 RootAppletState::Ended;
        //             // self.applet_shared_data.notify();
        //         }
        //     }

        //     applet_initizer(Box::new(AppletRunnerHook {
        //         // root_window_damaged: root_window_damaged.clone(),
        //         // ui_shared_data: ui_shared_data.clone(),
        //         sync_node: root_applet_sync_node,
        //         applet_shared_data: root_applet_shared_data_for_applet,
        //     }))
        // };

        // let mut runner = Self {
        //     applet,
        //     // root_window_damaged,
        //     // REVIEW
        //     window_size: DisplayContainerSize::new(0, 0),
        //     sync_node: self_sync_node,
        //     ui_shared_data: self_ui_shared_data,
        //     root_applet_shared_data: self_root_applet_shared_data,
        // };

        // // Don't let any updates happen unless we are waiting
        // let mut node_lock = runner.sync_node.lock();

        // // {
        // //     // Debug:
        // //     dbg!(runner.ui_shared_data.try_lock().is_some());
        // // }

        // for _ in 0..10 {
        //     dbg!("waiting");
        //     // dbg!(runner.ui_shared_data.try_lock().is_some());
        //     node_lock.wait_for_update();
        //     // dbg!(runner.ui_shared_data.try_lock().is_some());
        //     dbg!("update notified");

        //     let Some(mut ui_shared_data) = runner.ui_shared_data.try_lock() else {
        //         // we just start over without marking that we processed updates
        //         dbg!("Failed to get ui shared data");
        //         node_lock.wait_for_notif();
        //         continue;
        //     };
        //     dbg!("Succeed locking ui shared data");
        //     match &mut *ui_shared_data {
        //         UIState::Running { ui_event_queue, .. } => {
        //             dbg!("point 1");
        //             for ui_event in std::mem::take(ui_event_queue) {
        //                 if let UIEvent::WindowResized(window_size) = ui_event {
        //                     runner.window_size = window_size;
        //                 }

        //                 dbg!("point 2");

        //                 thread::spawn(move || runner.applet.handle_ui_event(ui_event));
        //             }
        //             dbg!("point 3");
        //         }
        //         UIState::Ended => {
        //             // let the applet know we're done; do this later bc it is potentially deadlock
        //             // *runner.root_applet_shared_data.wait_lock(node_lock) = RootAppletState::Ended;
        //             break;
        //         }
        //     }

        //     let Some(mut root_applet_guard) = runner.root_applet_shared_data.try_lock() else {
        //         dbg!("Failed to get root applet");
        //         node_lock.wait_for_notif();
        //         continue;
        //     };
        //     dbg!("Succeed to get root applet");
        //     match &*root_applet_guard {
        //         RootAppletState::Running {
        //             root_window_damaged: true,
        //         } => {
        //             *root_applet_guard = RootAppletState::Running {
        //                 root_window_damaged: false,
        //             };
        //             // we don't need to notify root applet bc main loop is the only one who waits for this

        //             match &mut *ui_shared_data {
        //                 UIState::Running { root_element, .. } => {
        //                     *root_element = runner.applet.get_window(runner.window_size);
        //                 }
        //                 UIState::Ended => {
        //                     *root_applet_guard = RootAppletState::Ended;
        //                 }
        //             }
        //         }
        //         RootAppletState::Running {
        //             root_window_damaged: false,
        //         } => {
        //             // no updates from root applet, don't need to do anything
        //         }
        //         RootAppletState::Ended => {
        //             // let the ui know we're done
        //             *ui_shared_data = UIState::Ended;
        //             break;
        //         }
        //     }

        //     // I just realized if there's a lot of edges and one of them always happens to be locked before we can get through this,
        //     // then we might process everything but never mark updates as processed.
        //     node_lock.mark_updates_processed();
        // }

        // dbg!("mainloop done!");
    }

    fn handle_ui_event(&self, event: UIEvent) {
        self.root_client.send_event(StandardEvent::UIEvent(event));
    }

    /// TODO: take in client id
    pub(crate) fn handle_client_request(&mut self, request: StandardRequest) {
        match request {
            StandardRequest::DamageSurface => {
                self.ui_handle
                    .set_ui_content(self.root_client.get_surface());
            }
            StandardRequest::DamageTreeview => todo!(),
        }
    }
}
