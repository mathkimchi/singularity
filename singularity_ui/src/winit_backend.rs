//! An agnostic_backend to replace the wayland_backend.
//!
//! A bulk of this code originates from [Glyphon's hello world](https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs)
//! as well as the old wayland_backend.

use crate::{
    ui_element::UIElement,
    winit_backend::ui_event::{KeyModifiers, UIEvent},
};
use glyphon::{FontSystem, SwashCache, TextAtlas};
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use wgpu::{
    CompositeAlphaMode, InstanceDescriptor, PresentMode, SurfaceConfiguration, TextureFormat,
    TextureUsages, include_wgsl, util::DeviceExt as _,
};
use winit::{event_loop::EventLoop, platform::wayland::EventLoopBuilderExtWayland, window::Window};

pub const FRAME_RATE: f32 = 30.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    // color: [f32; 3],
}
impl Vertex {
    // const ATTRIBS: [wgpu::VertexAttribute; 2] =
    //     wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    /// Large triangle trick to cover the whole screen
    /// https://webgpufundamentals.org/webgpu/lessons/webgpu-large-triangle-to-cover-clip-space.html
    const VERTICES: &[Vertex] = &[
        // A
        Vertex {
            position: [3., -1., 0.0],
            // color: [0.0, 0.0, 0.5],
        },
        // B
        Vertex {
            position: [-1., 3., 0.0],
            // color: [0.5, 0.5, 0.5],
        },
        // C
        Vertex {
            position: [-1., -1., 0.0],
            // color: [0.5, 0.0, 1.0],
        },
        // // D
        // Vertex {
        //     position: [0.35966998, -0.3473291, 0.0],
        //     color: [0.5, 0.0, 0.5],
        // },
        // // E
        // Vertex {
        //     position: [0.44147372, 0.2347359, 0.0],
        //     color: [0.0, 0.5, 0.5],
        // },
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct RoundRectInstance {
    /// This is the top left, not the center
    /// Currently includes the border
    origin: [f32; 2],
    /// Currently includes the border
    size: [f32; 2],
    /// NOTE: currently being ignored
    corner_radius: f32,
    border_dist: f32,
    main_color: [f32; 4],
    border_color: [f32; 4],
}
impl RoundRectInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32,
        4 => Float32,
        5 => Float32x4,
        6 => Float32x4
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
struct ImageInstance {
    /// This is the top left, not the center
    /// Currently includes the border
    origin: [f32; 2],
    /// Currently includes the border
    size: [f32; 2],
}
impl ImageInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Data needed to connect to winit.
/// It comes from https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs
struct WinitData {
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
    vertex_buffer: wgpu::Buffer,
    // // index_buffer: wgpu::Buffer,
    // instances: Vec<RoundRectInstance>,
    // instance_buffer: wgpu::Buffer,

    // Make sure that the winit window is last in the struct so that
    // it is dropped after the wgpu surface is dropped, otherwise the
    // program may crash when closed. This is probably a bug in wgpu.
    window: Arc<Window>,
}
impl WinitData {
    async fn new(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();
        // let scale_factor = window.scale_factor();

        // Set up surface
        let instance = wgpu::Instance::new(InstanceDescriptor::new_without_display_handle());

        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface");
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
                memory_hints: Default::default(),
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
                    compilation_options: Default::default(),
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
                    compilation_options: Default::default(),
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
                    compilation_options: Default::default(),
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
                    compilation_options: Default::default(),
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
            window,
            rectangle_render_pipeline,
            image_render_pipeline,
            image_texture_bind_group_layout,
            vertex_buffer,
            // // index_buffer,
            // instances,
            // instance_buffer,
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
            .run_app(&mut UIDisplay {
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

mod drawing_impls {
    use super::UIDisplay;
    use crate::{
        color::Color,
        display_units::{DisplayArea, DisplayCoord, DisplaySize, DisplayUnits},
        ui_element::{CharCell, UIElement},
        winit_backend::{ImageInstance, RoundRectInstance, Vertex, WinitData},
    };
    use glyphon::{Metrics, TextRenderer};
    use image::{ImageBuffer, Rgba};
    use std::iter;
    use wgpu::{BindGroupLayout, MultisampleState, SurfaceConfiguration, util::DeviceExt as _};

    /// Data needed for drawing
    struct DrawingSharedData<'a> {
        render_pass: wgpu::RenderPass<'a>,

        rectangle_render_pipeline: &'a wgpu::RenderPipeline,
        image_render_pipeline: &'a wgpu::RenderPipeline,
        image_texture_bind_group_layout: &'a BindGroupLayout,

        /// Just one large triangle
        vertex_buffer: &'a wgpu::Buffer,

        font_system: &'a mut glyphon::FontSystem,
        swash_cache: &'a mut glyphon::SwashCache,
        viewport: &'a mut glyphon::Viewport,
        atlas: &'a mut glyphon::TextAtlas,
        // text_renderer: &'a mut glyphon::TextRenderer,
        // text_buffer: &'a mut glyphon::Buffer,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        // surface: &'a wgpu::Surface<'static>,
        surface_config: &'a SurfaceConfiguration,
    }

    impl UIElement {
        fn fill_rect(
            drawing_shared_data: &mut DrawingSharedData,
            area: DisplayArea,
            radius: f32,
            border_dist: f32,
            inner_color: Color,
            border_color: Color,
        ) {
            drawing_shared_data
                .render_pass
                .set_pipeline(drawing_shared_data.rectangle_render_pipeline);
            // these buffers are how we pass data to the gpu
            drawing_shared_data
                .render_pass
                .set_vertex_buffer(0, drawing_shared_data.vertex_buffer.slice(..));

            let instances = vec![RoundRectInstance {
                // this currently takes in top left
                origin: [
                    area.0
                        .x
                        .pixels(drawing_shared_data.surface_config.width as _)
                        as _,
                    area.0
                        .y
                        .pixels(drawing_shared_data.surface_config.height as _)
                        as _,
                ],
                size: [
                    area.size()
                        .width
                        .pixels(drawing_shared_data.surface_config.width as _)
                        as _,
                    area.size()
                        .height
                        .pixels(drawing_shared_data.surface_config.height as _)
                        as _,
                ],
                corner_radius: radius,
                border_dist,
                main_color: inner_color.0.map(|c| c as f32 / u8::MAX as f32),
                // shouldn't matter
                border_color: border_color.0.map(|c| c as f32 / u8::MAX as f32),
            }];

            let instance_buffer =
                drawing_shared_data
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            drawing_shared_data
                .render_pass
                .set_vertex_buffer(1, instance_buffer.slice(..));
            // render_pass
            //     .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
            drawing_shared_data
                .render_pass
                .draw(0..Vertex::VERTICES.len() as _, 0..1); // 1 bc we only draw 1 rect at a time (which I am not happy about)

            // let mut pb = raqote::PathBuilder::new();
            // pb.rect(
            //     area.0.x.pixels(dt.width()) as f32,
            //     area.0.y.pixels(dt.height()) as f32,
            //     area.size().width.pixels(dt.width()) as f32,
            //     area.size().height.pixels(dt.height()) as f32,
            // );
            // let path = pb.finish();
            // dt.fill(&path, &Source::Solid(color.into()), &DrawOptions::new());
        }

        fn draw_image(
            drawing_shared_data: &mut DrawingSharedData,
            image_buffer: &ImageBuffer<Rgba<u8>, Vec<u8>>,
            area: DisplayArea,
        ) {
            drawing_shared_data
                .render_pass
                .set_pipeline(drawing_shared_data.image_render_pipeline);

            // set bind group (which holds the image texture)
            {
                let width = image_buffer.width();
                let height = image_buffer.height();

                let texture_size = wgpu::Extent3d {
                    width,
                    height,
                    // All textures are stored as 3D, we represent our 2D texture
                    // by setting depth to 1.
                    depth_or_array_layers: 1,
                };

                let diffuse_texture =
                    drawing_shared_data
                        .device
                        .create_texture(&wgpu::TextureDescriptor {
                            size: texture_size,
                            mip_level_count: 1, // We'll talk about this a little later
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            // Most images are stored using sRGB, so we need to reflect that here.
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                            // COPY_DST means that we want to copy data to this texture
                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_DST,
                            label: Some("diffuse_texture"),
                            // This is the same as with the SurfaceConfig. It
                            // specifies what texture formats can be used to
                            // create TextureViews for this texture. The base
                            // texture format (Rgba8UnormSrgb in this case) is
                            // always supported. Note that using a different
                            // texture format is not supported on the WebGL2
                            // backend.
                            view_formats: &[],
                        });

                drawing_shared_data.queue.write_texture(
                    // Tells wgpu where to copy the pixel data
                    wgpu::TexelCopyTextureInfo {
                        texture: &diffuse_texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    // The actual pixel data
                    image_buffer,
                    // The layout of the texture
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * width),
                        rows_per_image: Some(height),
                    },
                    texture_size,
                );

                // We don't need to configure the texture view much, so let's
                // let wgpu define it.
                let diffuse_texture_view =
                    diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let diffuse_sampler =
                    drawing_shared_data
                        .device
                        .create_sampler(&wgpu::SamplerDescriptor {
                            address_mode_u: wgpu::AddressMode::ClampToEdge,
                            address_mode_v: wgpu::AddressMode::ClampToEdge,
                            address_mode_w: wgpu::AddressMode::ClampToEdge,
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Nearest,
                            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                            ..Default::default()
                        });

                let diffuse_bind_group =
                    drawing_shared_data
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &drawing_shared_data.image_texture_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::TextureView(
                                        &diffuse_texture_view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                                },
                            ],
                            label: Some("diffuse_bind_group"),
                        });
                drawing_shared_data
                    .render_pass
                    .set_bind_group(0, Some(&diffuse_bind_group), &[]);
            }

            // these buffers are how we pass data to the gpu
            // pass in the large triangle
            drawing_shared_data
                .render_pass
                .set_vertex_buffer(0, drawing_shared_data.vertex_buffer.slice(..));

            // set instance buffer
            {
                let instances = vec![ImageInstance {
                    // this currently takes in top left
                    origin: [
                        area.0
                            .x
                            .pixels(drawing_shared_data.surface_config.width as _)
                            as _,
                        area.0
                            .y
                            .pixels(drawing_shared_data.surface_config.height as _)
                            as _,
                    ],
                    size: [
                        area.size()
                            .width
                            .pixels(drawing_shared_data.surface_config.width as _)
                            as _,
                        area.size()
                            .height
                            .pixels(drawing_shared_data.surface_config.height as _)
                            as _,
                    ],
                }];

                let instance_buffer = drawing_shared_data.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    },
                );

                // vertex buffer slot 1 is actually the instance buffer
                drawing_shared_data
                    .render_pass
                    .set_vertex_buffer(1, instance_buffer.slice(..));
            }
            drawing_shared_data
                .render_pass
                .draw(0..Vertex::VERTICES.len() as _, 0..1); // 1 bc we only draw 1 image at a time (which I am not happy about)
        }

