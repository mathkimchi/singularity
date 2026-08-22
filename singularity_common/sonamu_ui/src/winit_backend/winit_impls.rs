use crate::{
    display_units::DisplayContainerSize,
    ui_event::Key,
    winit_backend::{UIDisplay, WgpuData, WinitData},
};
use smithay::reexports::winit;
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
            .with_title("Sonamu");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.event_queue
            .send(super::ui_event::UIEvent::WindowResized(
                DisplayContainerSize::new(800, 600),
            ))
            .unwrap();

        let winit_data = WinitData::new(
            window,
            self.device.clone(),
            self.queue.clone(),
            &self.instance,
        );
        self.winit_data = Some(winit_data);
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

                self.event_queue
                    .send(crate::ui_event::UIEvent::WindowResized(
                        DisplayContainerSize::new(size.width, size.height),
                    ))
                    .unwrap();
            }
            winit::event::WindowEvent::CloseRequested => {
                self.is_running
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                event_loop.exit();
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
                    self.event_queue
                        .send(super::ui_event::UIEvent::KeyPress(key, self.key_modifiers))
                        .unwrap();
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
                Self::draw(&mut self.winit_data, &self.ui_content.get());
            }
            _ => {}
        }
    }
}
