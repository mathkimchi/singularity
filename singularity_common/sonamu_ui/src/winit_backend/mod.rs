//! An agnostic_backend to replace the wayland_backend.
//!
//! A bulk of this code originates from [Glyphon's hello world](https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs)
//! as well as the old wayland_backend.

use crate::{
    display_units::DisplayContainerSize,
    ui_element::PrimitiveScene,
    winit_backend::{
        rendering::{CharGridRenderer, ImageRenderer, RectangleRenderer, Vertex},
        ui_event::{KeyModifiers, UIEvent},
    },
};
use calloop::channel::Sender;
use glyphon::{FontSystem, SwashCache, TextAtlas};
use smithay::reexports::winit::{
    platform::wayland::EventLoopBuilderExtWayland as _, window::Window,
};
use sonamu_sync::EncapsulatedLock;
use std::sync::{Arc, atomic::AtomicBool};
use wgpu::{
    CompositeAlphaMode, PresentMode, SurfaceConfiguration, SurfaceTarget, TextureFormat,
    TextureUsages, util::DeviceExt as _,
};

mod rendering;
pub mod ui_event;
mod winit_impls;

pub const FRAME_RATE: f32 = 30.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

struct WgpuData {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: SurfaceConfiguration,

    // for glyphon font demo
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: glyphon::Viewport,
    atlas: TextAtlas,
    // text_renderer: glyphon::TextRenderer,
    // text_buffer: glyphon::Buffer,

    // for wgpu
    rectangle_renderer: RectangleRenderer,
    image_renderer: ImageRenderer,
    char_grid_renderer: CharGridRenderer,

    vertex_buffer: wgpu::Buffer,
    // // index_buffer: wgpu::Buffer,
    // instances: Vec<RoundRectInstance>,
    // instance_buffer: wgpu::Buffer,
}
impl WgpuData {
    fn new(
        target: impl Into<SurfaceTarget<'static>>,
        physical_size: DisplayContainerSize,
        device: wgpu::Device,
        queue: wgpu::Queue,
        instance: &wgpu::Instance,
    ) -> Self {
        // Set up surface
        let surface = instance.create_surface(target).expect("Create surface");
        let swapchain_format = TextureFormat::Bgra8UnormSrgb;
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: PresentMode::Fifo,
            // is a simple way of dealing with transparency
            alpha_mode: CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Set up text renderer
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);
        // let text_renderer =
        //     TextRenderer::new(&mut atlas, &device, MultisampleState::default(), None);
        // let mut text_buffer = glyphon::Buffer::new(&mut font_system, Metrics::new(12.0, 12.0));

        // let physical_width = (physical_size.width as f64 * scale_factor) as f32;
        // let physical_height = (physical_size.height as f64 * scale_factor) as f32;

        // text_buffer.set_size(
        //     &mut font_system,
        //     Some(physical_width),
        //     Some(physical_height),
        // );
        // text_buffer.set_text(&mut font_system,
        //     "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q r s t u v w x y z",
        // &Attrs::new().family(Family::Monospace), Shaping::Advanced,None,);
        // text_buffer.shape_until_scroll(&mut font_system, false);

        // Set up gpu pipeline

        let rectangle_renderer = RectangleRenderer::new(&device, &surface_config);
        let image_renderer = ImageRenderer::new(&device, &surface_config);
        let char_grid_renderer = CharGridRenderer::new(&device, &surface_config, &queue);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(Vertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        // let instances = vec![
        //     RoundRectInstance {
        //         origin: [200.0, 200.0],
        //         size: [300.0, 150.0],
        //         corner_radius: 40.0,
        //         border_dist: 3.0,
        //         main_color: [0.5, 0.7, 0.5, 1.0],
        //         border_color: [0.2, 0.2, 0.2, 1.0],
        //     },
        //     // RoundRectInstance {
        //     //     origin: [400.0, 400.0],
        //     //     size: [50.0, 150.0],
        //     //     corner_radius: 20.0,
        //     //     border_dist: 3.0,
        //     //     main_color: [0.5, 0.7, 0.5, 1.0],
        //     //     border_color: [0.2, 0.2, 0.2, 1.0],
        //     // },
        // ];

        // let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&instances),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        Self {
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            // text_renderer,
            // text_buffer,
            rectangle_renderer,
            image_renderer,
            char_grid_renderer,
            vertex_buffer,
            // // index_buffer,
            // instances,
            // instance_buffer,
        }
    }
}