        // Surface config used just for the width
        fn display_area_to_text_bounds(
            area: DisplayArea,
            surface_config: &SurfaceConfiguration,
        ) -> glyphon::TextBounds {
            glyphon::TextBounds {
                left: area.0.x.pixels(surface_config.width as _),
                top: area.0.y.pixels(surface_config.height as _),
                // TODO
                right: area.1.x.pixels(surface_config.width as _),
                bottom: area.1.y.pixels(surface_config.height as _),
            }
        }

        fn draw(&self, drawing_shared_data: &mut DrawingSharedData, container_area: DisplayArea) {
            /// think this is height in pixels
            const FONT_SIZE: i32 = 24;
            const FONT_SIZE_F: f32 = FONT_SIZE as f32;

            match self {
                UIElement::Container(children) => {
                    for ui_element in children {
                        // draw the inner widget
                        ui_element.draw(drawing_shared_data, container_area);
                    }
                }
                UIElement::Contained(inner_element, area) => {
                    inner_element.draw(drawing_shared_data, area.map_onto(container_area));
                }
                // FIXME: there are weird border lines
                UIElement::Bordered(inner_element, border_color) => {
                    // // draw the border
                    // let border_path = {
                    //     let mut pb = raqote::PathBuilder::new();
                    //     // top
                    //     pb.rect(
                    //         container_area.0.x.pixels(dt.width()) as f32,
                    //         container_area.0.y.pixels(dt.height()) as f32,
                    //         container_area.size().width.pixels(dt.width()) as f32,
                    //         1.,
                    //     );
                    //     // bot
                    //     pb.rect(
                    //         container_area.0.x.pixels(dt.width()) as f32 - 1.,
                    //         container_area.1.y.pixels(dt.height()) as f32 - 1.,
                    //         container_area.size().width.pixels(dt.width()) as f32 + 1.,
                    //         // NOTE: ^ the bottom right pixel is gone without this + 1. (both are needed for some reason)
                    //         1.,
                    //     );
                    //     // left
                    //     pb.rect(
                    //         container_area.0.x.pixels(dt.width()) as f32,
                    //         container_area.0.y.pixels(dt.height()) as f32,
                    //         1.,
                    //         container_area.size().height.pixels(dt.height()) as f32,
                    //     );
                    //     // right
                    //     pb.rect(
                    //         container_area.1.x.pixels(dt.width()) as f32 - 1.,
                    //         container_area.0.y.pixels(dt.height()) as f32 - 1.,
                    //         1.,
                    //         container_area.size().height.pixels(dt.height()) as f32 + 1.,
                    //         // NOTE: ^ the bottom right pixel is gone without this + 1. (both are needed for some reason)
                    //     );
                    //     pb.finish()
                    // };
                    // dt.fill(
                    //     &border_path,
                    //     &Source::Solid((*border_color).into()),
                    //     &DrawOptions::new(),
                    // );

                    UIElement::fill_rect(
                        drawing_shared_data,
                        container_area,
                        1.0,
                        1.0,
                        Color::TRANSPARENT,
                        *border_color,
                    );

                    let inner_area = DisplayArea(
                        DisplayCoord::new(1.into(), 1.into()),
                        DisplayCoord::new(
                            DisplayUnits::from_mixed(-1, 1.0),
                            DisplayUnits::from_mixed(-1, 1.0),
                        ),
                    )
                    .map_onto(container_area);

                    // dbg!(&container_area);
                    // dbg!(&container_area.size());
                    // dbg!(&inner_area);

                    // draw the inner widget
                    inner_element.draw(drawing_shared_data, inner_area);
                }
                UIElement::Backgrounded(inner_element, bg_color) => {
                    // clear the inside of the border
                    Self::fill_rect(
                        drawing_shared_data,
                        container_area,
                        0.,
                        0.,
                        *bg_color,
                        // shouldn't matter
                        Color::BLACK,
                    );

                    // draw the inner widget
                    inner_element.draw(drawing_shared_data, container_area);
                }
                UIElement::Text(text) => {
                    let mut text_buffer = glyphon::Buffer::new(
                        drawing_shared_data.font_system,
                        Metrics::new(FONT_SIZE_F, FONT_SIZE_F),
                    );
                    text_buffer.set_rich_text(
                        drawing_shared_data.font_system,
                        text.iter().map(|(s, attr)| (s.as_str(), attr.as_attrs())),
                        &glyphon::Attrs::new().family(glyphon::Family::Monospace),
                        glyphon::Shaping::Advanced,
                        None,
                    );

                    text_buffer.shape_until_scroll(drawing_shared_data.font_system, false);

                    let mut text_renderer = TextRenderer::new(
                        drawing_shared_data.atlas,
                        drawing_shared_data.device,
                        MultisampleState::default(),
                        None,
                    );

                    text_renderer
                        .prepare(
                            drawing_shared_data.device,
                            drawing_shared_data.queue,
                            drawing_shared_data.font_system,
                            drawing_shared_data.atlas,
                            drawing_shared_data.viewport,
                            [glyphon::TextArea {
                                buffer: &text_buffer,
                                left: container_area
                                    .0
                                    .x
                                    .pixels(drawing_shared_data.surface_config.width as _)
                                    as _,
                                top: container_area
                                    .0
                                    .y
                                    .pixels(drawing_shared_data.surface_config.height as _)
                                    as _,
                                scale: 1.0,
                                bounds: Self::display_area_to_text_bounds(
                                    container_area,
                                    drawing_shared_data.surface_config,
                                ),
                                default_color: glyphon::Color::rgb(255, 255, 255),
                                custom_glyphs: &[],
                            }],
                            drawing_shared_data.swash_cache,
                        )
                        .unwrap();

                    text_renderer
                        .render(
                            drawing_shared_data.atlas,
                            drawing_shared_data.viewport,
                            &mut drawing_shared_data.render_pass,
                        )
                        .unwrap();

                    // // FIXME: doesn't work with space
                    // dt.draw_text(
                    //     font,
                    //     FONT_SIZE as f32,
                    //     text,
                    //     DisplayCoord::new(
                    //         container_area.0.x,
                    //         container_area.0.y + FONT_SIZE.into(),
                    //     )
                    //     .into_raqote_point(dt),
                    //     &Source::Solid(SolidSource {
                    //         r: 0,
                    //         g: 0xFF,
                    //         b: 0xFF,
                    //         a: 0xFF,
                    //     }),
                    //     &DrawOptions::new(),
                    // );
                }
                UIElement::CharGrid(char_grid) => {
                    for (line_index, line) in char_grid.content.iter().enumerate() {
                        for (col_index, CharCell { character, fg, bg }) in line.iter().enumerate() {
                            let top_left = DisplayCoord::new(
                                container_area.0.x
                                    + DisplayUnits::Pixels(FONT_SIZE / 2 * (col_index as i32)),
                                container_area.0.y
                                    + DisplayUnits::Pixels(FONT_SIZE * (line_index as i32) + 1),
                            );

                            if !container_area.contains(
                                top_left,
                                [
                                    drawing_shared_data.viewport.resolution().width as i32,
                                    drawing_shared_data.viewport.resolution().height as i32,
                                ],
                            ) {
                                // FIXME: not completely foolproof -- main purpose is just optimization
                                continue;
                            }

                            // let bot_right = DisplayCoord::new(
                            //     container_area.0.x
                            //         + DisplayUnits::Pixels(
                            //             FONT_SIZE / 2 * ((col_index + 1) as i32),
                            //         ),
                            //     container_area.0.y
                            //         + DisplayUnits::Pixels(FONT_SIZE * (line_index + 1) as i32),
                            // );

                            Self::fill_rect(
                                drawing_shared_data,
                                DisplayArea::from_corner_size(
                                    top_left,
                                    DisplaySize::new(
                                        (FONT_SIZE / 2 + 1).into(),
                                        (FONT_SIZE + 2).into(),
                                    ),
                                ),
                                0.,
                                // Set to 1 for dbg boxes
                                0.,
                                *bg,
                                Color::TRANSPARENT,
                            );

                            if character == &' ' {
                                continue;
                            }

                            let mut text_buffer = glyphon::Buffer::new(
                                drawing_shared_data.font_system,
                                Metrics::new(FONT_SIZE_F, FONT_SIZE_F),
                            );

                            // text_buffer.set_size(
                            //     &mut drawing_shared_data.font_system,
                            //     Some(physical_width),
                            //     Some(physical_height),
                            // );

                            text_buffer.set_text(
                                drawing_shared_data.font_system,
                                character.to_string().as_str(),
                                &glyphon::Attrs::new().family(glyphon::Family::Monospace),
                                glyphon::Shaping::Advanced,
                                None,
                            );
                            text_buffer.shape_until_scroll(drawing_shared_data.font_system, false);

                            let mut text_renderer = TextRenderer::new(
                                drawing_shared_data.atlas,
                                drawing_shared_data.device,
                                MultisampleState::default(),
                                None,
                            );

                            text_renderer
                                .prepare(
                                    drawing_shared_data.device,
                                    drawing_shared_data.queue,
                                    drawing_shared_data.font_system,
                                    drawing_shared_data.atlas,
                                    drawing_shared_data.viewport,
                                    [glyphon::TextArea {
                                        buffer: &text_buffer,
                                        left: top_left
                                            .x
                                            .pixels(drawing_shared_data.surface_config.width as _)
                                            as _,
                                        top: top_left
                                            .y
                                            .pixels(drawing_shared_data.surface_config.height as _)
                                            as _,
                                        scale: 1.0,
                                        bounds: Self::display_area_to_text_bounds(
                                            container_area,
                                            drawing_shared_data.surface_config,
                                        ),
                                        default_color: glyphon::Color::rgb(
                                            fg.0[0], fg.0[1], fg.0[0],
                                        ),
                                        custom_glyphs: &[],
                                    }],
                                    drawing_shared_data.swash_cache,
                                )
                                .unwrap();

                            text_renderer
                                .render(
                                    drawing_shared_data.atlas,
                                    drawing_shared_data.viewport,
                                    &mut drawing_shared_data.render_pass,
                                )
                                .unwrap();

                            // drawing_shared_data.queue.submit(Some(encoder.finish()));
                            // drawing_shared_data.frame.present();

                            // drawing_shared_data.atlas.trim();

                            // dt.draw_text(
                            //     font,
                            //     FONT_SIZE as f32,
                            //     &character.to_string(),
                            //     // `start` is actually bottom left corner
                            //     bot_left.into_raqote_point(dt),
                            //     &raqote::Source::Solid((*fg).into()),
                            //     &DrawOptions::new(),
                            // );
                        }
                    }
                }
                UIElement::Image(image_buffer) => {
                    Self::draw_image(drawing_shared_data, image_buffer, container_area);
                }
                UIElement::Nothing => {}
            }
        }

