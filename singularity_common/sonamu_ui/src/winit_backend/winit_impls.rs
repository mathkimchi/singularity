use winit::platform::scancode::PhysicalKeyExtScancode;

use crate::{
    display_units::DisplayContainerSize,
    ui_element::{Cursor, CursorIcon},
    ui_event::{KeySymbol, MouseEvent, MouseEventKind, UIEvent},
    winit_backend::{UIDisplay, WgpuData, WinitData},
};
use smithay::reexports::winit;
use std::sync::Arc;
use winit::{
    dpi::LogicalSize,
    window::{CustomCursor, Window},
};

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
            .send(UIEvent::WindowResized(DisplayContainerSize::new(800, 600)))
            .unwrap();

        let winit_data = WinitData::new(
            window,
            self.device.clone(),
            self.queue.clone(),
            &self.instance,
        );
        self.winit_data = Some(winit_data);
    }

    /// Sent when the ui content is updated
    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, (): ()) {
        if let Some(winit_data) = &self.winit_data {
            winit_data.window.request_redraw();
        }
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

        // Other redraws are requested through `user_event` when content changes
        match event {
            winit::event::WindowEvent::Resized(size) => {
                surface_config.width = size.width;
                surface_config.height = size.height;
                surface.configure(device, surface_config);

                self.event_queue
                    .send(UIEvent::WindowResized(DisplayContainerSize::new(
                        size.width,
                        size.height,
                    )))
                    .unwrap();

                window.request_redraw();
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
                // log::debug!("{event:?}");
                self.event_queue
                    .send(UIEvent::Key {
                        symbol: KeySymbol::try_from(event.clone()).ok(),
                        modifiers: self.key_modifiers,
                        raw_keycode: event.physical_key.to_scancode().unwrap_or(0),
                        pressed: event.state.is_pressed(),
                    })
                    .unwrap();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = [position.x, position.y];
                self.update_cursor(event_loop);
                Self::send_mouse_event(
                    &self.event_queue,
                    self.cursor_position,
                    MouseEventKind::Motion,
                );
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                // REVIEW: (claude) evdev codes from linux/input-event-codes.h
                let button = match button {
                    winit::event::MouseButton::Left => 0x110,
                    winit::event::MouseButton::Right => 0x111,
                    winit::event::MouseButton::Middle => 0x112,
                    winit::event::MouseButton::Forward => 0x115,
                    winit::event::MouseButton::Back => 0x116,
                    winit::event::MouseButton::Other(_) => return,
                };
                Self::send_mouse_event(
                    &self.event_queue,
                    self.cursor_position,
                    MouseEventKind::Button {
                        button,
                        pressed: state.is_pressed(),
                    },
                );
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // REVIEW: (claude) winit's positive is up/left while Wayland's is down/right, so negate
                let kind = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => MouseEventKind::Scroll {
                        // 15px per notch, like most compositors
                        delta: [f64::from(-x) * 15., f64::from(-y) * 15.],
                        v120: Some([(-x * 120.) as i32, (-y * 120.) as i32]),
                    },
                    winit::event::MouseScrollDelta::PixelDelta(pos) => MouseEventKind::Scroll {
                        delta: [-pos.x, -pos.y],
                        v120: None,
                    },
                };
                Self::send_mouse_event(&self.event_queue, self.cursor_position, kind);
            }
            winit::event::WindowEvent::RedrawRequested => {
                if let Some(content) = self.ui_content.get_if_dirty() {
                    Self::draw(&mut self.winit_data, &content);
                    self.present_queue.send(()).unwrap();

                    // what's under the pointer may have changed
                    self.drawn_cursors = content.cursors;
                    self.update_cursor(event_loop);
                }
            }
            _ => {}
        }
    }
}
impl UIDisplay {
    /// Sets the window's cursor to the one under the pointer, if it changed
    fn update_cursor(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(winit_data) = &self.winit_data else {
            return;
        };
        let cursor = self.drawn_cursors.cursor_at(self.cursor_position);
        if cursor == self.applied_cursor.as_ref() {
            return;
        }

        let window = &winit_data.window;
        window.set_cursor_visible(cursor != Some(&Cursor::Hidden));
        match cursor {
            None => window.set_cursor(CursorIcon::Default),
            Some(Cursor::Named(icon)) => window.set_cursor(*icon),
            Some(Cursor::Hidden) => {}
            Some(Cursor::Custom {
                rgba,
                width,
                height,
                hotspot: [hotspot_x, hotspot_y],
            }) => match CustomCursor::from_rgba(
                rgba.to_vec(),
                *width,
                *height,
                *hotspot_x,
                *hotspot_y,
            ) {
                Ok(source) => window.set_cursor(event_loop.create_custom_cursor(source)),
                Err(err) => {
                    dbg!("Bad custom cursor:", err);
                }
            },
        }
        self.applied_cursor = cursor.cloned();
    }

    fn send_mouse_event(
        event_queue: &calloop::channel::Sender<UIEvent>,
        cursor_position: [f64; 2],
        kind: MouseEventKind,
    ) {
        event_queue
            .send(UIEvent::Mouse(MouseEvent {
                position: cursor_position,
                kind,
            }))
            .unwrap();
    }
}
