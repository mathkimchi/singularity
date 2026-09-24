use crate::runner::AppletRunner;
use calloop::{LoopHandle, PostAction};
use singularity_common::sap::packets::{StandardEvent, WlSurfaceId};
use slotmap::SlotMap;
use smithay::{
    backend::{
        input::Keycode,
        renderer::{
            BufferType, buffer_type,
            utils::{RendererSurfaceStateUserData, on_commit_buffer_handler},
        },
    },
    input::{Seat, SeatHandler, SeatState, keyboard::FilterResult},
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
use sonamu_ui::display_units::DisplayContainerSize;
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
    start_time: std::time::Instant,
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
    /// keypress events that need to be sent to a surface
    /// represented by (target surface id, keycode)
    /// Works by first setting key seat's focus to the target surface,
    /// then sending a quick pressed then unpressed event
    /// Ignores things like holding, timestamp, serial, ...
    key_event_queue: calloop::channel::Sender<(WlSurfaceId, u32)>,
    resize_event_queue: calloop::channel::Sender<(WlSurfaceId, DisplayContainerSize)>,
}
impl SmithayState {
    pub fn new(event_handle: &LoopHandle<AppletRunner>) -> Self {
        let start_time = std::time::Instant::now();

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
                |_, display, state| {
                    // profiling::scope!("dispatch_clients");
                    // Safety: we don't drop the display
                    let display = unsafe { display.get_mut() };
                    display.dispatch_clients(state).unwrap();
                    display.flush_clients().unwrap();

                    Ok(PostAction::Continue)
                },
            )
            .unwrap();

        let (key_event_queue, resize_event_queue) = Self::get_event_queues(event_handle);