        /*
        fn fill_rect(dt: &mut DrawTarget, area: DisplayArea, color: Color) {
            let mut pb = raqote::PathBuilder::new();
            pb.rect(
                area.0.x.pixels(dt.width()) as f32,
                area.0.y.pixels(dt.height()) as f32,
                area.size().width.pixels(dt.width()) as f32,
                area.size().height.pixels(dt.height()) as f32,
            );
            let path = pb.finish();
            dt.fill(&path, &Source::Solid(color.into()), &DrawOptions::new());
        }

        fn draw(&self, dt: &mut DrawTarget, container_area: DisplayArea, font: &Font) {
            /// think this is height in pixels
            const FONT_SIZE: i32 = 12;

            match self {
                UIElement::Container(children) => {
                    for ui_element in children {
                        // draw the inner widget
                        ui_element.draw(dt, container_area, font);
                    }
                }
                UIElement::Contained(inner_element, area) => {
                    inner_element.draw(dt, area.map_onto(container_area), font);
                }
                // FIXME: there are weird border lines
                UIElement::Bordered(inner_element, border_color) => {
                    // draw the border
                    let border_path = {
                        let mut pb = raqote::PathBuilder::new();
                        // top
                        pb.rect(
                            container_area.0.x.pixels(dt.width()) as f32,
                            container_area.0.y.pixels(dt.height()) as f32,
                            container_area.size().width.pixels(dt.width()) as f32,
                            1.,
                        );
                        // bot
                        pb.rect(
                            container_area.0.x.pixels(dt.width()) as f32 - 1.,
                            container_area.1.y.pixels(dt.height()) as f32 - 1.,
                            container_area.size().width.pixels(dt.width()) as f32 + 1.,
                            // NOTE: ^ the bottom right pixel is gone without this + 1. (both are needed for some reason)
                            1.,
                        );
                        // left
                        pb.rect(
                            container_area.0.x.pixels(dt.width()) as f32,
                            container_area.0.y.pixels(dt.height()) as f32,
                            1.,
                            container_area.size().height.pixels(dt.height()) as f32,
                        );
                        // right
                        pb.rect(
                            container_area.1.x.pixels(dt.width()) as f32 - 1.,
                            container_area.0.y.pixels(dt.height()) as f32 - 1.,
                            1.,
                            container_area.size().height.pixels(dt.height()) as f32 + 1.,
                            // NOTE: ^ the bottom right pixel is gone without this + 1. (both are needed for some reason)
                        );
                        pb.finish()
                    };
                    dt.fill(
                        &border_path,
                        &Source::Solid((*border_color).into()),
                        &DrawOptions::new(),
                    );

                    let inner_area = DisplayArea(
                        DisplayCoord::new(1.into(), 1.into()),
                        DisplayCoord::new(
                            DisplayUnits::from_mixed(-1, 1.0),
                            DisplayUnits::from_mixed(-1, 1.0),
                        ),
                    )
                    .map_onto(container_area);

                    // dbg!(&container_area);
                    // dbg!(&container_area.size());
                    // dbg!(&inner_area);

                    // draw the inner widget
                    inner_element.draw(dt, inner_area, font);
                }
                UIElement::Backgrounded(inner_element, bg_color) => {
                    // clear the inside of the border
                    Self::fill_rect(dt, container_area, *bg_color);

                    // draw the inner widget
                    inner_element.draw(dt, container_area, font);
                }
                UIElement::Text(text) => {
                    // FIXME: doesn't work with space
                    dt.draw_text(
                        font,
                        FONT_SIZE as f32,
                        text,
                        DisplayCoord::new(
                            container_area.0.x,
                            container_area.0.y + FONT_SIZE.into(),
                        )
                        .into_raqote_point(dt),
                        &Source::Solid(SolidSource {
                            r: 0,
                            g: 0xFF,
                            b: 0xFF,
                            a: 0xFF,
                        }),
                        &DrawOptions::new(),
                    );
                }
                UIElement::CharGrid(char_grid) => {
                    for (line_index, line) in char_grid.content.iter().enumerate() {
                        for (col_index, CharCell { character, fg, bg }) in line.iter().enumerate() {
                            let top_left = DisplayCoord::new(
                                container_area.0.x
                                    + DisplayUnits::Pixels(FONT_SIZE / 2 * (col_index as i32)),
                                container_area.0.y
                                    + DisplayUnits::Pixels(FONT_SIZE * (line_index as i32) + 1),
                            );

                            if !container_area.contains(top_left, [dt.width(), dt.height()]) {
                                // FIXME: not completely foolproof -- main purpose is just optimization
                                continue;
                            }

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
                                font,
                                FONT_SIZE as f32,
                                &character.to_string(),
                                // `start` is actually bottom left corner
                                bot_left.into_raqote_point(dt),
                                &raqote::Source::Solid((*fg).into()),
                                &DrawOptions::new(),
                            );
                        }
                    }
                }
                UIElement::Nothing => {}
            }
        }
        */
    }

    impl UIDisplay {
        pub fn draw(&mut self) {
            let Some(state) = &mut self.winit_data else {
                return;
            };

            let WinitData {
                // window,
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
                vertex_buffer,
                // index_buffer,
                // instance_buffer,
                // instances,
                ..
            } = state;

            let output = match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
                // TODO
                _ => panic!(),
            };
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // I don't really understand the other junk here,
                            // but this is the background
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                // if this isn't opaque, weird artifacts appear
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                // tell it the resolution, which shouldn't change for the rest of the draw calls
                viewport.update(
                    queue,
                    glyphon::Resolution {
                        width: surface_config.width,
                        height: surface_config.height,
                    },
                );

                let mut drawing_shared_data = DrawingSharedData {
                    render_pass,
                    rectangle_render_pipeline,
                    image_render_pipeline,
                    image_texture_bind_group_layout,
                    vertex_buffer,
                    device,
                    queue,
                    // surface,
                    surface_config,
                    font_system,
                    swash_cache,
                    viewport,
                    atlas,
                    // text_renderer,
                    // text_buffer,
                };

                self.root_element
                    .lock()
                    .unwrap()
                    .draw(&mut drawing_shared_data, DisplayArea::FULL);
            }

            queue.submit(iter::once(encoder.finish()));
            output.present();

            // let stride = self.width as i32 * 4;

            // let buffer = self.buffer.get_or_insert_with(|| {
            //     self.pool
            //         .create_buffer(
            //             self.width as i32,
            //             self.height as i32,
            //             stride,
            //             wl_shm::Format::Argb8888,
            //         )
            //         .expect("create buffer")
            //         .0
            // });

            // let canvas = match self.pool.canvas(buffer) {
            //     Some(canvas) => canvas,
            //     None => {
            //         // This should be rare, but if the compositor has not released the previous
            //         // buffer, we need double-buffering.
            //         let (second_buffer, canvas) = self
            //             .pool
            //             .create_buffer(
            //                 self.width as i32,
            //                 self.height as i32,
            //                 stride,
            //                 wl_shm::Format::Argb8888,
            //             )
            //             .expect("create buffer");
            //         *buffer = second_buffer;
            //         canvas
            //     }
            // };

            // // Draw to the window:
            // // FIXME find an actual fix to the height difference
            // if canvas.len() as u32 == 4 * self.width * self.height {
            //     let mut dt = DrawTarget::new(self.width as i32, self.height as i32);
            //     self.root_element
            //         .lock()
            //         .unwrap()
            //         .draw(&mut dt, DisplayArea::FULL, &self.font);
            //     canvas.copy_from_slice(dt.get_data_u8());
            // }

            // // Damage the entire window
            // self.window
            //     .wl_surface()
            //     .damage_buffer(0, 0, self.width as i32, self.height as i32);

            // // Request our next frame
            // self.window
            //     .wl_surface()
            //     .frame(qh, self.window.wl_surface().clone());

            // // Attach and commit to present.
            // buffer
            //     .attach_to(self.window.wl_surface())
            //     .expect("buffer attach");
            // self.window.commit();
        }
    }
}
mod winit_impls {
    use crate::{
        ui_event::Key,
        winit_backend::{UIDisplay, WinitData},
    };
    use std::sync::Arc;
    use winit::{dpi::LogicalSize, window::Window};

