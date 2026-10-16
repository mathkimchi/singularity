use crate::{pluck_place::PluckPlace, smithay::SmithayState, ui_handle::UIHandle};
use calloop::{EventLoop, LoopHandle, LoopSignal};
use singularity_common::sap::{
    client_handle::ClientHandle,
    detached::DetachedTree,
    packets::{StandardEvent, StandardRequest, StateKind, WlSurfaceId},
    raw_client_initializer::RawClientInitializer,
};
use sonamu_ui::{
    PointerGrab,
    display_units::DisplayContainerSize,
    ui_element::{PrimitiveScene, UIElement, UITree},
    ui_event::{KeyModifiers, KeySymbol, UIEvent},
};

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner {
    ui_handle: UIHandle,
    pub(crate) root_client: ClientHandle,

    // // TODO: later
    // subsurfaces: SlotMap<TextureId, EncapsulatedLock<UIElement>>,
    pub(crate) smithay_state: SmithayState,

    window_size: DisplayContainerSize,
    /// Set by damage; the redraw happens once per event loop iteration
    pub(crate) needs_redraw: bool,
    /// What the display was last asked to do with the pointer
    pointer_grab: PointerGrab,
    /// Stops `run` once the display closes
    pub(crate) loop_signal: LoopSignal,

    /// Taken out of the root's tree by a pluck, until it's placed
    pub(crate) plucked: Option<DetachedTree>,
    pub(crate) pluck_place: PluckPlace,
    /// In window px, for drawing what's plucked under the pointer
    pub(crate) pointer_position: [f64; 2],
}
impl AppletRunner {
    fn new(
        event_loop: &LoopHandle<'static, Self>,
        loop_signal: LoopSignal,
        root_applet_initializer: Box<dyn RawClientInitializer>,
        smithay_state: SmithayState,
        on_tty: bool,
    ) -> Self {
        let root_client = ClientHandle::spawn_new_client(
            root_applet_initializer,
            event_loop,
            |request, runner: &mut Self| runner.handle_client_request(request),
        );

        Self {
            ui_handle: UIHandle::init_ui(PrimitiveScene::new_empty(), event_loop, on_tty),
            smithay_state,
            root_client,
            // subsurfaces: SlotMap::with_key(),
            // idk if there's a way to actually get this
            window_size: DisplayContainerSize {
                width: 100,
                height: 100,
            },
            needs_redraw: true,
            pointer_grab: PointerGrab::None,
            loop_signal,
            plucked: None,
            pluck_place: PluckPlace::Idle,
            pointer_position: [0.0, 0.0],
        }
    }

    /// Grabs the pointer while the wl surface under it has an active pointer constraint
    fn update_pointer_grab(&mut self) {
        let grab = self.smithay_state.pointer_grab();
        if grab != self.pointer_grab {
            // a lock ended, so put the pointer where the client asked
            if self.pointer_grab == PointerGrab::Locked
                && let Some(position) = self.smithay_state.lock_end_position.take()
            {
                self.ui_handle.warp_pointer(position);
            }
            self.pointer_grab = grab;
            self.ui_handle.set_pointer_grab(grab);
        }
    }

