use singularity_sar::applet::BasicApplet;
use singularity_sttk::nodular_applet::NodularApplet;
use smithay::{
    input::{Seat, SeatState},
    reexports::{
        calloop::{EventLoop, Interest, LoopSignal, PostAction, generic::Generic},
        wayland_server::{
            Client, Display, DisplayHandle, backend::ClientData, protocol::wl_surface::WlSurface,
        },
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};
use std::{ffi::OsString, sync::Arc};

mod compositor;

pub struct WaylandApplet {
    pub start_time: std::time::Instant,
    pub display_handle: DisplayHandle,

    pub loop_signal: LoopSignal,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,

    pub seat: Seat<Self>,

    pub socket_name: OsString,

    pub main_client: Option<Client>,
    pub surface: Option<WlSurface>,
}
impl WaylandApplet {
    pub fn new(
        event_loop: &mut EventLoop<Self>,
        display: &mut smithay::reexports::wayland_server::Display<Self>,
    ) -> Self {
        let start_time = std::time::Instant::now();

        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        // let popups = PopupManager::default();

        // let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);

        // // Data device is responsible for clipboard and drag-and-drop
        // let data_device_state = DataDeviceState::new::<Self>(&dh);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "hello");

        seat.add_keyboard(Default::default(), 200, 25).unwrap();
        seat.add_pointer();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        let loop_signal = event_loop.get_signal();

        WaylandApplet {
            start_time,
            display_handle,
            loop_signal,

            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            // output_manager_state,
            // data_device_state: todo!(),
            // popups,
            seat,

            socket_name,

            main_client: None,
            surface: None,
        }
    }

    fn init_wayland_listener(
        display: &mut Display<Self>,
        event_loop: &mut EventLoop<Self>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket.socket_name().to_os_string();

        // let loop_handle = event_loop.handle();

        // loop_handle
        //     .insert_source(listening_socket, |client_stream, (), state| {
        //         let client = state
        //             .display_handle
        //             .insert_client(client_stream, Arc::new(ClientState::default()))
        //             .unwrap();

        //         state.main_client = Some(client);

        //         println!("Got new client");
        //     })
        //     .unwrap();

        // loop_handle
        //     .insert_source(
        //         Generic::new(
        //             display,
        //             Interest::READ,
        //             smithay::reexports::calloop::Mode::Level,
        //         ),
        //         |_, display, state| {
        //             // Safety: we don't drop the display
        //             unsafe {
        //                 display.get_mut().dispatch_clients(state).unwrap();
        //             }
        //             Ok(PostAction::Continue)
        //         },
        //     )
        //     .unwrap();

        socket_name
    }
}
impl BasicApplet for WaylandApplet {
    fn handle_ui_event(&mut self, _ui_event: singularity_ui::ui_event::UIEvent) {
        todo!()
    }

    fn get_window(&self) -> singularity_ui::ui_element::UIElement {
        todo!()
    }
}
impl NodularApplet for WaylandApplet {
    fn handle_nodular_event(
        &mut self,
        _nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        todo!()
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        todo!()
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        todo!()
    }
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: smithay::reexports::wayland_server::backend::ClientId) {
        println!("Client Initialized");
    }

    fn disconnected(
        &self,
        _client_id: smithay::reexports::wayland_server::backend::ClientId,
        _reason: smithay::reexports::wayland_server::backend::DisconnectReason,
    ) {
        println!("Client Disconnected");
    }
}
