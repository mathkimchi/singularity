use crate::applet::BasicApplet;
use singularity_ui::{UIDisplay, ui_element::UIElement, ui_event::UIEvent};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner<A: BasicApplet> {
    applet: A,

    // fields for dealing with the UI
    root_ui_element: Arc<Mutex<UIElement>>,
    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
    is_running: Arc<AtomicBool>,
}
impl<A: BasicApplet> AppletRunner<A> {
    /// Returns after the applet is closed.
    ///
    /// The logic of this is similar to `UIDisplay::run_display` in `wayland_backend`
    pub fn run(applet_initializing_data: A::InitializingData) {
        let root_ui_element = Arc::new(Mutex::new(UIElement::Nothing));
        let ui_event_queue = Arc::new(Mutex::new(Vec::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        {
            // clone to satisfy compiler
            let root_ui_element = root_ui_element.clone();
            let ui_event_queue = ui_event_queue.clone();
            let is_running = is_running.clone();

            std::thread::spawn(move || {
                UIDisplay::run_display(root_ui_element, ui_event_queue, is_running)
            });
        }

        let applet = {
            // clone to satisfy compiler
            let root_ui_element = root_ui_element.clone();

            A::initialize(
                applet_initializing_data,
                Box::new(move |display: &UIElement| {
                    // TODO: send reminder as well

                    *root_ui_element.lock().unwrap() = display.clone();
                }),
            )
        };

        let mut runner = Self {
            applet,
            root_ui_element,
            ui_event_queue,
            is_running,
        };

        while runner.is_running.load(std::sync::atomic::Ordering::Relaxed) {
            for ui_event in std::mem::take(&mut *(runner.ui_event_queue.lock().unwrap())) {
                // use singularity_ui::ui_event::{KeyModifiers, UIEvent};
                // match ui_event {
                //     // UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.raw_code == 16 => {
                //     //     // Ctrl+Q
                //     //     dbg!("Ending demo");
                //     //     is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                //     //     return;
                //     // }
                //     UIEvent::KeyPress(_, _) => {
                //         test_widget.handle_event(singularity_common::tab::packets::Event::UIEvent(
                //             ui_event,
                //         ));
                //     }
                //     UIEvent::WindowResized(_) => {}
                //     UIEvent::MousePress(
                //         [[click_x, click_y], [tot_width, tot_height]],
                //         container,
                //     ) => {
                //         test_widget.handle_event(singularity_common::tab::packets::Event::UIEvent(
                //             singularity_ui::ui_event::UIEvent::MousePress(
                //                 [[click_x, click_y], [tot_width, tot_height]],
                //                 container,
                //             ),
                //         ));
                //     }
                // }

                runner.applet.handle_ui_event(ui_event);
            }
        }
    }

    // /// Returns after the applet is closed.
    // pub fn run(self) {
    //     todo!()
    // }
}
