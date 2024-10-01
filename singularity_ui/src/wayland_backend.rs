use crate::UIElement;
use smithay_client_toolkit::{
    activation::{ActivationState, RequestData},
    compositor::CompositorState,
    output::OutputState,
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        xdg::{
            window::{Window, WindowDecorations},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm,
    },
};
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use ui_event::{KeyModifiers, UIEvent};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_pointer},
    Connection,
};

pub const FRAME_RATE: f32 = 30.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

pub struct UIDisplay {
    root_element: Arc<Mutex<UIElement>>,

    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,
    xdg_activation: Option<ActivationState>,

    /// REVIEW: make sure rwlock is good for this
    is_running: Arc<RwLock<bool>>,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    shift: Option<u32>,
    buffer: Option<Buffer>,
    window: Window,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    key_modifiers: KeyModifiers,
    pointer: Option<wl_pointer::WlPointer>,
    loop_handle: LoopHandle<'static, UIDisplay>,
}
impl UIDisplay {
    /// Returns when display is closed.
    pub fn run_display(
        root_element: Arc<Mutex<UIElement>>,
        ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
        is_running: Arc<RwLock<bool>>,
    ) {
        // All Wayland apps start by connecting the compositor (server).
        let conn = Connection::connect_to_env().unwrap();

        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();
        let mut event_loop: EventLoop<UIDisplay> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");
        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue)
            .insert(loop_handle)
            .unwrap();

        // The compositor (not to be confused with the server which is commonly called the compositor) allows
        // configuring surfaces to be presented.
        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        // For desktop platforms, the XDG shell is the standard protocol for creating desktop windows.
        let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");
        // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
        // we share with the compositor process.
        let shm = Shm::bind(&globals, &qh).expect("wl shm is not available.");
        // If the compositor supports xdg-activation it probably wants us to use it to get focus
        let xdg_activation = ActivationState::bind(&globals, &qh).ok();

        // A window is created from a surface.
        let surface = compositor.create_surface(&qh);
        // And then we can create the window.
        let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
        // Configure the window, this may include hints to the compositor about the desired minimum size of the
        // window, app id for WM identification, the window title, etc.
        window.set_title("A wayland window");
        // GitHub does not let projects use the `org.github` domain but the `io.github` domain is fine.
        window.set_app_id("io.github.smithay.client-toolkit.SimpleWindow");
        window.set_min_size(Some((256, 256)));
        window.set_maximized();

        // In order for the window to be mapped, we need to perform an initial commit with no attached buffer.
        // For more info, see WaylandSurface::commit
        //
        // The compositor will respond with an initial configure that we can then use to present to the window with
        // the correct options.
        window.commit();

        // To request focus, we first need to request a token
        if let Some(activation) = xdg_activation.as_ref() {
            activation.request_token(
                &qh,
                RequestData {
                    seat_and_serial: None,
                    surface: Some(window.wl_surface().clone()),
                    app_id: Some(String::from(
                        "io.github.smithay.client-toolkit.SimpleWindow",
                    )),
                },
            )
        }

        // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
        // initial memory allocation.
        let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

        let mut ui_display = UIDisplay {
            root_element,
            ui_event_queue,

            // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
            // listen for seats and outputs.
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),
            shm,
            xdg_activation,

