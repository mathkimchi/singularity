use crate::runner::AppletRunner;
use calloop::LoopHandle;
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{UIDisplay, ui_element::PrimitiveScene};
use std::{
    sync::{Arc, Mutex, atomic::AtomicBool},
    thread,
};

/// For the runner/main app being UI'd to hold
///
/// Instead of storing the event receiver in here,
/// it is registered in the calloop.
pub(crate) struct UIHandle {
    pub is_running: Arc<AtomicBool>,
    pub ui_content: EncapsulatedLock<PrimitiveScene>,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}
impl UIHandle {
    pub fn init_ui(
        initial_content: PrimitiveScene,
        runner_event_loop: &LoopHandle<'_, AppletRunner>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let ui_content = EncapsulatedLock::new(initial_content);
        let (tx, rx) = calloop::channel::channel();

        // I don't think order matters,
        // but just in-case I should start listening before I make the display
        runner_event_loop
            .insert_source(rx, |event, &mut (), runner| {
                let calloop::channel::Event::Msg(event) = event else {
                    dbg!("UI closed; haven't thought abt what to do");
                    return;
                };
                runner.handle_ui_event(event);
            })
            .unwrap();

        let winit_data = Arc::new(Mutex::new(None));

        {
            let is_running = is_running.clone();
            let ui_content = ui_content.clone();
            let winit_data = winit_data.clone();
            thread::spawn(|| {
                UIDisplay::run_display(is_running, tx, ui_content, winit_data);
            });

            dbg!("Started UI thread");
        }

        let (device, queue) = loop {
            if let Some(winit_data) = &*winit_data.lock().unwrap() {
                break winit_data.get_wgpu_data();
            }

            std::hint::spin_loop();
        };

        Self {
            is_running,
            ui_content,

            device,
            queue,
        }
    }

    pub fn set_ui_content(&self, new_content: PrimitiveScene) {
        self.ui_content.set(new_content);
    }
}
