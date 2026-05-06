// use crate::{ClientState, WaylandApplet};
// use smithay::{
//     input::{Seat, SeatHandler, SeatState},
//     reexports::wayland_server::{
//         Client,
//         protocol::{wl_buffer, wl_surface::WlSurface},
//     },
//     wayland::{
//         buffer::BufferHandler,
//         compositor::{
//             BufferAssignment, CompositorClientState, CompositorHandler, CompositorState,
//             SurfaceAttributes, with_states,
//         },
//         output::OutputHandler,
//         shell::xdg::XdgShellHandler,
//         shm::{ShmHandler, ShmState, with_buffer_contents},
//     },
// };

// impl CompositorHandler for WaylandApplet {
//     fn compositor_state(&mut self) -> &mut CompositorState {
//         println!("A");

//         &mut self.compositor_state
//     }

//     fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
//         println!("B");
//         &client.get_data::<ClientState>().unwrap().compositor_state
//     }

//     fn commit(&mut self, surface: &WlSurface) {
//         // on_commit_buffer_handler::<Self>(surface);
//         // if !is_sync_subsurface(surface) {
//         //     let mut root = surface.clone();
//         //     while let Some(parent) = get_parent(&root) {
//         //         root = parent;
//         //     }
//         //     if let Some(window) = self
//         //         .space
//         //         .elements()
//         //         .find(|w| w.toplevel().unwrap().wl_surface() == &root)
//         //     {
//         //         window.on_commit();
//         //     }
//         // };

//         // xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
//         // resize_grab::handle_commit(&mut self.space, surface);

//         println!("Surface committed");

//         with_states(surface, |states| {
//             let mut binding = states.cached_state.get::<SurfaceAttributes>();
//             let buffer = binding.current().buffer.as_ref().unwrap();

//             // renderer.import_shm_buffer(buffer, Some(surface.data().unwrap()), &[])

//             if let BufferAssignment::NewBuffer(buffer) = buffer {
//                 with_buffer_contents(buffer, |ptr, len, data| {
//                     let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

//                     print!("Slice: {:?}", slice);
//                 })
//                 .unwrap();
//             }
//         });
//     }
// }

// impl BufferHandler for WaylandApplet {
//     fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
// }

// impl ShmHandler for WaylandApplet {
//     fn shm_state(&self) -> &ShmState {
//         &self.shm_state
//     }
// }

// delegate_compositor!(WaylandApplet);
// delegate_shm!(WaylandApplet);

// impl SeatHandler for WaylandApplet {
//     type KeyboardFocus = WlSurface;
//     type PointerFocus = WlSurface;
//     type TouchFocus = WlSurface;

//     fn seat_state(&mut self) -> &mut SeatState<WaylandApplet> {
//         println!("C");
//         &mut self.seat_state
//     }

//     fn cursor_image(
//         &mut self,
//         _seat: &Seat<Self>,
//         _image: smithay::input::pointer::CursorImageStatus,
//     ) {
//     }

//     fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
//         // let dh = &self.display_handle;
//         // let client = focused.and_then(|s| dh.get_client(s.id()).ok());
//         // set_data_device_focus(dh, seat, client);
//     }
// }

// delegate_seat!(WaylandApplet);

// impl XdgShellHandler for WaylandApplet {
//     fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
//         &mut self.xdg_shell_state
//     }

//     fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
//         self.surface = Some(surface.wl_surface().clone());

//         surface.with_pending_state(|state| {
//             state.size = Some((800, 600).into());
//             // state
//             //     .states
//             //     .set(smithay::wayland::shell::xdg::ToplevelStateSet::Activated);
//         });
//         surface.send_configure();

//         println!("New toplevel surface registered");
//     }

//     fn new_popup(
//         &mut self,
//         surface: smithay::wayland::shell::xdg::PopupSurface,
//         positioner: smithay::wayland::shell::xdg::PositionerState,
//     ) {
//     }

//     fn grab(
//         &mut self,
//         surface: smithay::wayland::shell::xdg::PopupSurface,
//         seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
//         serial: smithay::utils::Serial,
//     ) {
//     }

//     fn reposition_request(
//         &mut self,
//         surface: smithay::wayland::shell::xdg::PopupSurface,
//         positioner: smithay::wayland::shell::xdg::PositionerState,
//         token: u32,
//     ) {
//     }
// }

// // Xdg Shell
// delegate_xdg_shell!(WaylandApplet);

use smithay::{
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        Client,
        protocol::{wl_buffer, wl_surface::WlSurface},
    },
    wayland::{
        buffer::BufferHandler,
        compositor::{
            BufferAssignment, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes, with_states,
        },
        output::OutputHandler,
        shell::xdg::XdgShellHandler,
        shm::{ShmHandler, ShmState, with_buffer_contents},
    },
};
use wayland_protocols::xdg::shell::server::xdg_toplevel;

use crate::{ClientState, WaylandApplet};

impl SeatHandler for WaylandApplet {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<WaylandApplet> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        // let dh = &self.display_handle;
        // let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        // set_data_device_focus(dh, seat, client);
    }
}

impl CompositorHandler for WaylandApplet {
    fn compositor_state(&mut self) -> &mut CompositorState {
        println!("A");

        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        println!("B");
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        // on_commit_buffer_handler::<Self>(surface);
        // if !is_sync_subsurface(surface) {
        //     let mut root = surface.clone();
        //     while let Some(parent) = get_parent(&root) {
        //         root = parent;
        //     }
        //     if let Some(window) = self
        //         .space
        //         .elements()
        //         .find(|w| w.toplevel().unwrap().wl_surface() == &root)
        //     {
        //         window.on_commit();
        //     }
        // };

        // xdg_shell::handle_commit(&mut self.popups, &self.space, surface);
        // resize_grab::handle_commit(&mut self.space, surface);

        println!("Surface committed");

        with_states(surface, |states| {
            let mut binding = states.cached_state.get::<SurfaceAttributes>();
            if let Some(BufferAssignment::NewBuffer(buffer)) = binding.current().buffer.as_ref() {
                // renderer.import_shm_buffer(buffer, Some(surface.data().unwrap()), &[])

                with_buffer_contents(buffer, |ptr: *const u8, len, data| {
                    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

                    print!("Slice: {:?}", slice);
                })
                .unwrap();
            }
        });

        println!("TODO");
    }
}

impl BufferHandler for WaylandApplet {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for WaylandApplet {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for WaylandApplet {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        self.surface = Some(surface.wl_surface().clone());

        surface.with_pending_state(|state| {
            state.size = Some((800, 600).into());
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();

        println!("New toplevel surface registered");
    }

    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
    }

    fn grab(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        seat: smithay::reexports::wayland_server::protocol::wl_seat::WlSeat,
        serial: smithay::utils::Serial,
    ) {
    }

    fn reposition_request(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
        token: u32,
    ) {
    }
}

impl OutputHandler for WaylandApplet {}

smithay::delegate_dispatch2!(WaylandApplet);