    /// Blocks until end.
    ///
    /// Runs on the monitor directly (as the actual compositor) when there's no display server to open a window in
    /// (no `WAYLAND_DISPLAY` or `DISPLAY`), or when `SONAMU_BACKEND=tty`; `SONAMU_BACKEND=winit` forces a window.
    pub fn run(root_applet_initializer: Box<dyn RawClientInitializer>) {
        let on_tty = match std::env::var("SONAMU_BACKEND").as_deref() {
            Ok("tty") => true,
            Ok("winit") => false,
            _ => std::env::var_os("WAYLAND_DISPLAY").is_none() && std::env::var_os("DISPLAY").is_none(),
        };
        let mut event_loop = EventLoop::try_new().unwrap();

        let smithay_state = SmithayState::new(&event_loop.handle());
        // For applets that launch wl clients (they set `WAYLAND_DISPLAY` to it).
        // Not `WAYLAND_DISPLAY` itself: Mesa's Vulkan device selection would connect here
        // before this loop runs, and wait forever (and in a window, winit connects to it).
        // SAFETY: nothing else runs yet (the display, root and applet threads start below)
        unsafe {
            std::env::set_var("SONAMU_WAYLAND_DISPLAY", &smithay_state.socket_name);
        }

        let mut runner = Self::new(
            &event_loop.handle(),
            event_loop.get_signal(),
            root_applet_initializer,
            smithay_state,
            on_tty,
        );

        dbg!("Will run event loop");

        event_loop
            .run(None, &mut runner, |runner| {
                // Run after each batch of events, so multiple damages become one redraw
                if std::mem::take(&mut runner.needs_redraw) {
                    runner.redraw_ui();
                }
                // constraints change on client requests and pointer focus changes
                runner.update_pointer_grab();
                // events sent outside client dispatch (eg frame callbacks after a present)
                // only reach clients once flushed
                runner.smithay_state.flush_clients();
            })
            .unwrap();
    }

    pub(crate) fn handle_ui_event(&mut self, event: UIEvent) {
        if let UIEvent::Key {
            raw_keycode,
            pressed: false,
            ..
        } = event
        {
            // evdev -> xkb
            self.release_wl_key(raw_keycode + 8);
        }

        if let UIEvent::WindowResized(screen_size) = event {
            self.window_size = screen_size;
            self.smithay_state.resize_output(screen_size);

            self.needs_redraw = true;
        }

        if let UIEvent::Mouse(mouse_event) = event {
            self.pointer_position = mouse_event.position;
            // what's plucked follows the pointer
            if self.plucked.is_some() {
                self.needs_redraw = true;
            }
        }

        // global keybinds
        if let UIEvent::Key {
            symbol: Some(KeySymbol::Char('p' | 'P')),
            modifiers:
                KeyModifiers {
                    alt: true,
                    ctrl: false,
                    logo: false,
                    ..
                },
            pressed: true,
            ..
        } = event
        {
            self.toggle_pluck_place();
            return;
        }

        self.root_client.send_event(StandardEvent::UIEvent(event));
    }

    /// TODO: take in client id
    pub(crate) fn handle_client_request(&mut self, request: StandardRequest) {
        match request {
            StandardRequest::Damage(StateKind::Surface) => {
                self.needs_redraw = true;
            }
            // the root draws its treeview into its surface
            StandardRequest::Damage(StateKind::Treeview) => {}
            StandardRequest::Quit => self.ui_handle.quit(),
            StandardRequest::WlKeyboardFocus(surface_id) => self.focus_wl_keyboard(surface_id),
            StandardRequest::Plucked(plucked) => self.plucked(plucked),
            StandardRequest::Placed(given_back) => self.placed(given_back),
            StandardRequest::RequestPluck(path) => self.pluck(Some(path)),
            StandardRequest::RequestPlace(path) => self.request_place(path),
        }
    }

    fn redraw_ui(&mut self) {
        let mut content = self.root_client.get_surface();
        if let Some(ghost) = self.plucked_ghost() {
            content = UITree::new_container(vec![content, ghost]);
        }
        let scene = PrimitiveScene::from_ui_tree(
            content,
            self.window_size,
            |surface_id| {
                // NOTE: currently only handles subsurfaces being the wl surfaces
                let wl_surface_id = WlSurfaceId::from_u64(surface_id);
                self.smithay_state
                    .get_wl_surface_as_element(
                        wl_surface_id,
                        &self.ui_handle.device,
                        &self.ui_handle.queue,
                    )
                    .unwrap_or_else(|| {
                        UITree::leaf(UIElement::Backgrounded(sonamu_ui::color::Color::RED))
                    })
            },
        );
        self.smithay_state.set_tile_areas(
            scene
                .subsurface_areas
                .iter()
                .map(|&(surface_id, area)| (WlSurfaceId::from_u64(surface_id), area)),
        );
        self.ui_handle.set_ui_content(scene);
    }
}
