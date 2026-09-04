use crate::runner::AppletRunner;
use calloop::{LoopHandle, PostAction};
use singularity_common::sap::packets::{StandardEvent, WlSurfaceId};
use slotmap::SlotMap;
use smithay::{
    backend::renderer::{
        BufferType, buffer_type,
        utils::{RendererSurfaceStateUserData, on_commit_buffer_handler},
    },
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        Client, Display, DisplayHandle, backend::ClientData, protocol::wl_surface::WlSurface,
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes,
            TraversalAction, with_states, with_surface_tree_downward,
        },
        output::OutputHandler,
        security_context::SecurityContext,
        shell::xdg::{XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
        socket::ListeningSocketSource,
    },
};
use std::sync::Arc;
use wgpu::{TextureView, TextureViewDescriptor};

#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    pub security_context: Option<SecurityContext>,
}
impl ClientData for ClientState {
    /// Notification that a client was initialized
    fn initialized(&self, _client_id: smithay::reexports::wayland_server::backend::ClientId) {
        dbg!("Client initialized");
    }
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

        event_handle
            .insert_source(
                calloop::generic::Generic::new(
                    display,
                    calloop::Interest::READ,
                    calloop::Mode::Level,
                ),
                |_, display, data| {
                    // profiling::scope!("dispatch_clients");
                    // Safety: we don't drop the display
                    let display = unsafe { display.get_mut() };
                    display.dispatch_clients(data).unwrap();
                    display.flush_clients().unwrap();

                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

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

    pub fn get_wl_surface_as_element(
        &self,
        surface_id: WlSurfaceId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> TextureView {
        let surface = self.surfaces.get(surface_id).unwrap();
        let wl_buffer = with_states(surface, |states| {
            let surface_state = states
                .data_map
                .get::<RendererSurfaceStateUserData>()
                .unwrap()
                .lock()
                .unwrap();
            let buffer = surface_state.buffer().unwrap();
            buffer.clone()
        });

        match buffer_type(&wl_buffer) {
            Some(BufferType::Shm) => {
                dbg!("shm");

                let (data, metadata) = smithay::wayland::shm::with_buffer_contents(
                    &wl_buffer,
                    |content_ptr, length, metadata| {
                        (
                            unsafe { std::slice::from_raw_parts(content_ptr, length) },
                            metadata,
                        )
                    },
                )
                .unwrap();

                let data = &data
                    .chunks_exact(4)
                    .flat_map(|x| [x[0], x[1], x[2], 0xFF])
                    .collect::<Vec<_>>();

                // let data = &data
                //     .iter()
                //     .enumerate()
                //     .map(|(i, _)| match i % 4 {
                //         0 => (i % 256) as _,
                //         1 => (i % 256) as _,
                //         2 => (i % 256) as _,
                //         // Alpha
                //         3 => 255,
                //         _ => unreachable!(),
                //     })
                //     .collect::<Vec<_>>();

                let texture_size = wgpu::Extent3d {
                    width: metadata.width.cast_unsigned(),
                    height: metadata.height.cast_unsigned(),
                    // All textures are stored as 3D
                    depth_or_array_layers: 1,
                };

                dbg!(metadata.format);
                let format = match metadata.format {
                    smithay::reexports::wayland_server::protocol::wl_shm::Format::Argb8888 => {
                        wgpu::TextureFormat::Rgba8UnormSrgb
                    }
                    _ => todo!(),
                };

                image::save_buffer(
                    "./examples/smithay.png",
                    data,
                    metadata.width.try_into().unwrap(),
                    metadata.height.try_into().unwrap(),
                    image::ColorType::Rgba8,
                )
                .unwrap();

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1, // We'll talk about this a little later
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("wayland_buffer_texture"),
                    // This is the same as with the SurfaceConfig. It
                    // specifies what texture formats can be used to
                    // create TextureViews for this texture. The base
                    // texture format (Rgba8UnormSrgb in this case) is
                    // always supported. Note that using a different
                    // texture format is not supported on the WebGL2
                    // backend.
                    view_formats: &[],
                });

                queue.write_texture(
                    // Tells wgpu where to copy the pixel data
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    // The actual pixel data
                    data,
                    // The layout of the texture
                    wgpu::TexelCopyBufferLayout {
                        // NOTE: Idk why this is 1
                        offset: u64::from(metadata.offset.cast_unsigned()),
                        bytes_per_row: Some(metadata.stride.cast_unsigned()),
                        rows_per_image: Some(metadata.height.cast_unsigned()),
                    },
                    texture_size,
                );

                texture.create_view(&TextureViewDescriptor::default())
            }
            Some(BufferType::Dma) => {
                dbg!("Dma");
                todo!()
            }
            Some(BufferType::SinglePixel | _) | None => todo!(),
        }
    }
}

/// NOTE: Disclosure: from by GitHub Copilot
/// Fixes the Alacritty problem.
/// See 2026-09-04 DEVLOG
/// I think this is just going through the entire tree and telling all the frame callbacks that the request was done
fn send_frames_surface_tree(surface: &WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
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
        dbg!("Committed");
        on_commit_buffer_handler::<Self>(surface);

        send_frames_surface_tree(surface, 0);

        dbg!("TODO: make this redraw request");
        self.redraw_ui();
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
        dbg!("Handling new toplevel surface");
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