    impl winit::application::ApplicationHandler for UIDisplay {
        fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
            if self.winit_data.is_some() {
                return;
            }

            // Set up window
            let window_attributes = Window::default_attributes()
                .with_inner_size(LogicalSize::new(800, 600))
                .with_title("Singularity");
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            self.winit_data = Some(pollster::block_on(WinitData::new(window)));
        }

        fn window_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: winit::event::WindowEvent,
        ) {
            if !self.is_running.load(std::sync::atomic::Ordering::Relaxed) {
                event_loop.exit();
                return;
            }

            let Some(state) = &mut self.winit_data else {
                return;
            };

            let WinitData {
                window,
                device,
                // queue,
                surface,
                surface_config,
                // font_system,
                // swash_cache,
                // viewport,
                // atlas,
                // text_renderer,
                // text_buffer,
                // render_pipeline,
                // vertex_buffer,
                // // index_buffer,
                // instance_buffer,
                // instances,
                ..
            } = state;

            match event {
                winit::event::WindowEvent::Resized(size) => {
                    surface_config.width = size.width;
                    surface_config.height = size.height;
                    surface.configure(device, surface_config);
                    window.request_redraw();

                    self.ui_event_queue.lock().unwrap().push(
                        crate::ui_event::UIEvent::WindowResized([size.width, size.height]),
                    );
                }
                winit::event::WindowEvent::CloseRequested => {
                    self.is_running
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    event_loop.exit()
                }
                // winit::event::WindowEvent::Focused(focus) => self.ui_event_queue.lock().unwrap().push(crate::ui_event::UIEvent::Focused),
                winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                    self.key_modifiers = modifiers.into();
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } => {
                    if let Ok(key) = Key::try_from(event) {
                        self.ui_event_queue
                            .lock()
                            .unwrap()
                            .push(super::ui_event::UIEvent::KeyPress(key, self.key_modifiers));
                    }

                    window.request_redraw();
                }
                // winit::event::WindowEvent::MouseInput {
                //     device_id,
                //     state,
                //     button,
                // } => {
                //     println!("TODO: mouse press");
                // }
                winit::event::WindowEvent::RedrawRequested => {
                    // viewport.update(
                    //     queue,
                    //     Resolution {
                    //         width: surface_config.width,
                    //         height: surface_config.height,
                    //     },
                    // );

                    // // queue.write_buffer(, offset, data);

                    // text_renderer
                    //     .prepare(
                    //         device,
                    //         queue,
                    //         font_system,
                    //         atlas,
                    //         viewport,
                    //         [TextArea {
                    //             buffer: text_buffer,
                    //             left: 10.0,
                    //             top: 10.0,
                    //             scale: 1.0,
                    //             bounds: TextBounds {
                    //                 left: 0,
                    //                 top: 0,
                    //                 right: 600,
                    //                 bottom: 160,
                    //             },
                    //             default_color: Color::rgb(255, 255, 255),
                    //             custom_glyphs: &[],
                    //         }],
                    //         swash_cache,
                    //     )
                    //     .unwrap();

                    // let frame = surface.get_current_texture().unwrap();
                    // let view = frame.texture.create_view(&TextureViewDescriptor::default());
                    // let mut encoder =
                    //     device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                    // {
                    //     let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    //         label: None,
                    //         color_attachments: &[Some(RenderPassColorAttachment {
                    //             view: &view,
                    //             depth_slice: None,
                    //             resolve_target: None,
                    //             ops: Operations {
                    //                 load: LoadOp::Clear(wgpu::Color::BLACK),
                    //                 store: wgpu::StoreOp::Store,
                    //             },
                    //         })],
                    //         depth_stencil_attachment: None,
                    //         timestamp_writes: None,
                    //         occlusion_query_set: None,
                    //         multiview_mask: None,
                    //     });

                    //     text_renderer.render(atlas, viewport, &mut pass).unwrap();
                    // }

                    // queue.submit(Some(encoder.finish()));
                    // frame.present();

                    // atlas.trim();

                    // // We can't render unless the surface is configured
                    // if !self.is_surface_configured {
                    //     return Ok(());
                    // }

                    // let output = surface.get_current_texture().unwrap();
                    // let view = output
                    //     .texture
                    //     .create_view(&wgpu::TextureViewDescriptor::default());

                    // let mut encoder =
                    //     device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    //         label: Some("Render Encoder"),
                    //     });

                    // {
                    //     let mut render_pass =
                    //         encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    //             label: Some("Render Pass"),
                    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    //                 view: &view,
                    //                 resolve_target: None,
                    //                 ops: wgpu::Operations {
                    //                     // I don't really understand the other junk here,
                    //                     // but this is the background
                    //                     load: wgpu::LoadOp::Clear(wgpu::Color {
                    //                         r: 0.0,
                    //                         g: 0.0,
                    //                         b: 0.0,
                    //                         // if this isn't opaque, weird artifacts appear
                    //                         a: 1.0,
                    //                     }),
                    //                     store: wgpu::StoreOp::Store,
                    //                 },
                    //                 depth_slice: None,
                    //             })],
                    //             depth_stencil_attachment: None,
                    //             occlusion_query_set: None,
                    //             timestamp_writes: None,
                    //             multiview_mask: None,
                    //         });

                    //     render_pass.set_pipeline(render_pipeline);
                    //     render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    //     render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    //     // render_pass
                    //     //     .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    //     // render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
                    //     render_pass.draw(0..super::VERTICES.len() as _, 0..instances.len() as _);
                    // }

                    // queue.submit(std::iter::once(encoder.finish()));
                    // output.present();

                    self.draw();
                }
                _ => {}
            }
        }
    }
}
pub mod ui_event {
    use crate::display_units::DisplayArea;

