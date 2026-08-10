use crate::{client_handle::ClientHandle, smithay::SmithayState};
use calloop::{EventLoop, LoopHandle};
use singularity_common::sap::{
    packets::{StandardEvent, StandardRequest},
    raw_client_initializer::RawClientInitializer,
};
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{
    UIDisplay,
    display_units::DisplayContainerSize,
    ui_element::{PrimitiveScene, UIElement},
    ui_event::UIEvent,
};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};

/// For the runner/main app being UI'd to hold
///
/// Instead of storing the event receiver in here,
/// it is registered in the calloop.
struct UIHandle {
    is_running: Arc<AtomicBool>,
    ui_content: EncapsulatedLock<PrimitiveScene>,
}
impl UIHandle {
    pub fn init_ui(
        initial_content: PrimitiveScene,
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
                    dbg!("UI closed; haven't thought abt what to do");
                    return;
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

    pub fn set_ui_content(&self, new_content: PrimitiveScene) {
        self.ui_content.set(new_content);
    }
}

slotmap::new_key_type! {
    pub struct TextureId;
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

    // // TODO: later
    // subsurfaces: SlotMap<TextureId, EncapsulatedLock<UIElement>>,
    smithay_state: SmithayState,

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
            ui_handle: UIHandle::init_ui(PrimitiveScene::new_empty(), &event_loop),
            smithay_state: SmithayState::new(&event_loop),
            event_loop,
            root_client,
            // subsurfaces: SlotMap::with_key(),
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
    }

    fn handle_ui_event(&mut self, event: UIEvent) {
        if let UIEvent::WindowResized(screen_size) = event {
            self.window_size = screen_size;

            self.redraw_ui();
        }

        self.root_client.send_event(StandardEvent::UIEvent(event));
    }

    /// TODO: take in client id
    pub(crate) fn handle_client_request(&mut self, request: StandardRequest) {
        match request {
            StandardRequest::DamageSurface => {
                self.redraw_ui();
            }
            StandardRequest::DamageTreeview => todo!(),
            StandardRequest::Quit => {
                self.ui_handle
                    .is_running
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn redraw_ui(&mut self) {
        self.ui_handle
            .set_ui_content(PrimitiveScene::from_ui_element(
                self.root_client.get_surface(),
                self.window_size,
            ));
    }
}