/// Data needed to connect to winit.
/// It comes from https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs
pub struct WinitData {
    wgpu_data: WgpuData,

    // Make sure that the winit window is last in the struct so that
    // it is dropped after the wgpu surface is dropped, otherwise the
    // program may crash when closed. This is probably a bug in wgpu.
    window: Arc<Window>,
}
impl WinitData {
    fn new(
        window: Arc<Window>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        instance: &wgpu::Instance,
    ) -> Self {
        let physical_size = window.inner_size();
        // let scale_factor = window.scale_factor();

        Self {
            wgpu_data: WgpuData::new(
                window.clone(),
                DisplayContainerSize::new(physical_size.width, physical_size.height),
                device,
                queue,
                instance,
            ),
            window,
        }
    }

    pub fn get_wgpu_data(&self) -> (wgpu::Device, wgpu::Queue) {
        (self.wgpu_data.device.clone(), self.wgpu_data.queue.clone())
    }
}

/// REVIEW: rename this
/// REVIEW: don't even expose this to pub?
/// I'm thinking I have the UISharedData standardized, and then it has a run function that depends on each backend
pub struct UIDisplay {
    is_running: Arc<AtomicBool>,
    event_queue: Sender<UIEvent>,
    ui_content: EncapsulatedLock<PrimitiveScene>,

    // width: u32,
    // height: u32,
    key_modifiers: KeyModifiers,

    device: wgpu::Device,
    queue: wgpu::Queue,
    instance: wgpu::Instance,

    winit_data: Option<WinitData>,
}
impl UIDisplay {
    // pub fn new<State: AsMut<Self>>(
    //     // TODO: with event loop, these don't need to be mutex and stuff
    //     is_running: Arc<AtomicBool>,
    //     event_queue: Sender<UIEvent>,
    //     ui_content: EncapsulatedLock<PrimitiveScene>,
    //     event_loop: &LoopHandle<State>,
    //     dh: &mut DisplayHandle,
    // ) -> Self {
    //     let builder = WindowAttributes::default()
    //         // .with_surface_size(LogicalSize::new(1280.0, 800.0))
    //         // .with_resizable(false)
    //         .with_title("sonamu");
    //     let (mut backend, winit_event_loop) =
    //         smithay::backend::winit::init_from_attributes::<GlesRenderer>(builder).unwrap();

    //     event_loop
    //         .insert_source(winit_event_loop, |event, (), state| {
    //             let ui_display = state.as_mut();
    //             ui_display.process_winit_event(event);
    //         })
    //         .unwrap();

    //     // backend.renderer().bind_wl_display(dh).unwrap();

    //     Self {
    //         is_running,
    //         event_queue,
    //         ui_content,
    //         // width: 256,
    //         // height: 256,
    //         key_modifiers: KeyModifiers::NONE,
    //         winit_data: None,
    //     }
    // }

    // fn process_winit_event(&mut self, event: WinitEvent) {
    //     if !self.is_running.load(std::sync::atomic::Ordering::Relaxed) {
    //         // event_loop.exit();
    //         return;
    //     }

    //     let Some(state) = &mut self.winit_data else {
    //         return;
    //     };

    //     let WinitData {
    //         window,
    //         wgpu_data:
    //             WgpuData {
    //                 device,
    //                 // queue,
    //                 surface,
    //                 surface_config,
    //                 ..
    //             },
    //     } = state;

    //     match event {
    //         WinitEvent::Resized { size, .. } => {
    //             surface_config.width = size.w.cast_unsigned();
    //             surface_config.height = size.h.cast_unsigned();
    //             surface.configure(device, surface_config);
    //             window.request_redraw();