    /// TODO: not great that I am reexporting smithay's event, given that the goal is to be backend agnostic.
    /// I am doing it right now because I'd rather get something working sooner, even if I have to compromise a bit
    ///
    /// TODO: also, figure out a way to easily match keypresses and shortcuts
    ///
    /// TODO: figure out a standard way of "forwarding" events to child
    #[derive(Debug, Clone)]
    pub enum UIEvent {
        KeyPress(Key, KeyModifiers),
        WindowResized([u32; 2]),
        /// ([mouse location [x, y], window size [w h]], container)
        ///
        /// REVIEW: definitely redundant, but might be helpful?
        ///
        /// NOTE: container should always be FULL for the outermost, but is helpful when trying to forward it to children:
        /// the forwarded area should be: `child_area.map_onto(parent_area)`
        MousePress([[u32; 2]; 2], DisplayArea),
    }
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct KeyModifiers {
        pub ctrl: bool,
        pub alt: bool,
        pub shift: bool,
        pub caps_lock: bool,
        pub logo: bool,
        // pub num_lock: bool,
    }
    #[derive(Debug, Clone)]
    pub enum Key {
        ArrowKeyUp,
        ArrowKeyDown,
        ArrowKeyLeft,
        ArrowKeyRight,
        Enter,
        Backspace,
        Char(char),
    }

