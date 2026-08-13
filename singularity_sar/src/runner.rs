use crate::{client_handle::ClientHandle, smithay::SmithayState, ui_handle::UIHandle};
use calloop::{EventLoop, LoopHandle};
use singularity_common::sap::{
    packets::{StandardEvent, StandardRequest, WlSurfaceId},
    raw_client_initializer::RawClientInitializer,
};
use sonamu_ui::{
    display_units::DisplayContainerSize,
    ui_element::{PrimitiveScene, UIElement},
    ui_event::UIEvent,
};

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner {
    event_loop: LoopHandle<'static, Self>,

    ui_handle: UIHandle,
    pub(crate) root_client: ClientHandle,

    // // TODO: later
    // subsurfaces: SlotMap<TextureId, EncapsulatedLock<UIElement>>,
    pub(crate) smithay_state: SmithayState,

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

        dbg!("Will run event loop");

        event_loop
            .run(None, &mut runner, |_| {
                // I think this is run between events, but I don't need this rn
            })
            .unwrap();
    }

    pub(crate) fn handle_ui_event(&mut self, event: UIEvent) {
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
                |surface_id| {
                    // NOTE: currently only handles subsurfaces being the wl surfaces
                    let wl_surface_id = WlSurfaceId::from_u64(surface_id);
                    UIElement::Texture(self.smithay_state.get_wl_surface_as_element(wl_surface_id))
                },
            ));
    }
}
