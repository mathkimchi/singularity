//! Since this is a private module, I can just make all the structs and fields pub as needed

use super::UIDisplay;
use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplayUnits},
    ui_element::{CharGrid, FONT_SIZE_F, InternalCharCell, UIElement},
    winit_backend::WinitData,
};
use glyphon::{Metrics, TextRenderer};
use image::RgbaImage;
use std::iter;
use wgpu::{BindGroupLayout, MultisampleState, SurfaceConfiguration, util::DeviceExt as _};

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
    pub(super) const VERTICES: &[Vertex] = &[
        // A
        Vertex {
            position: [3., -1., 0.0],
        },
        // B
        Vertex {
            position: [-1., 3., 0.0],
        },
        // C
        Vertex {
            position: [-1., -1., 0.0],
        },
    ];

    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
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

    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
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

    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
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

    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Data needed for drawing
pub(super) struct DrawingSharedData<'a> {
    render_pass: wgpu::RenderPass<'a>,

    rectangle_render_pipeline: &'a wgpu::RenderPipeline,

    image_render_pipeline: &'a wgpu::RenderPipeline,
    image_texture_bind_group_layout: &'a BindGroupLayout,

    char_grid_render_pipeline: &'a wgpu::RenderPipeline,
    char_grid_texture_bind_group_layout: &'a BindGroupLayout,

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
        image_buffer: &RgbaImage,
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
                        layout: drawing_shared_data.image_texture_bind_group_layout,
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
            .set_pipeline(drawing_shared_data.char_grid_render_pipeline);

        let width = char_grid.width() as u32;
        let height = char_grid.height() as u32;

        // load the actual char grid info as if it was a texture where each pixel is a char
        {
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
                        layout: drawing_shared_data.char_grid_texture_bind_group_layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        }],
                        label: Some("bind_group"),
                    });
            drawing_shared_data
                .render_pass
                .set_bind_group(0, Some(&bind_group), &[]);
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
                    // area.size()
                    //     .width
                    //     .pixels(drawing_shared_data.surface_config.width as _)
                    //     as _,
                    100.,
                    area.size()
                        .height
                        .pixels(drawing_shared_data.surface_config.height as _)
                        as _,
                ],
                // grid_size: [width, height],
                grid_size: [10, 10],
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

    fn draw(&self, drawing_shared_data: &mut DrawingSharedData, container_area: DisplayArea) {
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
                Self::draw_char_grid(
                    drawing_shared_data,
                    char_grid,
                    DisplayArea::from_corner_size(
                        container_area.0,
                        char_grid.display_size().into(),
                    ),
                );

                /*
                let mut text_buffer = glyphon::Buffer::new(
                    drawing_shared_data.font_system,
                    Metrics::new(FONT_SIZE_F, FONT_SIZE_F),
                );

                let attrs = &glyphon::Attrs::new().family(glyphon::Family::Monospace);
                let shaping = glyphon::Shaping::Advanced;

                // // text_buffer.set_size(
                // //     &mut drawing_shared_data.font_system,
                // //     Some(physical_width),
                // //     Some(physical_height),
                // // );

                // // text_buffer.set_text(
                // //     drawing_shared_data.font_system,
                // //     character.to_string().as_str(),
                // //     &glyphon::Attrs::new().family(glyphon::Family::Monospace),
                // //     glyphon::Shaping::Advanced,
                // //     None,
                // // );
                // text_buffer.shape_until_scroll(drawing_shared_data.font_system, false);

                // let mut text_renderer = TextRenderer::new(
                //     drawing_shared_data.atlas,
                //     drawing_shared_data.device,
                //     MultisampleState::default(),
                //     None,
                // );

                for row in 0..char_grid.height() {
                    for col in 0..char_grid.width() {
                        // log::debug!(
                        //     "Row: {row}, col: {col}, w: {}, h: {}",
                        //     char_grid.width(),
                        //     char_grid.height()
                        // );
                        let CharCell { character, fg, bg } = char_grid.get_char(row, col);

                        let top_left = DisplayCoord::new(
                            container_area.0.x + DisplayUnits::Pixels(FONT_SIZE / 2 * (col as i32)),
                            container_area.0.y + DisplayUnits::Pixels(FONT_SIZE * (row as i32) + 1),
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

                        if bg != Color::TRANSPARENT {
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
                                bg,
                                Color::TRANSPARENT,
                            );
                        }

                        if character == ' ' {
                            continue;
                        }

                        // let mut text_buffer = glyphon::Buffer::new(
                        //     drawing_shared_data.font_system,
                        //     Metrics::new(FONT_SIZE_F, FONT_SIZE_F),
                        // );

                        // text_buffer.set_size(
                        //     &mut drawing_shared_data.font_system,
                        //     Some(physical_width),
                        //     Some(physical_height),
                        // );

                        text_buffer.set_text(
                            drawing_shared_data.font_system,
                            character.to_string().as_str(),
                            // &glyphon::Attrs::new().family(glyphon::Family::Monospace),
                            attrs,
                            // glyphon::Shaping::Advanced,
                            shaping,
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
                                    default_color: fg.into(),
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
                */
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
            char_grid_render_pipeline,
            char_grid_texture_bind_group_layout,
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
                char_grid_render_pipeline,
                char_grid_texture_bind_group_layout,
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