            is_running,
            first_configure: true,
            pool,
            width: 256,
            height: 256,
            shift: None,
            buffer: None,
            window,
            keyboard: None,
            key_modifiers: KeyModifiers::default(),
            pointer: None,
            loop_handle: event_loop.handle(),
        };

        // We don't draw immediately, the configure will notify us when to first draw.
        while *ui_display.is_running.read().unwrap() {
            event_loop
                .dispatch(
                    Duration::from_secs_f32(FRAME_DELTA_SECONDS),
                    &mut ui_display,
                )
                .unwrap();
        }
        println!("exiting example");
    }
}
mod drawing_impls {
    use super::{Color, UIDisplay};
    use crate::{
        display_units::{DisplayArea, DisplayCoord, DisplaySize, DisplayUnits},
        CharCell, UIElement,
    };
    use font_kit::{
        family_name::FamilyName,
        properties::{Properties, Weight},
        source::SystemSource,
    };
    use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};
    use smithay_client_toolkit::shell::WaylandSurface;
    use wayland_client::{protocol::wl_shm, Connection, QueueHandle};

    impl UIElement {
        fn fill_rect(dt: &mut DrawTarget, area: DisplayArea, color: Color) {
            dt.fill_rect(
                area.0.x.pixels(dt.width()) as f32,
                area.0.y.pixels(dt.height()) as f32,
                area.size().width.pixels(dt.width()) as f32,
                area.size().height.pixels(dt.height()) as f32,
                &Source::Solid(SolidSource {
                    r: color.0[0],
                    g: color.0[1],
                    b: color.0[2],
                    a: color.0[3],
                }),
                &DrawOptions::new(),
            );
        }

        fn draw(&self, dt: &mut DrawTarget, container_area: DisplayArea) {
            /// think this is height in pixels
            const FONT_SIZE: i32 = 12;

            match self {
                UIElement::Container(children) => {
                    for (ui_element, area) in children {
                        // draw the inner widget
                        ui_element.draw(dt, area.map_onto(container_area));
                    }
                }
                UIElement::Bordered(inner_element) => {
                    // draw the border
                    Self::fill_rect(dt, container_area, Color::LIGHT_GREEN);

                    let inner_area = DisplayArea(
                        DisplayCoord::new(1.into(), 1.into()),
                        DisplayCoord::new(
                            DisplayUnits::from_mixed(-1, 1.0),
                            DisplayUnits::from_mixed(-1, 1.0),
                        ),
                    )
                    .map_onto(container_area);

                    // clear the inside of the border
                    Self::fill_rect(dt, inner_area, Color::BLACK);

                    // draw the inner widget
                    inner_element.draw(dt, inner_area);
                }
                UIElement::Text(text) => {
                    dt.draw_text(
                        &SystemSource::new()
                            .select_best_match(
                                &[FamilyName::Monospace],
                                Properties::new().weight(Weight::MEDIUM),
                            )
                            .unwrap()
                            .load()
                            .unwrap(),
                        FONT_SIZE as f32,
                        text,
                        Point::new(0., 0.),
                        &Source::Solid(SolidSource {
                            r: 0,
                            g: 0,
                            b: 0xFF,
                            a: 0xFF,
                        }),
                        &DrawOptions::new(),
                    );
                }
                UIElement::CharGrid(char_grid) => {
                    let font = SystemSource::new()
                        .select_best_match(
                            &[FamilyName::Monospace],
                            Properties::new().weight(Weight::MEDIUM),
                        )
                        .unwrap()
                        .load()
                        .unwrap();

                    for (line_index, line) in char_grid.content.iter().enumerate() {
                        for (col_index, CharCell { character, fg, bg }) in line.iter().enumerate() {
                            let top_left = DisplayCoord::new(
                                container_area.0.x
                                    + DisplayUnits::Pixels(FONT_SIZE / 2 * (col_index as i32)),
                                container_area.0.y
                                    + DisplayUnits::Pixels(FONT_SIZE * (line_index as i32) + 1),
                            );
                            let bot_left = DisplayCoord::new(
                                container_area.0.x
                                    + DisplayUnits::Pixels(FONT_SIZE / 2 * (col_index as i32)),
                                container_area.0.y
                                    + DisplayUnits::Pixels(FONT_SIZE * (line_index + 1) as i32),
                            );

                            Self::fill_rect(
                                dt,
                                DisplayArea::from_corner_size(
                                    top_left,
                                    DisplaySize::new(
                                        (FONT_SIZE / 2 + 1).into(),
                                        (FONT_SIZE + 2).into(),
                                    ),
                                ),
                                *bg,
                            );

                            if character == &' ' {
                                continue;
                            }

                            dt.draw_text(
                                &font,
                                FONT_SIZE as f32,
                                &character.to_string(),
                                // `start` is actually bottom left corner
                                bot_left.into_point(dt),
                                &raqote::Source::Solid(SolidSource {
                                    r: fg.0[0],
                                    g: fg.0[1],
                                    b: fg.0[2],
                                    a: fg.0[3],
                                }),
                                &DrawOptions::new(),
                            );
                        }
                    }
                }
            }
        }
    }

    impl UIDisplay {
        pub fn draw(&mut self, _conn: &Connection, qh: &QueueHandle<Self>) {
            let stride = self.width as i32 * 4;

            let buffer = self.buffer.get_or_insert_with(|| {
                self.pool
                    .create_buffer(
                        self.width as i32,
                        self.height as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    )
                    .expect("create buffer")
                    .0
            });

            let canvas = match self.pool.canvas(buffer) {
                Some(canvas) => canvas,
                None => {
                    // This should be rare, but if the compositor has not released the previous
                    // buffer, we need double-buffering.
                    let (second_buffer, canvas) = self
                        .pool
                        .create_buffer(
                            self.width as i32,
                            self.height as i32,
                            stride,
                            wl_shm::Format::Argb8888,
                        )
                        .expect("create buffer");
                    *buffer = second_buffer;
                    canvas
                }
            };

            // Draw to the window:
            // FIXME find an actual fix to the height difference
            if canvas.len() as u32 == 4 * self.width * self.height {
                let mut dt = DrawTarget::new(self.width as i32, self.height as i32);
                self.root_element
                    .lock()
                    .unwrap()
                    .draw(&mut dt, DisplayArea::FULL);
                canvas.copy_from_slice(dt.get_data_u8());
            }

            // Damage the entire window
            self.window
                .wl_surface()
                .damage_buffer(0, 0, self.width as i32, self.height as i32);

            // Request our next frame
            self.window
                .wl_surface()
                .frame(qh, self.window.wl_surface().clone());

            // Attach and commit to present.
            buffer
                .attach_to(self.window.wl_surface())
                .expect("buffer attach");
            self.window.commit();
        }
    }
}
mod ui_display_wayland_impls {
    use super::{
        ui_event::{Key, KeyModifiers},
        UIDisplay,
    };
    use smithay_client_toolkit::{
        activation::{ActivationHandler, RequestData},
        compositor::CompositorHandler,
        delegate_activation, delegate_compositor, delegate_keyboard, delegate_output,
        delegate_pointer, delegate_registry, delegate_seat, delegate_shm, delegate_xdg_shell,
        delegate_xdg_window,
        output::{OutputHandler, OutputState},
        registry::{ProvidesRegistryState, RegistryState},
        registry_handlers,
        seat::{
            keyboard::{KeyboardHandler, Keysym, Modifiers},
            pointer::{PointerEvent, PointerEventKind, PointerHandler},
            Capability, SeatHandler, SeatState,
        },
        shell::{
            xdg::window::{Window, WindowConfigure, WindowHandler},
            WaylandSurface,
        },
        shm::{Shm, ShmHandler},
    };
    use wayland_client::{
        protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
        Connection, QueueHandle,
    };

