//! An agnostic_backend to replace the wayland_backend.
//!
//! A bulk of this code originates from [Glyphon's hello world](https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs)
//! as well as the old wayland_backend.

use crate::{
    display_units::DisplayContainerSize,
    ui_element::UIElement,
    winit_backend::{
        rendering::{CharGridRenderer, ImageRenderer, RectangleRenderer, Vertex},
        ui_event::{KeyModifiers, UIEvent},
    },
};
use glyphon::{FontSystem, SwashCache, TextAtlas};
use sonamu_sync::shared_state::SharedData;
use std::{
    collections::VecDeque,
    sync::Arc,
};
use wgpu::{
    CompositeAlphaMode, InstanceDescriptor, PresentMode, SurfaceConfiguration, SurfaceTarget,
    TextureFormat, TextureUsages, util::DeviceExt as _,
};
use winit::{event_loop::EventLoop, platform::wayland::EventLoopBuilderExtWayland, window::Window};

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

    // for wgpu
    rectangle_renderer: RectangleRenderer,
    image_renderer: ImageRenderer,
    char_grid_renderer: CharGridRenderer,

    vertex_buffer: wgpu::Buffer,
}
impl WgpuData {
    async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        physical_size: DisplayContainerSize,
    ) -> Self {
        // Set up surface
        let instance = wgpu::Instance::new(InstanceDescriptor::new_without_display_handle());

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

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off, // Trace path
            })
            .await
            .unwrap();

        surface.configure(&device, &surface_config);

        // Set up text renderer
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(&device);
        let viewport = glyphon::Viewport::new(&device, &cache);
        let atlas = TextAtlas::new(&device, &queue, &cache, swapchain_format);

        // Set up gpu pipeline

        let rectangle_renderer = RectangleRenderer::new(&device, &surface_config);
        let image_renderer = ImageRenderer::new(&device, &surface_config);
        let char_grid_renderer = CharGridRenderer::new(&device, &surface_config, &queue);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(Vertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            font_system,
            swash_cache,
            viewport,
            atlas,
            rectangle_renderer,
            image_renderer,
            char_grid_renderer,
            vertex_buffer,
        }
    }
}

/// Data needed to connect to winit.
/// It comes from https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs
struct WinitData {
    wgpu_data: WgpuData,

    // Make sure that the winit window is last in the struct so that
    // it is dropped after the wgpu surface is dropped, otherwise the
    // program may crash when closed. This is probably a bug in wgpu.
    window: Arc<Window>,
}
impl WinitData {
    async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        // let scale_factor = window.scale_factor();

        Self {
            wgpu_data: WgpuData::new(
                window.clone(),
                DisplayContainerSize::new(physical_size.width, physical_size.height),
            )
            .await,
            window,
        }
    }
}

/// Look at 2026-07-25 devlog
/// This is the "edge" data shared between UI Display and the root applet logic.
/// Every time the state of this is changed by one of the threads, it should notify the other thread.
pub enum UIState {
    Running {
        root_element: UIElement,
        /// Individually, mpsc would be better than mutex vec dequeue, but with other things to be locked,
        /// this is much nicer to organize and probably faster.
        /// Same with the AtomicBool for is_running which is now just represented via the enum states
        ui_event_queue: VecDeque<UIEvent>,
    },
    /// I was going to use Option instead of manually naming the running vs ended, but I think this is clearer.
    Ended,
}

/// REVIEW: rename this
/// REVIEW: don't even expose this to pub?
/// I'm thinking I have the UISharedData standardized, and then it has a run function that depends on each backend
pub struct UIDisplay {
    shared_data: SharedData<UIState>,

    key_modifiers: KeyModifiers,

    winit_data: Option<WinitData>,
}
impl UIDisplay {
    /// Returns when display is closed.
    pub fn run_display(shared_data: SharedData<UIState>) {
        let event_loop = EventLoop::builder()
            .with_wayland()
            .with_any_thread(true)
            .build()
            .unwrap();
        event_loop
            .run_app(&mut Self {
                shared_data,
                key_modifiers: KeyModifiers::NONE,
                winit_data: None,
            })
            .unwrap();

        println!("Graciously ending display loop.");
    }
}
impl Drop for UIDisplay {
    fn drop(&mut self) {
        // TODO: is this even needed?
        *self.shared_data.lock_state() = UIState::Ended;
        self.shared_data.notify();
    }
}
