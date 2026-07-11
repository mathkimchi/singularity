use crate::{
    display_units::DisplayContainerSize,
    ui_event::Key,
    winit_backend::{UIDisplay, WgpuData, WinitData},
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
            wgpu_data:
                WgpuData {
                    device,
                    // queue,
                    surface,
                    surface_config,
                    ..
                },
        } = state;

        match event {
            winit::event::WindowEvent::Resized(size) => {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(device, surface_config);
                window.request_redraw();

                self.ui_event_queue
                    .lock()
                    .unwrap()
                    .push(crate::ui_event::UIEvent::WindowResized(
                        DisplayContainerSize::new(size.width, size.height),
                    ));
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
