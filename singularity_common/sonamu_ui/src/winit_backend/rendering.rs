//! Since this is a private module, I can just make all the structs and fields pub as needed

use super::UIDisplay;
use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplayUnits},
    ui_element::{CharGrid, FONT_SIZE_F, InternalCharCell, UIElement},
    winit_backend::{WgpuData, WinitData},
};
use glyphon::{AttrsOwned, Metrics, TextRenderer};
use image::RgbaImage;
use msdfgen::{Bitmap, FillRule, FontExt, MsdfGeneratorConfig};
use std::iter;
use wgpu::{
    BindGroup, BindGroupLayout, Device, MultisampleState, PipelineCompilationOptions,
    RenderPipeline, SurfaceConfiguration, include_wgsl, util::DeviceExt as _,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 3],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    /// Large triangle trick to cover the whole screen
    /// https://webgpufundamentals.org/webgpu/lessons/webgpu-large-triangle-to-cover-clip-space.html (allegedly 5%)
    /// TODO: Doing two triangles for tight rectangles will probably boost speed much more, but I might end up only needing one render call so idk...
    pub(super) const VERTICES: &[Self] = &[
        // A
        Self {
            position: [3., -1., 0.0],
        },
        // B
        Self {
            position: [-1., 3., 0.0],
        },
        // C
        Self {
            position: [-1., -1., 0.0],
        },
    ];

    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub(super) struct RoundRectInstance {
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

    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct RectangleRenderer {
    render_pipeline: RenderPipeline,
}
impl RectangleRenderer {
    pub fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
        let rectangle_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let rectangle_shader = device.create_shader_module(include_wgsl!("rectangle_shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            multisample: MultisampleState {
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

        Self { render_pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub(super) struct ImageInstance {
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

    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    fn set_instance_buffer(drawing_shared_data: &mut DrawingSharedData, area: DisplayArea) {
        let instances = vec![Self {
            // this currently takes in top left
            origin: [
                area.0
                    .x
                    .pixels(drawing_shared_data.surface_config.width as _) as _,
                area.0
                    .y
                    .pixels(drawing_shared_data.surface_config.height as _) as _,
            ],
            size: [
                area.size()
                    .width
                    .pixels(drawing_shared_data.surface_config.width as _) as _,
                area.size()
                    .height
                    .pixels(drawing_shared_data.surface_config.height as _) as _,
            ],
        }];

        let instance_buffer =
            drawing_shared_data
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // vertex buffer slot 1 is actually the instance buffer
        drawing_shared_data
            .render_pass
            .set_vertex_buffer(1, instance_buffer.slice(..));
    }
}

pub struct ImageRenderer {
    render_pipeline: RenderPipeline,
    image_texture_bind_group_layout: BindGroupLayout,
}
impl ImageRenderer {
    pub fn new(device: &Device, surface_config: &SurfaceConfiguration) -> Self {
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
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            multisample: MultisampleState {
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

        Self {
            render_pipeline,
            image_texture_bind_group_layout,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub(super) struct CharGridInstance {
    /// This is the top left, not the center
    /// Currently includes the border
    origin: [f32; 2],
    /// Currently includes the border
    size: [f32; 2],
    /// In terms of characters
    grid_size: [u32; 2],
}
impl CharGridInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        1 => Float32x2,
        2 => Float32x2,
        3 => Uint32x2,
    ];

    pub(super) const fn desc() -> wgpu::VertexBufferLayout<'static> {
        

        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct CharGridRenderer {
    render_pipeline: RenderPipeline,
    /// TODO: rename
    char_grid_texture_bind_group_layout: BindGroupLayout,
    atlas_bind_group: BindGroup,
}
impl CharGridRenderer {
    pub fn new(
        device: &Device,
        surface_config: &SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {
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
        let (atlas_bind_group_layout, atlas_bind_group) = Self::atlas_bind_group(device, queue);

        let char_grid_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&atlas_bind_group_layout),
                    Some(&char_grid_texture_bind_group_layout),
                ],
                immediate_size: 0,
            });
        let char_grid_shader = device.create_shader_module(include_wgsl!("char_grid_shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            multisample: MultisampleState {
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

        Self {
            render_pipeline,
            char_grid_texture_bind_group_layout,
            atlas_bind_group,
        }
    }

    /// 4 channels at f32
    /// I don't want to use this much,
    /// TODO: use klyff_msdf crate bc it supports u8 and runs on wgpu
    // const SDF_PIXEL_BYTES: usize = 4 * 4;
    const SDF_PIXEL_BYTES: usize = 4;
    const SDF_WIDTH: usize = 32;
    const SDF_HEIGHT: usize = 64;
    /// Num bytes for each character's atlas
    const ATLAS_SIZE: usize = Self::SDF_PIXEL_BYTES * Self::SDF_WIDTH * Self::SDF_HEIGHT;

    fn generate_atlas_raw_data() -> Vec<u8> {
        let mut data = vec![0u8; Self::ATLAS_SIZE * (127 - 33)];

        let font = ttf_parser::Face::parse(dejavu::sans_mono::regular(), 0).unwrap();

        let framing = msdfgen::Bound::new(
            0.,
            f64::from(font.descender()),
            f64::from(
                font.glyph_hor_advance(font.glyph_index(' ').unwrap())
                    .unwrap(),
            ),
            f64::from(font.ascender()),
        )
        .autoframe(
            Self::SDF_WIDTH as _,
            Self::SDF_HEIGHT as _,
            // NOTE: this should be the same number as PX_RANGE constant in the shader
            msdfgen::Range::Px(4.0),
            None,
        )
        .unwrap();

        for ascii_code in 33..127u8 {
            // Mostly just taken from msdf gen library: https://crates.io/crates/msdfgen
            // NOTE: Versions are weird, might need to downgrade ttf_parser
            let glyph = font.glyph_index(ascii_code as char).unwrap();

            let mut shape = font.glyph_shape(glyph).unwrap();

            let fill_rule = FillRule::default();

            let mut bitmap = Bitmap::new(Self::SDF_WIDTH as _, Self::SDF_HEIGHT as _);

            shape.edge_coloring_simple(3.0, 0);

            let config = MsdfGeneratorConfig::default();

            shape.generate_mtsdf(&mut bitmap, framing, config);

            // optionally
            shape.correct_sign(&mut bitmap, framing, fill_rule);
            shape.correct_msdf_error(&mut bitmap, framing, config);

            // let error = shape.estimate_error(&mut bitmap, framing, 5, FillRule::default());
            // log::debug!("Estimated error: {error}");

            bitmap.flip_y();

            let atlas_index = ascii_code as usize - 33;
            data[(atlas_index * Self::ATLAS_SIZE)..((atlas_index + 1) * Self::ATLAS_SIZE)]
                .copy_from_slice(bitmap.convert::<msdfgen::Rgba<u8>>().raw_pixels());
        }

        data
    }

    fn atlas_bind_group(device: &Device, queue: &wgpu::Queue) -> (BindGroupLayout, BindGroup) {
        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            // idk what filterable or multisampled does, but the rest are straight-forward
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
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
                label: Some("atlas_bind_group_layout"),
            });

        let texture_size = wgpu::Extent3d {
            width: Self::SDF_WIDTH as _,
            height: Self::SDF_HEIGHT as _,
            // All textures are stored as 3D
            depth_or_array_layers: (127 - 33),
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1, // We'll talk about this a little later
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            // COPY_DST means that we want to copy data to this texture
            // I guess texture_2d_array is also storage binding, even though google says it uses texture binding (grrr)
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("atlas_texture"),
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
            bytemuck::cast_slice(&Self::generate_atlas_raw_data()),
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((Self::SDF_PIXEL_BYTES * Self::SDF_WIDTH) as u32),
                rows_per_image: Some(Self::SDF_HEIGHT as u32),
            },
            texture_size,
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    // TODO: use texture view array?
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("atlas_bind_group"),
        });

        (atlas_bind_group_layout, bind_group)
    }
}

/// Data needed for drawing
pub(super) struct DrawingSharedData<'a> {
    render_pass: wgpu::RenderPass<'a>,

    rectangle_renderer: &'a RectangleRenderer,
    image_renderer: &'a ImageRenderer,
    char_grid_renderer: &'a CharGridRenderer,

    /// Just one large triangle
    vertex_buffer: &'a wgpu::Buffer,

    font_system: &'a mut glyphon::FontSystem,
    swash_cache: &'a mut glyphon::SwashCache,
    viewport: &'a mut glyphon::Viewport,
    atlas: &'a mut glyphon::TextAtlas,
    // text_renderer: &'a mut glyphon::TextRenderer,
    // text_buffer: &'a mut glyphon::Buffer,
    device: &'a Device,
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
            .set_pipeline(&drawing_shared_data.rectangle_renderer.render_pipeline);
        // these buffers are how we pass data to the gpu
        drawing_shared_data
            .render_pass
            .set_vertex_buffer(0, drawing_shared_data.vertex_buffer.slice(..));

        let instances = vec![RoundRectInstance {
            // this currently takes in top left
            origin: [
                area.0
                    .x
                    .pixels(drawing_shared_data.surface_config.width as _) as _,
                area.0
                    .y
                    .pixels(drawing_shared_data.surface_config.height as _) as _,
            ],
            size: [
                area.size()
                    .width
                    .pixels(drawing_shared_data.surface_config.width as _) as _,
                area.size()
                    .height
                    .pixels(drawing_shared_data.surface_config.height as _) as _,
            ],
            corner_radius: radius,
            border_dist,
            main_color: inner_color.0.map(|c| f32::from(c) / f32::from(u8::MAX)),
            // shouldn't matter
            border_color: border_color.0.map(|c| f32::from(c) / f32::from(u8::MAX)),
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
    }

    fn draw_image(
        drawing_shared_data: &mut DrawingSharedData,
        image_buffer: &RgbaImage,
        area: DisplayArea,
    ) {
        drawing_shared_data
            .render_pass
            .set_pipeline(&drawing_shared_data.image_renderer.render_pipeline);

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
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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
                        layout: &drawing_shared_data
                            .image_renderer
                            .image_texture_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
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

        ImageInstance::set_instance_buffer(drawing_shared_data, area);

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

    fn draw_char_grid(
        drawing_shared_data: &mut DrawingSharedData,
        char_grid: &CharGrid,
        area: DisplayArea,
    ) {
        // NOTE: rn, the area is just auto-computed from the bounds,
        // TODO: let apps customize bounds

        drawing_shared_data
            .render_pass
            .set_pipeline(&drawing_shared_data.char_grid_renderer.render_pipeline);

        let width = char_grid.width() as u32;
        let height = char_grid.height() as u32;

        // load the actual char grid info as if it was a texture where each pixel is a char
        {
            drawing_shared_data.render_pass.set_bind_group(
                0,
                Some(&drawing_shared_data.char_grid_renderer.atlas_bind_group),
                &[],
            );

            let texture_size = wgpu::Extent3d {
                width,
                height,
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                depth_or_array_layers: 1,
            };

            let texture = drawing_shared_data
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    size: texture_size,
                    mip_level_count: 1, // We'll talk about this a little later
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba32Uint,
                    // STORAGE_BINDING instead of TEXTURE_BINDING because we are using a storage_texture instead of normal texture
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("char_grid_storage"),
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
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                // The actual pixel data
                bytemuck::cast_slice(char_grid.content()),
                // The layout of the texture
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(InternalCharCell::BYTES as u32 * width),
                    rows_per_image: Some(height),
                },
                texture_size,
            );

            // We don't need to configure the texture view much, so let's
            // let wgpu define it.
            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group =
                drawing_shared_data
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &drawing_shared_data
                            .char_grid_renderer
                            .char_grid_texture_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        }],
                        label: Some("bind_group"),
                    });
            drawing_shared_data
                .render_pass
                .set_bind_group(1, Some(&bind_group), &[]);
        }

        // these buffers are how we pass data to the gpu
        // pass in the large triangle
        drawing_shared_data
            .render_pass
            .set_vertex_buffer(0, drawing_shared_data.vertex_buffer.slice(..));

        // set instance buffer
        {
            let instances = vec![CharGridInstance {
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
                grid_size: [width, height],
            }];

            let instance_buffer =
                drawing_shared_data
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instances),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            // vertex buffer slot 1 is actually the instance buffer
            drawing_shared_data
                .render_pass
                .set_vertex_buffer(1, instance_buffer.slice(..));
        }

        drawing_shared_data
            .render_pass
            .draw(0..Vertex::VERTICES.len() as _, 0..1); // 1 bc we only draw 1 image at a time (which I am not happy about)
    }

    fn draw_text_with_glyphon(
        drawing_shared_data: &mut DrawingSharedData,
        text: &[(String, AttrsOwned)],
        container_area: DisplayArea,
    ) {
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
    }

    fn draw(&self, drawing_shared_data: &mut DrawingSharedData, container_area: DisplayArea) {
        match self {
            Self::Container(children) => {
                for ui_element in children {
                    // draw the inner widget
                    ui_element.draw(drawing_shared_data, container_area);
                }
            }
            Self::Contained(inner_element, area) => {
                inner_element.draw(drawing_shared_data, area.map_onto(container_area));
            }
            // FIXME: there are weird border lines
            Self::Bordered(inner_element, border_color) => {
                Self::fill_rect(
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

                // draw the inner widget
                inner_element.draw(drawing_shared_data, inner_area);
            }
            Self::Backgrounded(inner_element, bg_color) => {
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
            Self::Text(text) => {
                Self::draw_text_with_glyphon(drawing_shared_data, text, container_area);
            }
            Self::CharGrid(char_grid) => {
                Self::draw_char_grid(drawing_shared_data, char_grid, container_area);
            }
            Self::Image(image_buffer) => {
                Self::draw_image(drawing_shared_data, image_buffer, container_area);
            }
            Self::Nothing => {}
        }
    }
}

impl UIDisplay {
    /// REVIEW: move somewhere else?
    pub(super) fn draw(winit_data: &mut Option<WinitData>, root_element: &UIElement) {
        let Some(state) = winit_data else {
            return;
        };

        let WgpuData {
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
            ..
        } = &mut state.wgpu_data;

        let wgpu::CurrentSurfaceTexture::Success(output) = surface.get_current_texture() else {
            // TODO: handle surface loss gracefully (reconfigure and skip the frame)
            panic!("failed to acquire the surface texture for rendering")
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
                rectangle_renderer,
                image_renderer,
                char_grid_renderer,
                vertex_buffer,
                font_system,
                swash_cache,
                viewport,
                atlas,
                device,
                queue,
                surface_config,
            };

            root_element.draw(&mut drawing_shared_data, DisplayArea::FULL);
        }

        queue.submit(iter::once(encoder.finish()));
        output.present();
    }
}
