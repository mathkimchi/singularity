use crate::applet::{BasicApplet, BasicRunnerHook};
use singularity_ui::{
    UIDisplay, display_units::DisplayContainerSize, ui_element::UIElement, ui_event::UIEvent,
};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

// /// Wrap this around an Arc
// struct UIConnections {
//     root_ui_element: Mutex<UIElement>,
//     ui_event_queue: Mutex<Vec<UIEvent>>,
//     is_running: AtomicBool,
// }

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner<Applet: BasicApplet> {
    applet: Applet,

    // fields for dealing with the UI
    root_window: Arc<Mutex<UIElement>>,
    root_window_damaged: Arc<AtomicBool>,

    /// TODO: use mpsc
    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
    is_running: Arc<AtomicBool>,

    window_size: DisplayContainerSize,
}
impl<Applet: BasicApplet> AppletRunner<Applet> {
    /// Returns after the applet is closed.
    ///
    /// The logic of this is similar to `UIDisplay::run_display` in `wayland_backend`
    pub fn run(applet_initizer: impl FnOnce(Box<dyn BasicRunnerHook>) -> Applet) {
        let root_window = Arc::new(Mutex::new(UIElement::Nothing));
        let root_window_damaged = Arc::new(AtomicBool::new(true));
        let ui_event_queue = Arc::new(Mutex::new(Vec::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        {
            // clone to satisfy compiler
            let root_window = root_window.clone();
            let ui_event_queue = ui_event_queue.clone();
            let is_running = is_running.clone();

            std::thread::spawn(move || {
                UIDisplay::run_display(root_window, ui_event_queue, is_running);
            });
        }

        let applet = {
            // anon implementation
            struct AppletRunnerHook {
                root_window_damaged: Arc<AtomicBool>,

                is_running: Arc<AtomicBool>,
            }
            impl BasicRunnerHook for AppletRunnerHook {
                // fn update_display(&self, display: &UIElement) {
                //     // TODO: send reminder as well

                //     *self.root_ui_element.lock().unwrap() = display.clone();
                // }

                fn damage_window(&self) {
                    self.root_window_damaged
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }

                fn close(&self) {
                    self.is_running
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                }
            }

            applet_initizer(Box::new(AppletRunnerHook {
                root_window_damaged: root_window_damaged.clone(),
                is_running: is_running.clone(),
            }))
        };

        let mut runner = Self {
            applet,
            root_window,
            root_window_damaged,
            ui_event_queue,
            is_running,
            // REVIEW
            window_size: DisplayContainerSize::new(0, 0),
        };

        while runner.is_running.load(std::sync::atomic::Ordering::Relaxed) {
            for ui_event in std::mem::take(&mut *(runner.ui_event_queue.lock().unwrap())) {
                if let UIEvent::WindowResized(window_size) = ui_event {
                    runner.window_size = window_size;
                }

                runner.applet.handle_ui_event(ui_event);
            }

            if runner
                .root_window_damaged
                .swap(false, std::sync::atomic::Ordering::Relaxed)
            {
                *runner.root_window.lock().unwrap() = runner.applet.get_window(runner.window_size);
            }
        }
    }
}