    /// TODO: Get rid of keytrait and just implement directly?
    pub trait KeyTrait {
        fn to_alphabet(&self) -> Option<char>;
        fn to_digit(&self) -> Option<u8>;
        fn to_char(&self) -> Option<char>;
    }
    impl KeyTrait for Key {
        fn to_alphabet(&self) -> Option<char> {
            let c = self.to_char()?;
            if c.is_ascii() { Some(c) } else { None }
        }

        fn to_digit(&self) -> Option<u8> {
            let c = self.to_char()?;
            c.to_digit(10).map(|c| c as u8)
        }

        fn to_char(&self) -> Option<char> {
            // if self.raw_code == 28 {
            //     // FIXME: I added this bc I thought ENTER had no char, but it is actually `\r` already.
            //     return Some('\n');
            // }
            // self.logical_key.to_text().and_then(|s| s.chars().nth(0))
            match self {
                Key::Enter => Some('\n'),
                Key::Backspace => Some('\u{8}'),
                Key::Char(c) => Some(*c),
                _ => None,
            }
        }
    }

    impl KeyModifiers {
        pub const NONE: Self = KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            // num_lock: false,
        };

        pub const CTRL: Self = KeyModifiers {
            ctrl: true,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: false,
            // num_lock: false,
        };

        pub const ALT: Self = KeyModifiers {
            ctrl: false,
            alt: true,
            shift: false,
            caps_lock: false,
            logo: false,
            // num_lock: false,
        };

