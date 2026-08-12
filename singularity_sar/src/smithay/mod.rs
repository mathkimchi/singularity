use crate::runner::AppletRunner;
use calloop::LoopHandle;
use singularity_common::sap::packets::{StandardEvent, WlSurfaceId};
use slotmap::SlotMap;
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        Client, Display, DisplayHandle, backend::ClientData, protocol::wl_surface::WlSurface,
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        output::OutputHandler,
        security_context::SecurityContext,
        shell::xdg::{XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub security_context: Option<SecurityContext>,
}
impl ClientData for ClientState {
    /// Notification that a client was initialized
    fn initialized(&self, _client_id: smithay::reexports::wayland_server::backend::ClientId) {}
    /// Notification that a client is disconnected
    fn disconnected(
        &self,
        _client_id: smithay::reexports::wayland_server::backend::ClientId,
        _reason: smithay::reexports::wayland_server::backend::DisconnectReason,
    ) {
    }
}

/// TODO: rename to smithay client handle (adds consistency w/ ui handle and client handle)
#[derive(Debug)]
pub struct SmithayState {
    // start_time: std::time::Instant,
    display_handle: DisplayHandle,

    // _loop_signal: LoopSignal,

    // smithay state
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<AppletRunner>,

    seat: Seat<AppletRunner>,
    // _socket_name: OsString,

    // _main_client: Option<Client>,
    // surface: Option<WlSurface>,
    surfaces: SlotMap<WlSurfaceId, WlSurface>,
    // image: Arc<Mutex<Option<RgbaImage>>>,
    // // I think it might be more efficient to share the seat
    // // but this is easier for me to implement
    // input_queue: mpsc::Receiver<UIEvent>,
    // // Singularity stuff
    // hook: Box<dyn NodularRunnerHook>,
}
impl SmithayState {
    pub fn new(event_handle: &LoopHandle<AppletRunner>) -> Self {
        let display: Display<AppletRunner> = Display::new().unwrap();
        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<AppletRunner>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<AppletRunner>(&display_handle);
        let shm_state = ShmState::new::<AppletRunner>(&display_handle, Vec::new());
        let mut seat_state = SeatState::new();

        let mut seat = seat_state.new_wl_seat(&display_handle, "hello");

        seat.add_keyboard(smithay::input::keyboard::XkbConfig::default(), 200, 25)
            .unwrap();
        seat.add_pointer();

        // set up socket
        {
            // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
            let listening_socket = ListeningSocketSource::new_auto().unwrap();

            // Get the name of the listening socket.
            // Clients will connect to this socket.
            let socket_name = listening_socket.socket_name().to_os_string();

            event_handle
                .insert_source(listening_socket, |client_stream, (), state| {
                    let _client = state
                        .smithay_state
                        .display_handle
                        .insert_client(client_stream, Arc::new(ClientState::default()))
                        .unwrap();

                    dbg!("Inserted new client!");
                })
                .unwrap();

            dbg!(&socket_name);

            // unsafe {
            //     set_var("WAYLAND_DISPLAY", socket_name);
            //     // // Firefox just spawns in normal compositor with this unset
            //     // // Whoop dee doo, it still doesn't work
            //     // set_var("MOZ_ENABLE_WAYLAND", "1");
            // }
        }

        Self {
            display_handle,
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            seat,
            surfaces: SlotMap::with_key(),
        }
    }
}

smithay::delegate_dispatch2!(AppletRunner);
// smithay::delegate_dispatch2!(SmithayState);

impl SeatHandler for AppletRunner {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.smithay_state.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
}
impl CompositorHandler for AppletRunner {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.smithay_state.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl BufferHandler for AppletRunner {
    fn buffer_destroyed(
        &mut self,
        _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

impl ShmHandler for AppletRunner {
    fn shm_state(&self) -> &ShmState {
        &self.smithay_state.shm_state
    }
}

impl XdgShellHandler for AppletRunner {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.smithay_state.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.size = Some((800, 600).into());
            state
                .states
                .set(wayland_protocols::xdg::shell::server::xdg_toplevel::State::Activated);
        });
        surface.send_configure();

        self.smithay_state.seat.get_keyboard().unwrap().set_focus(
            self,
            Some(surface.wl_surface().clone()),
            // Idk what serial should be
            Serial::from(42),
        );

        // add surface to list of surfaces
        let surface_id = self
            .smithay_state
            .surfaces
            .insert(surface.wl_surface().clone());

        // let root applet know abt new surface
        self.root_client
            .send_event(StandardEvent::WlSurfaceRegistered { surface_id });

        println!("New toplevel surface registered");
    }

    fn new_popup(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
    }

    fn grab(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        _serial: Serial,
    ) {
    }

    fn reposition_request(
        &mut self,
        _surface: smithay::wayland::shell::xdg::PopupSurface,
        _positioner: smithay::wayland::shell::xdg::PositionerState,
        _token: u32,
    ) {
    }
}

impl OutputHandler for AppletRunner {}
