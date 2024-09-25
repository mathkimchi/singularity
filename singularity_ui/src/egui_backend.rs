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
}
impl UIDisplay {
    pub fn create_display(root_element: Arc<Mutex<UIElement>>) -> UIDisplay {
        UIDisplay { root_element }
    }

    pub fn collect_events(&self) -> Vec<UIEvent> {
        todo!()
    }

    pub fn run_display(root_element: Arc<Mutex<UIElement>>) {
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
            Box::new(move |_cc| Ok(Box::new(UIDisplay::create_display(root_element)))),
        )
        .unwrap();
    }
}

impl eframe::App for UIDisplay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.root_element.lock().unwrap().ui(ui);
        });

        ctx.request_repaint_after_secs(FRAME_DELTA_SECONDS);
    }
}