        pub const SHIFT: Self = KeyModifiers {
            ctrl: false,
            alt: false,
            shift: true,
            caps_lock: false,
            logo: false,
            // num_lock: false,
        };

        pub const LOGO: Self = KeyModifiers {
            ctrl: false,
            alt: false,
            shift: false,
            caps_lock: false,
            logo: true,
            // num_lock: false,
        };

        pub const CTRL_SHIFT: Self = KeyModifiers::both(Self::CTRL, Self::SHIFT);

        /// For example, combine(CTRL, SHIFT) is CTRL_SHIFT
        pub const fn both(self, rhs: Self) -> Self {
            Self {
                ctrl: self.ctrl | rhs.ctrl,
                alt: self.alt | rhs.alt,
                shift: self.shift | rhs.shift,
                caps_lock: self.caps_lock | rhs.caps_lock,
                logo: self.logo | rhs.logo,
                // num_lock: self.num_lock | rhs.num_lock,
            }
        }
    }
    impl From<winit::event::Modifiers> for KeyModifiers {
        fn from(value: winit::event::Modifiers) -> Self {
            Self {
                ctrl: value.state().control_key(),
                alt: value.state().alt_key(),
                shift: value.state().shift_key(),
                // TODO
                caps_lock: false,
                logo: value.state().super_key(),
                // num_lock,
            }
        }
    }
    impl std::ops::BitOr for KeyModifiers {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            Self::both(self, rhs)
        }
    }
    impl std::ops::BitAnd for KeyModifiers {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            Self {
                ctrl: self.ctrl & rhs.ctrl,
                alt: self.alt & rhs.alt,
                shift: self.shift & rhs.shift,
                caps_lock: self.caps_lock & rhs.caps_lock,
                logo: self.logo & rhs.logo,
                // num_lock: self.num_lock & rhs.num_lock,
            }
        }
    }

    impl TryFrom<winit::event::KeyEvent> for Key {
        type Error = ();

        fn try_from(value: winit::event::KeyEvent) -> Result<Self, Self::Error> {
            if value.state.is_pressed() {
                match value.physical_key {
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft) => {
                        Ok(Key::ArrowKeyLeft)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) => {
                        Ok(Key::ArrowKeyRight)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown) => {
                        Ok(Key::ArrowKeyDown)
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp) => {
                        Ok(Key::ArrowKeyUp)
                    }

                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Enter) => {
                        Ok(Key::Enter)
                    }

                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace) => {
                        Ok(Key::Backspace)
                    }

                    _ => value
                        .logical_key
                        .to_text()
                        .and_then(|s| s.chars().next())
                        .map(Key::Char)
                        .ok_or(()),
                }
            } else {
                Err(())
            }
        }
    }
}
