use image::RgbaImage;
use singularity_sttk::nodular_applet::{
    AppletSpawner, AppletSpawnerTrait, NodularApplet, NodularAppletInitializer, NodularRunnerHook,
    recursive_node_applet::RecursiveNodeApplet,
};
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
    input::{Seat, SeatState, keyboard::FilterResult},
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
use sonamu_ui::ui_event::{Key, UIEvent};
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
    _display_handle: DisplayHandle,

    _loop_signal: LoopSignal,

    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,

    seat: Seat<Self>,

    _socket_name: OsString,

    _main_client: Option<Client>,
    surface: Option<WlSurface>,

    image: Arc<Mutex<Option<RgbaImage>>>,
    // I think it might be more efficient to share the seat
    // but this is easier for me to implement
    input_queue: mpsc::Receiver<UIEvent>,
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
        let event_loop: EventLoop<Self> = EventLoop::try_new().unwrap();

        let mut display: smithay::reexports::wayland_server::Display<Self> =
            smithay::reexports::wayland_server::Display::new().unwrap();

        let mut state = Self::create(&event_loop, &display, image, input_queue);

        let mut listener_count = 0;
        let listener = loop {
            if let Ok(listener) = ListeningSocket::bind(format!("wayland-{listener_count}")) {
                break listener;
            }
            listener_count += 1;
        };

        unsafe {
            set_var("WAYLAND_DISPLAY", format!("wayland-{listener_count}"));
        }
        // TODO
        std::process::Command::new(program)
            .env("WAYLAND_DISPLAY", format!("wayland-{listener_count}"))
            .env("MOZ_ENABLE_WAYLAND", "1")
            .spawn()
            .ok();

        let mut renderer = PixmanRenderer::new().unwrap();
        let mut image = pixman::Image::new(pixman::FormatCode::R8G8B8A8, 800, 600, false).unwrap();

        loop {
            while let Ok(ui_event) = state.input_queue.try_recv() {
                state.process_ui_event(ui_event);
            }

            let mut target = renderer.bind(&mut image).unwrap();

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
                    println!("Got a client: {stream:?}");

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

                *state.image.lock().unwrap() = Some(rgba_image);

                // I need this bc if I quit while rendering, it doesn't work
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }

    /// Creates WaylandApplet but doesn't run it
    fn create(
        event_loop: &EventLoop<Self>,
        display: &smithay::reexports::wayland_server::Display<Self>,
        image: Arc<Mutex<Option<RgbaImage>>>,
        input_queue: mpsc::Receiver<UIEvent>,
    ) -> Self {
        let start_time = std::time::Instant::now();

        let display_handle = display.handle();

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, "hello");

        seat.add_keyboard(smithay::input::keyboard::XkbConfig::default(), 200, 25)
            .unwrap();
        seat.add_pointer();

        let socket_name = Self::init_wayland_listener();

        let loop_signal = event_loop.get_signal();

        Self {
            start_time,
            _display_handle: display_handle,
            _loop_signal: loop_signal,

            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            seat,

            _socket_name: socket_name,

            _main_client: None,
            surface: None,

            image,
            input_queue,
        }
    }

    fn init_wayland_listener() -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        listening_socket.socket_name().to_os_string()
    }

    /// NOTE: ignores modifiers
    /// I'm too tired for ts
    /// https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
    fn key_to_keycode(key: Key) -> Option<Keycode> {
        Some(Keycode::new(
            include!(concat!(env!("OUT_DIR"), "/keycode_matches.rs")) + 8,
        ))
    }

    fn process_ui_event(&mut self, ui_event: UIEvent) {
        match ui_event {
            sonamu_ui::ui_event::UIEvent::KeyPress(key, _key_modifiers) => {
                if let Some(keycode) = Self::key_to_keycode(key) {
                    self.seat.get_keyboard().unwrap().input::<(), _>(
                        self,
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
                    self.seat.get_keyboard().unwrap().input::<(), _>(
                        self,
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
            sonamu_ui::ui_event::UIEvent::WindowResized(_) => {}
            sonamu_ui::ui_event::UIEvent::MousePress(_, _display_area) => {
                log::debug!("TODO: handle keypress in wayland applet");
            }
        }
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
    _thread: JoinHandle<WaylandCompositor>,
    input_sender: mpsc::Sender<UIEvent>,
    hook: Box<dyn NodularRunnerHook>,
}
impl WaylandApplet {
    #[must_use]
    pub fn new(hook: Box<dyn NodularRunnerHook>, program: String) -> Self {
        let image = Arc::new(Mutex::new(None));

        let (tx, rx) = mpsc::channel();

        let image_clone = image.clone();
        let thread = thread::spawn(|| WaylandCompositor::new(image_clone, program, rx));

        Self {
            image,
            _thread: thread,
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
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct WaylandSpawner;
        impl AppletSpawnerTrait for WaylandSpawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let program = args.first().unwrap_or(&"kitty");
                // TODO: args later

                Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
                    WaylandApplet::get_boxed_initiator(program.to_string()),
                ))
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
