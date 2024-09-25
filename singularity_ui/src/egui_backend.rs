use crate::{UIDisplay, UIElement};
use std::sync::{Arc, Mutex};

pub const FRAME_RATE: f32 = 5.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

impl UIElement {
    fn draw(&self, ui: &mut egui::Ui) {
        match self {
            UIElement::Div(children) => {
                for child in children {
                    child.draw(ui);
                }
            }
            UIElement::Letter(c) => {
                ui.heading(c.to_string());
            }
        }
    }
}

impl eframe::App for UIDisplay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.root_element.lock().unwrap().draw(ui);
        });

        ctx.request_repaint_after_secs(FRAME_DELTA_SECONDS);
    }
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