    //             self.event_queue
    //                 .send(UIEvent::WindowResized(DisplayContainerSize::new(
    //                     size.w.cast_unsigned(),
    //                     size.h.cast_unsigned(),
    //                 )))
    //                 .unwrap();
    //         }
    //         WinitEvent::CloseRequested => {
    //             self.is_running
    //                 .store(false, std::sync::atomic::Ordering::Relaxed);
    //             // event_loop.exit();
    //         }
    //         WinitEvent::Focus(_) => {
    //             // self.ui_event_queue
    //             //     .lock()
    //             //     .unwrap()
    //             //     .push(crate::ui_event::UIEvent::Focused);
    //         }
    //         WinitEvent::Input(input_event) => match input_event {
    //             // smithay::backend::input::InputEvent::DeviceAdded { device } => todo!(),
    //             // smithay::backend::input::InputEvent::DeviceRemoved { device } => todo!(),
    //             smithay::backend::input::InputEvent::Keyboard { event } => {
    //                 dbg!(event.key_code());
    //             }
    //             // smithay::backend::input::InputEvent::PointerMotion { event } => todo!(),
    //             // smithay::backend::input::InputEvent::PointerMotionAbsolute { event } => todo!(),
    //             smithay::backend::input::InputEvent::PointerButton { event: _ } => {}
    //             // smithay::backend::input::InputEvent::PointerAxis { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GestureSwipeBegin { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GestureSwipeUpdate { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GestureSwipeEnd { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GesturePinchBegin { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GesturePinchUpdate { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GesturePinchEnd { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GestureHoldBegin { event } => todo!(),
    //             // smithay::backend::input::InputEvent::GestureHoldEnd { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TouchDown { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TouchMotion { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TouchUp { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TouchCancel { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TouchFrame { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TabletToolAxis { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TabletToolProximity { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TabletToolTip { event } => todo!(),
    //             // smithay::backend::input::InputEvent::TabletToolButton { event } => todo!(),
    //             // smithay::backend::input::InputEvent::SwitchToggle { event } => todo!(),
    //             // smithay::backend::input::InputEvent::Special(_) => todo!(),
    //             _ => {}
    //         },
    //         // WinitEvent::ModifiersChanged(modifiers) => {
    //         //     self.key_modifiers = modifiers.into();
    //         // }
    //         // WinitEvent::KeyboardInput {
    //         //     device_id: _,
    //         //     event,
    //         //     is_synthetic: _,
    //         // } => {
    //         //     if let Ok(key) = Key::try_from(event) {
    //         //         self.event_queue
    //         //             .send(super::ui_event::UIEvent::KeyPress(key, self.key_modifiers))
    //         //             .unwrap();
    //         //     }

    //         //     window.request_redraw();
    //         // }
    //         // winit::event::WindowEvent::MouseInput {
    //         //     device_id,
    //         //     state,
    //         //     button,
    //         // } => {
    //         //     println!("TODO: mouse press");
    //         // }
    //         WinitEvent::Redraw => {
    //             Self::draw(&mut self.winit_data, &self.ui_content.get());
    //         }
    //     }
    // }