    impl CompositorHandler for UIDisplay {
        fn scale_factor_changed(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _new_factor: i32,
        ) {
            // Not needed for this example.
        }

        fn transform_changed(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _new_transform: wl_output::Transform,
        ) {
            // Not needed for this example.
        }

        fn frame(
            &mut self,
            conn: &Connection,
            qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _time: u32,
        ) {
            self.draw(conn, qh);
        }

        fn surface_enter(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _output: &wl_output::WlOutput,
        ) {
            // Not needed for this example.
        }

        fn surface_leave(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _surface: &wl_surface::WlSurface,
            _output: &wl_output::WlOutput,
        ) {
            // Not needed for this example.
        }
    }

    impl OutputHandler for UIDisplay {
        fn output_state(&mut self) -> &mut OutputState {
            &mut self.output_state
        }

        fn new_output(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }

        fn update_output(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }

        fn output_destroyed(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _output: wl_output::WlOutput,
        ) {
        }
    }

    impl WindowHandler for UIDisplay {
        fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
            *self.is_running.write().unwrap() = false;
        }

        fn configure(
            &mut self,
            conn: &Connection,
            qh: &QueueHandle<Self>,
            _window: &Window,
            configure: WindowConfigure,
            _serial: u32,
        ) {
            self.buffer = None;
            self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
            self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);

