use crate::applet::{BasicApplet, BasicRunnerHook};
use sonamu_sync::shared_state::SharedData;
use sonamu_ui::{
    UIDisplay, display_units::DisplayContainerSize, ui_element::UIElement, ui_event::UIEvent,
    winit_backend::UIState,
};
use std::collections::VecDeque;

/// Because of the "edge" model (look at devlog sometime before 2026-07-26),
/// we have main runner loop share a state with UI and with root applet,
/// but they should be different locks (ie: UI and root applet can not lock each other)
/// so, for redundant information like whether the app runs or not,
/// I will duplicate that information and let the middleman keep them synchronized with each other
pub enum RootAppletState {
    Running { root_window_damaged: bool },
    Ended,
}

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner<Applet: BasicApplet> {
    applet: Applet,

    ui_shared_data: SharedData<UIState>,
    root_applet_shared_data: SharedData<RootAppletState>,

    window_size: DisplayContainerSize,
}
impl<Applet: BasicApplet> AppletRunner<Applet> {
    /// Returns after the applet is closed.
    ///
    /// The logic of this is similar to `UIDisplay::run_display` in `winit_backend`
    pub fn run(applet_initizer: impl FnOnce(Box<dyn BasicRunnerHook>) -> Applet) {
        let ui_shared_data = SharedData::new(UIState::Running {
            root_element: UIElement::Nothing,
            ui_event_queue: VecDeque::new(),
        });
        let root_applet_shared_data = SharedData::new_with_same_cond(
            RootAppletState::Running {
                root_window_damaged: false,
            },
            &ui_shared_data,
        );

        {
            // clone to satisfy compiler
            let ui_shared_data = ui_shared_data.clone();

            std::thread::spawn(move || {
                UIDisplay::run_display(ui_shared_data);
            });
        }

        let applet = {
            // anon implementation
            struct AppletRunnerHook {
                // NOTE: UI and root applet should NOT have any direct locks shared
                applet_shared_data: SharedData<RootAppletState>,
            }
            impl BasicRunnerHook for AppletRunnerHook {
                fn damage_window(&self) {
                    if let RootAppletState::Running {
                        root_window_damaged,
                    } = &mut *self.applet_shared_data.lock_state()
                    {
                        *root_window_damaged = true;
                    }
                    // if it was ended, we should leave it like that
                }

                fn close(&self) {
                    log::debug!("Closing");
                    // FIXME: Culprit: this is being called in main loop and mainloop is locking lock_state
                    *self.applet_shared_data.lock_state() = RootAppletState::Ended;
                    self.applet_shared_data.notify();
                }
            }

            applet_initizer(Box::new(AppletRunnerHook {
                applet_shared_data: root_applet_shared_data.clone(),
            }))
        };

        let mut runner = Self {
            applet,
            // REVIEW
            window_size: DisplayContainerSize::new(0, 0),
            ui_shared_data,
            root_applet_shared_data,
        };

        let mut ui_shared_state_guard = runner.ui_shared_data.lock_state();

        while let UIState::Running {
            root_element,
            ui_event_queue,
        } = &mut *ui_shared_state_guard
        {
            for ui_event in std::mem::take(ui_event_queue) {
                if let UIEvent::WindowResized(window_size) = ui_event {
                    runner.window_size = window_size;
                }

                runner.applet.handle_ui_event(ui_event);
            }

            let mut root_applet_guard = runner.root_applet_shared_data.lock_state();
            match &*root_applet_guard {
                RootAppletState::Running {
                    root_window_damaged: true,
                } => {
                    *root_applet_guard = RootAppletState::Running {
                        root_window_damaged: false,
                    };
                    // we don't need to notify root applet bc main loop is the only one who waits for this

                    drop(root_applet_guard); // avoid deadlock

                    *root_element = runner.applet.get_window(runner.window_size);
                    runner.ui_shared_data.notify();
                }
                RootAppletState::Running {
                    root_window_damaged: false,
                } => {
                    // no updates from root applet, don't need to do anything
                }
                RootAppletState::Ended => {
                    *ui_shared_state_guard = UIState::Ended;
                    runner.ui_shared_data.notify();
                    break;
                }
            }

            log::debug!("Main loop waiting");
            ui_shared_state_guard.wait_for_update();
        }

        *runner.root_applet_shared_data.lock_state() = RootAppletState::Ended;
        runner.root_applet_shared_data.notify();
        log::debug!("mainloop done!");
    }
}