        Self {
            start_time,
            display_handle,
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            seat,
            surfaces: SlotMap::with_key(),
            key_event_queue,
            resize_event_queue,
        }
    }

    fn get_event_queues(
        event_handle: &LoopHandle<AppletRunner>,
    ) -> (
        calloop::channel::Sender<(WlSurfaceId, u32)>,
        calloop::channel::Sender<(WlSurfaceId, DisplayContainerSize)>,
    ) {
        let (key_event_queue_sender, key_event_queue_reciever) = calloop::channel::channel();

        event_handle
            .insert_source(key_event_queue_reciever, |event, &mut (), state| {
                let calloop::channel::Event::Msg((target_surface_id, keycode)) = event else {
                    return;
                };

                // the surface may already be gone (client quit between the applet
                // queueing this and us handling it)
                let Some(target_surface) =
                    state.smithay_state.surfaces.get(target_surface_id).cloned()
                else {
                    return;
                };

                // target_surface.key(seat, data, key, state, serial, time); // TODO: use this instead of needing to jankily set focus?
                state.smithay_state.seat.get_keyboard().unwrap().set_focus(
                    state,
                    Some(target_surface),
                    Serial::from(42),
                );
                state
                    .smithay_state
                    .seat
                    .get_keyboard()
                    .unwrap()
                    .input::<(), _>(
                        state,
                        Keycode::new(keycode),
                        smithay::backend::input::KeyState::Pressed,
                        // dk bro
                        Serial::from(42),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u32,
                        |_, _, _| FilterResult::Forward,
                    );
                state
                    .smithay_state
                    .seat
                    .get_keyboard()
                    .unwrap()
                    .input::<(), _>(
                        state,
                        Keycode::new(keycode),
                        smithay::backend::input::KeyState::Released,
                        // dk bro
                        Serial::from(43),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u32,
                        |_, _, _| FilterResult::Forward,
                    );

                dbg!("Sent keypress with keycode", keycode);

                // Events queued above only land in wayland-server's internal
                // per-client buffers; this source isn't triggered by client
                // socket activity like the display dispatch source is, so we
                // have to flush explicitly or the bytes never hit the wire.
                state.smithay_state.display_handle.flush_clients().unwrap();
            })
            .unwrap();

        let (resize_event_queue_sender, resize_event_queue_reciever) = calloop::channel::channel();

        event_handle
            .insert_source(
                resize_event_queue_reciever,
                |event: calloop::channel::Event<(WlSurfaceId, DisplayContainerSize)>,
                 &mut (),
                 state| {
                    let calloop::channel::Event::Msg((target_surface_id, new_size)) = event else {
                        return;
                    };

                    // the surface may already be gone (client quit between the applet
                    // queueing this and us handling it)
                    let Some(target_surface) =
                        state.smithay_state.surfaces.get(target_surface_id).cloned()
                    else {
                        dbg!("Event sent to unknown surface");
                        return;
                    };

                    let Some(toplevel) = state
                        .smithay_state
                        .xdg_shell_state
                        .toplevel_surfaces()
                        .iter()
                        .find(|toplevel| toplevel.wl_surface() == &target_surface)
                        .cloned()
                    else {
                        dbg!("Resize sent to surface with no toplevel");
                        return;
                    };

                    toplevel.with_pending_state(|toplevel_state| {
                        toplevel_state.size =
                            Some((new_size.width as i32, new_size.height as i32).into());
                    });
                    toplevel.send_configure();

                    // Events queued above only land in wayland-server's internal
                    // per-client buffers; this source isn't triggered by client
                    // socket activity like the display dispatch source is, so we
                    // have to flush explicitly or the bytes never hit the wire.
                    state.smithay_state.display_handle.flush_clients().unwrap();
                },
            )
            .unwrap();

        (key_event_queue_sender, resize_event_queue_sender)
    }

    /// Returns `None` when the surface has nothing displayable yet
    /// (eg the initial xdg_surface commit carries no buffer by protocol)
    /// or when the surface id is stale.
    pub fn get_wl_surface_as_element(
        &self,
        surface_id: WlSurfaceId,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<TextureView> {
        let surface = self.surfaces.get(surface_id)?;
        let wl_buffer = with_states(surface, |states| {
            let surface_state = states
                .data_map
                .get::<RendererSurfaceStateUserData>()?
                .lock()
                .unwrap();
            surface_state.buffer().cloned()
        })?;

        match buffer_type(&wl_buffer) {
            Some(BufferType::Shm) => {
                // NOTE: everything touching the mapped pool has to stay inside this
                // closure: the pointer is only valid while it runs, and smithay only
                // installs its SIGBUS guard for that duration.
                smithay::wayland::shm::with_buffer_contents(
                    &wl_buffer,
                    |content_ptr, length, metadata| {
                        // SAFETY: valid for the body of this closure only. The client
                        // may mutate the pool concurrently, so don't hold this anywhere.
                        let data = unsafe { std::slice::from_raw_parts(content_ptr, length) };

                        let texture_size = wgpu::Extent3d {
                            width: metadata.width.cast_unsigned(),
                            height: metadata.height.cast_unsigned(),
                            // All textures are stored as 3D
                            depth_or_array_layers: 1,
                        };

                        use smithay::reexports::wayland_server::protocol::wl_shm::Format;
                        let format = match metadata.format {
                            // Both are little-endian 0xAARRGGBB / 0xXXRRGGBB,
                            // ie B,G,R,A in byte order. Xrgb's X byte is ignored
                            // by the shader, so the two map to the same wgpu format.
                            Format::Argb8888 | Format::Xrgb8888 => {
                                wgpu::TextureFormat::Bgra8UnormSrgb
                            }
                            unsupported => {
                                // log::warn!("unsupported wl_shm format: {unsupported:?}");
                                dbg!("unsupported wl_shm format:", unsupported);
                                return None;
                            }
                        };

                        let texture = device.create_texture(&wgpu::TextureDescriptor {
                            size: texture_size,
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format,
                            // COPY_DST means that we want to copy data to this texture
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST,
                            label: Some("wayland_buffer_texture"),
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
                            // NOTE: this is the whole pool; `offset` below picks
                            // out this buffer's region of it.
                            data,
                            // The layout of the texture
                            wgpu::TexelCopyBufferLayout {
                                offset: u64::from(metadata.offset.cast_unsigned()),
                                bytes_per_row: Some(metadata.stride.cast_unsigned()),
                                rows_per_image: Some(metadata.height.cast_unsigned()),
                            },
                            texture_size,
                        );

                        Some(texture.create_view(&TextureViewDescriptor::default()))
                    },
                )
                .ok()
                .flatten()
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

        send_frames_surface_tree(
            surface,
            self.smithay_state.start_time.elapsed().as_millis() as u32,
        );

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
            .send_event(StandardEvent::WlSurfaceRegistered {
                surface_id,
                key_event_queue: self.smithay_state.key_event_queue.clone(),
                resize_event_queue: self.smithay_state.resize_event_queue.clone(),
            });

        println!("New toplevel surface registered");
    }

    fn toplevel_destroyed(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        // Without this the slotmap keeps a dead `WlSurface` forever and the
        // applet holding its id renders a stale/absent buffer.
        // TODO: also tell the owning applet so it can remove itself from the tree
        let wl_surface = surface.wl_surface();
        self.smithay_state
            .surfaces
            .retain(|_, registered| registered != wl_surface);

        self.redraw_ui();
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
