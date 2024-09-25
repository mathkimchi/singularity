use crate::{UIDisplay, UIElement};
use std::sync::{Arc, Mutex};

impl eframe::App for UIDisplay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
        });
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
