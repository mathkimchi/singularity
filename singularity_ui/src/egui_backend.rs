use crate::{UIElement, UIEvent};
use egui::Widget;
use std::sync::{Arc, Mutex};

pub const FRAME_RATE: f32 = 5.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

impl egui::Widget for &UIElement {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            UIElement::Container(children) => {
                ui.horizontal(move |ui| {
                    for (child, size) in children {
                        ui.add_sized((size.0 as f32, size.1 as f32), child);
                    }
                })
                .response
            }
            UIElement::Bordered(inner) => {
                egui::Frame::none()
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .show(ui, move |ui| inner.ui(ui))
                    .response
            }
            UIElement::Text(s) => ui.label(s),
        }
    }
}

pub struct UIDisplay {
    root_element: Arc<Mutex<UIElement>>,

    event_queue: Arc<Mutex<Vec<UIEvent>>>,
}
impl UIDisplay {
    pub fn create_display(
        root_element: Arc<Mutex<UIElement>>,
        event_queue: Arc<Mutex<Vec<UIEvent>>>,
    ) -> UIDisplay {
        UIDisplay {
            root_element,
            event_queue,
        }
    }

    // pub fn collect_events(&mut self) -> Vec<UIEvent> {
    //     std::mem::take(&mut self.event_queue)
    // }

    /// Returns when display is closed.
    pub fn run_display(root_element: Arc<Mutex<UIElement>>, event_queue: Arc<Mutex<Vec<UIEvent>>>) {
        eframe::run_native(
            "Singularity",
            eframe::NativeOptions {
                event_loop_builder: Some(Box::new(|event_loop_builder| {
                    use winit::platform::wayland::EventLoopBuilderExtWayland;
                    // NOTE: eframe 28 uses winit 0.29, and this doesn't work with winit 0.30
                    event_loop_builder.with_any_thread(true);
                })),
                ..Default::default()
            },
            Box::new(move |_cc| {
                Ok(Box::new(UIDisplay::create_display(
                    root_element,
                    event_queue,
                )))
            }),
        )
        .unwrap();
    }
}

impl eframe::App for UIDisplay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.root_element.lock().unwrap().ui(ui);
        });

        for new_event in ctx.input(|i| i.events.clone()) {
            #[allow(clippy::single_match)]
            match new_event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } => match match key {
                    egui::Key::Num0 => Some('0'),
                    egui::Key::Num1 => Some('1'),
                    egui::Key::Num2 => Some('2'),
                    egui::Key::Num3 => Some('3'),
                    egui::Key::Num4 => Some('4'),
                    egui::Key::Num5 => Some('5'),
                    egui::Key::Num6 => Some('6'),
                    egui::Key::Num7 => Some('7'),
                    egui::Key::Num8 => Some('8'),
                    egui::Key::Num9 => Some('9'),
                    egui::Key::A => Some('a'),
                    egui::Key::B => Some('b'),
                    egui::Key::C => Some('c'),
                    egui::Key::D => Some('d'),
                    egui::Key::E => Some('e'),
                    egui::Key::F => Some('f'),
                    egui::Key::G => Some('g'),
                    egui::Key::H => Some('h'),
                    egui::Key::I => Some('i'),
                    egui::Key::J => Some('j'),
                    egui::Key::K => Some('k'),
                    egui::Key::L => Some('l'),
                    egui::Key::M => Some('m'),
                    egui::Key::N => Some('n'),
                    egui::Key::O => Some('o'),
                    egui::Key::P => Some('p'),
                    egui::Key::Q => Some('q'),
                    egui::Key::R => Some('r'),
                    egui::Key::S => Some('s'),
                    egui::Key::T => Some('t'),
                    egui::Key::U => Some('u'),
                    egui::Key::V => Some('v'),
                    egui::Key::W => Some('w'),
                    egui::Key::X => Some('x'),
                    egui::Key::Y => Some('y'),
                    egui::Key::Z => Some('z'),
                    _ => None,
                } {
                    Some(c) => {
                        self.event_queue.lock().unwrap().push(UIEvent::KeyPress {
                            key_char: c,
                            alt: modifiers.alt,
                            ctrl: modifiers.ctrl,
                            shift: modifiers.shift,
                        });
                    }
                    None => {}
                },
                _ => {}
            }
        }

        ctx.request_repaint_after_secs(FRAME_DELTA_SECONDS);
    }
}
