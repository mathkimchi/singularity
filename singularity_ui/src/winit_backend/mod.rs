//! An agnostic_backend to replace the wayland_backend.
//!
//! A bulk of this code originates from [Glyphon's hello world](https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs)
//! as well as the old wayland_backend.

use crate::{
    display_units::DisplayContainerSize,
    ui_element::UIElement,
    winit_backend::{
        rendering::{CharGridInstance, ImageInstance, RoundRectInstance, Vertex},
        ui_event::{KeyModifiers, UIEvent},
    },
};
use glyphon::{FontSystem, SwashCache, TextAtlas};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use wgpu::{
    CompositeAlphaMode, InstanceDescriptor, PipelineCompilationOptions, PresentMode,
    SurfaceConfiguration, SurfaceTarget, TextureFormat, TextureUsages, include_wgsl,
    util::DeviceExt as _,
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
    atlas: glyphon::TextAtlas,
    // text_renderer: glyphon::TextRenderer,
    // text_buffer: glyphon::Buffer,

    // for wgpu
    rectangle_render_pipeline: wgpu::RenderPipeline,
    image_render_pipeline: wgpu::RenderPipeline,
    image_texture_bind_group_layout: wgpu::BindGroupLayout,
    char_grid_render_pipeline: wgpu::RenderPipeline,
    char_grid_texture_bind_group_layout: wgpu::BindGroupLayout,

    vertex_buffer: wgpu::Buffer,
    // // index_buffer: wgpu::Buffer,
    // instances: Vec<RoundRectInstance>,
    // instance_buffer: wgpu::Buffer,
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

        let rectangle_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let rectangle_shader = device.create_shader_module(include_wgsl!("rectangle_shader.wgsl"));
        let rectangle_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Rectangle Render Pipeline"),
                layout: Some(&rectangle_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &rectangle_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), RoundRectInstance::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &rectangle_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // tells wgpu to render to just specific texture layers.
                multiview_mask: None,
                // Useful for optimizing shader compilation on Android
                cache: None,
            });

        let image_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("image_texture_bind_group_layout"),
            });
        let image_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    // Difference
                    Some(&image_texture_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let image_shader = device.create_shader_module(include_wgsl!("image_shader.wgsl"));
        let image_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Image Render Pipeline"),
                layout: Some(&image_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &image_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), ImageInstance::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &image_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // tells wgpu to render to just specific texture layers.
                multiview_mask: None,
                // Useful for optimizing shader compilation on Android
                cache: None,
            });

        let char_grid_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: wgpu::TextureFormat::Rgba32Uint,
                        // Hmm... no option for `texture_storage_2d`, hopefully this works
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
                label: Some("char_grid_texture_bind_group_layout"),
            });
        let char_grid_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    // Difference
                    Some(&char_grid_texture_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let char_grid_shader = device.create_shader_module(include_wgsl!("char_grid_shader.wgsl"));
        let char_grid_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Char Grid Render Pipeline"),
                layout: Some(&char_grid_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &char_grid_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), CharGridInstance::desc()],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &char_grid_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // tells wgpu to render to just specific texture layers.
                multiview_mask: None,
                // Useful for optimizing shader compilation on Android
                cache: None,
            });

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
            rectangle_render_pipeline,
            image_render_pipeline,
            image_texture_bind_group_layout,
            char_grid_render_pipeline,
            char_grid_texture_bind_group_layout,
            vertex_buffer,
            // // index_buffer,
            // instances,
            // instance_buffer,
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

/// REVIEW: rename this
pub struct UIDisplay {
    /// TODO: use `EncapsulatedLock`?
    root_element: Arc<Mutex<UIElement>>,

    /// TODO: just use mpsc
    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,

    /// REVIEW: Use `Arc<Mutex<bool>>`, `Arc<RwLock<bool>>`, or `Arc<AtomicBool>`?
    is_running: Arc<AtomicBool>,
    // width: u32,
    // height: u32,
    key_modifiers: KeyModifiers,

    winit_data: Option<WinitData>,
}
impl UIDisplay {
    /// Returns when display is closed.
    pub fn run_display(
        root_element: Arc<Mutex<UIElement>>,
        ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
        is_running: Arc<AtomicBool>,
    ) {
        let event_loop = EventLoop::builder()
            .with_wayland()
            .with_any_thread(true)
            .build()
            .unwrap();
        event_loop
            .run_app(&mut Self {
                root_element,
                ui_event_queue,
                is_running,
                // width: 256,
                // height: 256,
                key_modifiers: KeyModifiers::NONE,
                winit_data: None,
            })
            .unwrap();

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
        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