            // Initiate the first draw.
            if self.first_configure {
                self.first_configure = false;
                self.draw(conn, qh);
            }
        }
    }

    impl ActivationHandler for UIDisplay {
        type RequestData = RequestData;

        fn new_token(&mut self, token: String, _data: &Self::RequestData) {
            self.xdg_activation
                .as_ref()
                .unwrap()
                .activate::<UIDisplay>(self.window.wl_surface(), token);
        }
    }

    impl SeatHandler for UIDisplay {
        fn seat_state(&mut self) -> &mut SeatState {
            &mut self.seat_state
        }

        fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

        fn new_capability(
            &mut self,
            _conn: &Connection,
            qh: &QueueHandle<Self>,
            seat: wl_seat::WlSeat,
            capability: Capability,
        ) {
            if capability == Capability::Keyboard && self.keyboard.is_none() {
                println!("Set keyboard capability");
                let keyboard = self
                    .seat_state
                    .get_keyboard_with_repeat(
                        qh,
                        &seat,
                        None,
                        self.loop_handle.clone(),
                        Box::new(|_state, _wl_kbd, event| {
                            println!("Repeat: {:?} ", event);
                        }),
                    )
                    .expect("Failed to create keyboard");

                self.keyboard = Some(keyboard);
            }

            if capability == Capability::Pointer && self.pointer.is_none() {
                println!("Set pointer capability");
                let pointer = self
                    .seat_state
                    .get_pointer(qh, &seat)
                    .expect("Failed to create pointer");
                self.pointer = Some(pointer);
            }
        }

        fn remove_capability(
            &mut self,
            _conn: &Connection,
            _: &QueueHandle<Self>,
            _: wl_seat::WlSeat,
            capability: Capability,
        ) {
            if capability == Capability::Keyboard && self.keyboard.is_some() {
                println!("Unset keyboard capability");
                self.keyboard.take().unwrap().release();
            }

            if capability == Capability::Pointer && self.pointer.is_some() {
                println!("Unset pointer capability");
                self.pointer.take().unwrap().release();
            }
        }

        fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    }

    impl KeyboardHandler for UIDisplay {
        fn enter(
            &mut self,
            _: &Connection,
            _: &QueueHandle<Self>,
            _: &wl_keyboard::WlKeyboard,
            _surface: &wl_surface::WlSurface,
            _: u32,
            _: &[u32],
            _keysyms: &[Keysym],
        ) {
        }

        fn leave(
            &mut self,
            _: &Connection,
            _: &QueueHandle<Self>,
            _: &wl_keyboard::WlKeyboard,
            _surface: &wl_surface::WlSurface,
            _: u32,
        ) {
        }

        fn press_key(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _: &wl_keyboard::WlKeyboard,
            _: u32,
            event: Key,
        ) {
            dbg!(event.raw_code);
            self.ui_event_queue
                .lock()
                .unwrap()
                .push(super::ui_event::UIEvent::KeyPress(
                    event,
                    self.key_modifiers,
                ));
        }

        fn release_key(
            &mut self,
            _: &Connection,
            _: &QueueHandle<Self>,
            _: &wl_keyboard::WlKeyboard,
            _: u32,
            _event: Key,
        ) {
        }

        fn update_modifiers(
            &mut self,
            _: &Connection,
            _: &QueueHandle<Self>,
            _: &wl_keyboard::WlKeyboard,
            _serial: u32,
            modifiers: Modifiers,
            _layout: u32,
        ) {
            self.key_modifiers = KeyModifiers::from(modifiers);
        }
    }

    impl PointerHandler for UIDisplay {
        fn pointer_frame(
            &mut self,
            _conn: &Connection,
            _qh: &QueueHandle<Self>,
            _pointer: &wl_pointer::WlPointer,
            events: &[PointerEvent],
        ) {
            for event in events {
                // Ignore events for other surfaces
                if &event.surface != self.window.wl_surface() {
                    continue;
                }

                match event.kind {
                    PointerEventKind::Enter { .. } => {
                        println!("Pointer entered @{:?}", event.position);
                    }
                    PointerEventKind::Leave { .. } => {
                        println!("Pointer left");
                    }
                    PointerEventKind::Motion { .. } => {}
                    PointerEventKind::Press { button, .. } => {
                        println!("Press {:x} @ {:?}", button, event.position);
                        self.shift = self.shift.xor(Some(0));
                    }
                    PointerEventKind::Release { button, .. } => {
                        println!("Release {:x} @ {:?}", button, event.position);
                    }
                    PointerEventKind::Axis {
                        horizontal,
                        vertical,
                        ..
                    } => {
                        println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                    }
                }
            }
        }
    }

    impl ShmHandler for UIDisplay {
        fn shm_state(&mut self) -> &mut Shm {
            &mut self.shm
        }
    }

    delegate_compositor!(UIDisplay);
    delegate_output!(UIDisplay);
    delegate_shm!(UIDisplay);

    delegate_seat!(UIDisplay);
    delegate_keyboard!(UIDisplay);
    delegate_pointer!(UIDisplay);

    delegate_xdg_shell!(UIDisplay);
    delegate_xdg_window!(UIDisplay);
    delegate_activation!(UIDisplay);

    delegate_registry!(UIDisplay);

    impl ProvidesRegistryState for UIDisplay {
        fn registry(&mut self) -> &mut RegistryState {
            &mut self.registry_state
        }
        registry_handlers![OutputState, SeatState,];
    }
}