    /// Returns when display is closed.
    pub fn run_display(
        is_running: Arc<AtomicBool>,
        event_queue: Sender<UIEvent>,
        ui_content: EncapsulatedLock<PrimitiveScene>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        instance: wgpu::Instance,
    ) {
        let event_loop = smithay::reexports::winit::event_loop::EventLoop::builder()
            .with_wayland()
            .with_any_thread(true)
            .build()
            .unwrap();
        let mut app = Self {
            is_running,
            event_queue,
            ui_content,
            // width: 256,
            // height: 256,
            key_modifiers: KeyModifiers::NONE,
            winit_data: None,
            device,
            queue,
            instance,
        };
        event_loop.run_app(&mut app).unwrap();

        // // All Wayland apps start by connecting the compositor (server).
        // let conn = Connection::connect_to_env().unwrap();

        // // Enumerate the list of globals to get the protocols the server implements.
        // let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        // let qh = event_queue.handle();
        // let mut event_loop: EventLoop<UIDisplay> =
        //     EventLoop::try_new().expect("Failed to initialize the event loop!");
        // let loop_handle = event_loop.handle();
        // WaylandSource::new(conn.clone(), event_queue)
        //     .insert(loop_handle)
        //     .unwrap();

        // // The compositor (not to be confused with the server which is commonly called the compositor) allows
        // // configuring surfaces to be presented.
        // let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        // // For desktop platforms, the XDG shell is the standard protocol for creating desktop windows.
        // let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");
        // // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
        // // we share with the compositor process.
        // let shm = Shm::bind(&globals, &qh).expect("wl shm is not available.");
        // // If the compositor supports xdg-activation it probably wants us to use it to get focus
        // let xdg_activation = ActivationState::bind(&globals, &qh).ok();

        // // A window is created from a surface.
        // let surface = compositor.create_surface(&qh);
        // // And then we can create the window.
        // let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
        // // Configure the window, this may include hints to the compositor about the desired minimum size of the
        // // window, app id for WM identification, the window title, etc.
        // window.set_title("A wayland window");
        // // GitHub does not let projects use the `org.github` domain but the `io.github` domain is fine.
        // window.set_app_id("io.github.smithay.client-toolkit.SimpleWindow");
        // window.set_min_size(Some((256, 256)));
        // window.set_maximized();

        // // In order for the window to be mapped, we need to perform an initial commit with no attached buffer.
        // // For more info, see WaylandSurface::commit
        // //
        // // The compositor will respond with an initial configure that we can then use to present to the window with
        // // the correct options.
        // window.commit();

        // // To request focus, we first need to request a token
        // if let Some(activation) = xdg_activation.as_ref() {
        //     activation.request_token(
        //         &qh,
        //         RequestData {
        //             seat_and_serial: None,
        //             surface: Some(window.wl_surface().clone()),
        //             app_id: Some(String::from(
        //                 "io.github.smithay.client-toolkit.SimpleWindow",
        //             )),
        //         },
        //     )
        // }

        // // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
        // // initial memory allocation.
        // let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

        // let mut ui_display = UIDisplay {
        //     root_element,
        //     ui_event_queue,

        //     // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
        //     // listen for seats and outputs.
        //     registry_state: RegistryState::new(&globals),
        //     seat_state: SeatState::new(&globals, &qh),
        //     output_state: OutputState::new(&globals, &qh),
        //     shm,
        //     xdg_activation,

        //     is_running,
        //     first_configure: true,
        //     pool,
        //     width: 256,
        //     height: 256,
        //     _shift: None,
        //     buffer: None,
        //     window,
        //     keyboard: None,
        //     key_modifiers: KeyModifiers::default(),
        //     pointer: None,
        //     loop_handle: event_loop.handle(),
        //     font: SystemSource::new()
        //         .select_best_match(
        //             &[font_kit::family_name::FamilyName::Monospace],
        //             font_kit::properties::Properties::new()
        //                 .weight(font_kit::properties::Weight::MEDIUM),
        //         )
        //         .unwrap()
        //         .load()
        //         .unwrap(),
        // };

        // // We don't draw immediately, the configure will notify us when to first draw.
        // while ui_display.is_running.load(Ordering::Relaxed) {
        //     event_loop
        //         .dispatch(
        //             Duration::from_secs_f32(FRAME_DELTA_SECONDS),
        //             &mut ui_display,
        //         )
        //         .unwrap();
        // }
        println!("Graciously ending display loop.");
    }
}
impl Drop for UIDisplay {
    fn drop(&mut self) {
        // TODO: is this even needed?
        // self.shared_data.set_ended();
        // let mut node_lock = self.node.lock();
        // loop {
        //     if let Some(mut shared_data) = self.shared_data.try_lock() {
        //         *shared_data = UIState::Ended;
        //         break;
        //     }
        //     node_lock.wait_for_update();
        // }
        // *self.shared_data.wait_lock(self.node.lock()) = UIState::Ended;

        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
