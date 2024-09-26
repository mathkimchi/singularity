use crate::{ui_event::UIEvent, DisplayArea, UIElement};
use egui::{widget_text, Widget};
use std::sync::{Arc, Mutex};

pub const FRAME_RATE: f32 = 5.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

impl egui::Widget for &UIElement {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            UIElement::Container(children) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.horizontal(move |ui| {
                    for (child, size) in children {
                        ui.add_sized((size.0 as f32, size.1 as f32), child);
                    }
                })
                .response
            }
            UIElement::Bordered(inner) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                egui::Frame::none()
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .show(ui, move |ui| inner.ui(ui))
                    .response
            }
            UIElement::Text(s) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.label(s)
            }
            UIElement::CharGrid { content } => {
                // FIXME: heights not constant for some reason
                const CHAR_SIZE: DisplayArea = (8, 16);
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.spacing_mut().window_margin = egui::Margin::ZERO;
                ui.spacing_mut().indent = 0.0;
                ui.vertical(|ui| {
                    for line in content {
                        ui.horizontal(|ui| {
                            for &(c, color) in line {
                                dbg!(ui.spacing());
                                ui.add_sized(
                                    egui::Vec2::new(CHAR_SIZE.0 as f32, CHAR_SIZE.1 as f32),
                                    egui::Label::new(
                                        widget_text::RichText::monospace(c.to_string().into())
                                            .size(CHAR_SIZE.1 as f32)
                                            .color(color),
                                    ),
                                );
                            }
                        });
                    }
                })
                .response
            }
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

        for event in ctx.input(|i| i.events.clone()) {
            self.event_queue.lock().unwrap().push(event);
        }

        ctx.request_repaint_after_secs(FRAME_DELTA_SECONDS);
    }
}