pub mod ui_event {
    use smithay_client_toolkit::seat::keyboard;

    /// FIXME: not great that I am reexporting egui's event, given that the goal is to be backend agnostic.
    /// I am doing it right now because I'd rather get something working sooner, even if I have to compromise a bit
    pub enum UIEvent {
        KeyPress(Key, KeyModifiers),
    }
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct KeyModifiers {
        pub ctrl: bool,
        pub alt: bool,
        pub shift: bool,
        pub caps_lock: bool,
        pub logo: bool,
        pub num_lock: bool,
    }
    pub type Key = keyboard::KeyEvent;

    pub trait KeyTrait {
        fn to_alphabet(&self) -> Option<char>;
        fn to_digit(&self) -> Option<u8>;
        fn to_char(&self) -> Option<char>;
    }
    impl KeyTrait for Key {
        fn to_alphabet(&self) -> Option<char> {
            let c = self.to_char()?;
            if c.is_ascii() {
                Some(c)
            } else {
                None
            }
        }

        fn to_digit(&self) -> Option<u8> {
            let c = self.to_char()?;
            c.to_digit(10).map(|c| c as u8)
        }

        fn to_char(&self) -> Option<char> {
            if self.raw_code == 28 {
                return Some('\n');
            }
            self.keysym.key_char()
        }
    }

    impl KeyModifiers {
        pub const NONE: Self = KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            num_lock: false,
        };

        pub const CTRL: Self = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            num_lock: false,
        };

        pub const ALT: Self = KeyModifiers {
            ctrl: false,
            alt: true,
            shift: false,
            caps_lock: false,
            logo: false,
            num_lock: false,
        };
    }
    impl From<keyboard::Modifiers> for KeyModifiers {
        fn from(
            keyboard::Modifiers {
                ctrl,
                alt,
                shift,
                caps_lock,
                logo,
                num_lock,
            }: keyboard::Modifiers,
        ) -> Self {
            Self {
                ctrl,
                alt,
                shift,
                caps_lock,
                logo,
                num_lock,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Color([0, 0, 0, 0]);
    pub const BLACK: Self = Color([0, 0, 0, 0xFF]);
    pub const LIGHT_YELLOW: Self = Color([0xFF, 0xFF, 0, 0xFF]);
    pub const LIGHT_GREEN: Self = Color([0, 0xFF, 0, 0xFF]);
    pub const LIGHT_BLUE: Self = Color([0, 0, 0xFF, 0xFF]);
}
impl From<Color> for raqote::Color {
    fn from(value: Color) -> Self {
        raqote::Color::new(value.0[3], value.0[0], value.0[1], value.0[2])
    }
}
