use image::RgbaImage;
use singularity_sttk::nodular_applet::{
    AppletSpawner, AppletSpawnerTrait, NodularApplet, NodularAppletInitializer, NodularRunnerHook,
};
use singularity_ui::ui_event::{Key, UIEvent};
use smithay::{
    backend::{
        input::Keycode,
        renderer::{
            Bind, Color32F, Frame, Renderer,
            element::{
                Kind,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            pixman::PixmanRenderer,
            utils::draw_render_elements,
        },
    },
    input::{
        Seat, SeatState,
        keyboard::{FilterResult, Keysym},
    },
    reexports::{
        calloop::{EventLoop, LoopSignal},
        pixman,
        wayland_server::{
            Client, DisplayHandle, ListeningSocket,
            backend::ClientData,
            protocol::wl_surface::{self, WlSurface},
        },
    },
    utils::{Rectangle, Serial, Size, Transform},
    wayland::{
        compositor::{
            CompositorClientState, CompositorState, SurfaceAttributes, TraversalAction,
            with_surface_tree_downward,
        },
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};
use std::{
    env::set_var,
    ffi::{OsStr, OsString},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

mod applet_impls;
mod compositor;

/// Currently responsible for running the wl server and client,
/// and saving the output to a shared image
struct WaylandCompositor {
    start_time: std::time::Instant,
    display_handle: DisplayHandle,

    loop_signal: LoopSignal,

    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,

    seat: Seat<Self>,

    socket_name: OsString,

    main_client: Option<Client>,
    surface: Option<WlSurface>,

    image: Arc<Mutex<Option<RgbaImage>>>,
    // I think it might be more efficient to share the seat
    // but this is easier for me to implement
    input_queue: mpsc::Receiver<UIEvent>,
    // // Singularity stuff
    // hook: Box<dyn NodularRunnerHook>,
}
impl WaylandCompositor {
    /// Creates and runs
    /// NOTE: this should be run in a thread that isn't the main thread
    /// REVIEW: I have new and create as two different functions bc it required the least change
    /// REVIEW: make it one function?
    fn new<S: AsRef<OsStr>>(
        image: Arc<Mutex<Option<RgbaImage>>>,
        program: S,
        input_queue: mpsc::Receiver<UIEvent>,
    ) -> Self {
        let mut event_loop: EventLoop<WaylandCompositor> = EventLoop::try_new().unwrap();

        let mut display: smithay::reexports::wayland_server::Display<WaylandCompositor> =
            smithay::reexports::wayland_server::Display::new().unwrap();

        let mut state =
            WaylandCompositor::create(&mut event_loop, &mut display, image, input_queue);

        let mut listener_count = 0;
        let listener = loop {
            if let Ok(listener) = ListeningSocket::bind(format!("wayland-{listener_count}")) {
                break listener;
            }
            listener_count += 1;
        };

        unsafe {
            // set_var("WAYLAND_DISPLAY", &state.socket_name);
            set_var("WAYLAND_DISPLAY", format!("wayland-{listener_count}"));
            // // Firefox just spawns in normal compositor with this unset
            // // Whoop dee doo, it still doesn't work
            // set_var("MOZ_ENABLE_WAYLAND", "1");
        }
        // TODO
        std::process::Command::new(program)
            .env("WAYLAND_DISPLAY", format!("wayland-{listener_count}"))
            .env("MOZ_ENABLE_WAYLAND", "1")
            .spawn()
            .ok();

        let mut renderer = PixmanRenderer::new().unwrap();
        let mut image = pixman::Image::new(pixman::FormatCode::R8G8B8A8, 800, 600, false).unwrap();
        // let mut target = renderer.bind(&mut image).unwrap();

        // event_loop
        //     .run(None, &mut state, |state| {
        //         dbg!(state.surface.is_none());

        //         if let Some(surface) = &state.surface {
        //             // main_client.get_data();
        //             // state.display_handle.get_client(ObjectId:: main_client);
        //             // let elements: Vec<WaylandSurfaceRenderElement<_>> =
        //             //     render_elements_from_surface_tree(
        //             //         &mut renderer,
        //             //         surface,
        //             //         (0, 0),
        //             //         1.0,
        //             //         1.0,
        //             //         smithay::backend::renderer::element::Kind::Unspecified,
        //             //     );

        //             with_states(surface, |states| {
        //                 let mut binding = states.cached_state.get::<SurfaceAttributes>();
        //                 let buffer = binding.current().buffer.as_ref().unwrap();

        //                 // renderer.import_shm_buffer(buffer, Some(surface.data().unwrap()), &[])

        //                 if let BufferAssignment::NewBuffer(buffer) = buffer {
        //                     with_buffer_contents(buffer, |ptr, len, data| {
        //                         let slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        //                         print!("Slice: {:?}", slice);
        //                     })
        //                     .unwrap();
        //                 }
        //             });
        //         }
        //     })
        //     .unwrap();

        loop {
            for ui_event in state.input_queue.try_iter().collect::<Vec<_>>() {
                match ui_event {
                    singularity_ui::ui_event::UIEvent::KeyPress(key, key_modifiers) => {
                        let keysym = Keysym::from_char('e');
                        if let Some(keycode) = state.seat.get_keyboard().unwrap().with_xkb_state(
                            &mut state,
                            |xkb_state| unsafe {
                                xkb_state
                                    .xkb()
                                    .lock()
                                    .ok()?
                                    .keymap()
                                    .key_by_name(keysym.name()?)
                            },
                        ) {
                            log::debug!("{}", keysym.raw());
                            log::debug!("lalala keypress");
                            state.seat.get_keyboard().unwrap().input::<(), _>(
                                &mut state,
                                keycode,
                                smithay::backend::input::KeyState::Pressed,
                                // dk bro
                                Serial::from(42),
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u32,
                                |_, _, _| FilterResult::Forward,
                            );
                            state.seat.get_keyboard().unwrap().input::<(), _>(
                                &mut state,
                                keycode,
                                smithay::backend::input::KeyState::Released,
                                // dk bro
                                Serial::from(43),
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u32,
                                |_, _, _| FilterResult::Forward,
                            );
                        }
                    }
                    singularity_ui::ui_event::UIEvent::WindowResized(_) => {}
                    singularity_ui::ui_event::UIEvent::MousePress(_, display_area) => {
                        log::debug!("TODO: handle keypress in wayland applet");
                    }
                }
            }

            let mut target = renderer.bind(&mut image).unwrap();

            // // let size: Size<usize, smithay::utils::Physical> = Size::new(image.width(), image.height());
            // let size = target.size();
            // let damage = Rectangle::from_size(size);
            let damage = Rectangle::from_size(Size::new(800, 600));
            {
                let elements = state
                    .xdg_shell_state
                    .toplevel_surfaces()
                    .iter()
                    .flat_map(|surface| {
                        render_elements_from_surface_tree(
                            &mut renderer,
                            surface.wl_surface(),
                            (0, 0),
                            1.0,
                            1.0,
                            Kind::Unspecified,
                        )
                    })
                    .collect::<Vec<WaylandSurfaceRenderElement<PixmanRenderer>>>();

                let mut frame = renderer
                    .render(&mut target, Size::new(800, 600), Transform::Normal)
                    .unwrap();
                frame
                    .clear(Color32F::new(0.1, 0.0, 0.0, 1.0), &[damage])
                    .unwrap();
                draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap();
                // We rely on the nested compositor to do the sync for us
                let _ = frame.finish().unwrap();

                for surface in state.xdg_shell_state.toplevel_surfaces() {
                    send_frames_surface_tree(
                        surface.wl_surface(),
                        state.start_time.elapsed().as_millis() as u32,
                    );
                }

                if let Some(stream) = listener.accept().unwrap() {
                    println!("Got a client: {:?}", stream);

                    let _client = display
                        .handle()
                        .insert_client(stream, Arc::new(ClientState::default()))
                        .unwrap();
                    // clients.push(client);
                }

                display.dispatch_clients(&mut state).unwrap();
                display.flush_clients().unwrap();

                let raw_image_data: Vec<_> = unsafe {
                    std::slice::from_raw_parts(image.data(), image.width() * image.height())
                }
                .iter()
                .flat_map(|pixel| pixel.to_be_bytes())
                .collect();

                let rgba_image = RgbaImage::from_vec(800, 600, raw_image_data).unwrap();

                // rgba_image.save("examples/smithay.png").unwrap();
                *state.image.lock().unwrap() = Some(rgba_image);

                // save_buffer(
                //     "examples/smithay.png",
                //     &raw_image_data,
                //     800,
                //     600,
                //     ColorType::Rgba8,
                // )
                // .unwrap();

                // I need this bc if I quit while rendering, it doesn't work
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // // It is important that all events on the display have been dispatched and flushed to clients before
            // // swapping buffers because this operation may block.
            // backend.submit(Some(&[damage])).unwrap();
        }
    }

    /// Creates WaylandApplet but doesn't run it
    fn create(
        event_loop: &mut EventLoop<Self>,
        display: &mut smithay::reexports::wayland_server::Display<Self>,
        image: Arc<Mutex<Option<RgbaImage>>>,
        input_queue: mpsc::Receiver<UIEvent>,
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

        let socket_name = Self::init_wayland_listener();

        let loop_signal = event_loop.get_signal();

        WaylandCompositor {
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

            image,
            input_queue,
        }
    }

    fn init_wayland_listener(// display: &mut Display<Self>,
        // event_loop: &mut EventLoop<Self>,
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

fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been committed
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

pub struct WaylandApplet {
    image: Arc<Mutex<Option<RgbaImage>>>,
    thread: JoinHandle<WaylandCompositor>,
    input_sender: mpsc::Sender<UIEvent>,
    hook: Box<dyn NodularRunnerHook>,
}
impl WaylandApplet {
    pub fn new(hook: Box<dyn NodularRunnerHook>, program: String) -> Self {
        let image = Arc::new(Mutex::new(None));

        let (tx, rx) = mpsc::channel();

        let image_clone = image.clone();
        let thread = thread::spawn(|| WaylandCompositor::new(image_clone, program, rx));

        Self {
            image,
            thread,
            input_sender: tx,
            hook,
        }
    }

    /// REVIEW: cut down on this boilerplate?
    /// Macros would work but might be unnecessary
    /// Just do trait and impl?
    pub fn get_initiator(program: String) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        |hook: Box<dyn NodularRunnerHook>| Self::new(hook, program)
    }
    pub fn get_boxed_initiator(
        program: String,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet> {
        |hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(hook, program))
    }
    pub fn get_applet_spawner() -> AppletSpawner {
        struct WaylandSpawner;
        impl AppletSpawnerTrait for WaylandSpawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let program = args.first().unwrap_or(&"kitty");
                // TODO: args later

                Some(Box::new(WaylandApplet::get_boxed_initiator(
                    program.to_string(),
                )))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(WaylandSpawner)
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
