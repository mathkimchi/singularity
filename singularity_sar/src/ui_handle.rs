use crate::runner::AppletRunner;
use calloop::LoopHandle;
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{UIDisplay, ui_element::PrimitiveScene};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};
use wgpu::InstanceDescriptor;

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

        let instance = wgpu::Instance::new(InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web we'll have to disable some.
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off, // Trace path
        }))
        .unwrap();

        {
            let is_running = is_running.clone();
            let ui_content = ui_content.clone();
            let device = device.clone();
            let queue = queue.clone();
            thread::spawn(|| {
                UIDisplay::run_display(is_running, tx, ui_content, device, queue, instance);
            });

            dbg!("Started UI thread");
        }

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
