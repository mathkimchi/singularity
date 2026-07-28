use crate::{
    display_units::DisplayContainerSize,
    ui_event::Key,
    winit_backend::{UIDisplay, UIState, WgpuData, WinitData},
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
        let mut guard = self.shared_data.lock_state();
        let UIState::Running {
            root_element,
            ui_event_queue,
        } = &mut *guard
        else {
            // UI ended, we can quit
            event_loop.exit();
            return;
        };

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

                ui_event_queue.push_back(crate::ui_event::UIEvent::WindowResized(
                    DisplayContainerSize::new(size.width, size.height),
                ));
                self.shared_data.notify();
            }
            winit::event::WindowEvent::CloseRequested => {
                *guard = UIState::Ended;
                event_loop.exit();
                self.shared_data.notify();
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.key_modifiers = modifiers.into();
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let Ok(key) = Key::try_from(event) {
                    ui_event_queue
                        .push_back(super::ui_event::UIEvent::KeyPress(key, self.key_modifiers));
                    self.shared_data.notify();
                }

                window.request_redraw();
            }
            // TODO: handle WindowEvent::MouseInput (mouse press)
            winit::event::WindowEvent::RedrawRequested => {
                Self::draw(&mut self.winit_data, &root_element);
            }
            _ => {}
        }
    }
}
