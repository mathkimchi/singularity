use crate::runner::AppletRunner;
use calloop::LoopHandle;
use smithay::reexports::winit::event_loop::EventLoopProxy;
use sonamu_sync::TrackedEncapsulatedLock;
use sonamu_ui::{
    DisplayNotification, DisplayRequest, PointerGrab, UIDisplay, ui_element::PrimitiveScene,
};
use std::{
    sync::{Arc, atomic::AtomicBool, mpsc},
    thread,
};
use wgpu::InstanceDescriptor;

/// For the runner/main app being UI'd to hold
///
/// Instead of storing the event receiver in here,
/// it is registered in the calloop.
pub(crate) struct UIHandle {
    pub is_running: Arc<AtomicBool>,
    pub ui_content: TrackedEncapsulatedLock<PrimitiveScene>,
    /// Wakes the display to redraw
    display_proxy: EventLoopProxy<DisplayRequest>,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
}
impl UIHandle {
    pub fn init_ui(
        initial_content: PrimitiveScene,
        runner_event_loop: &LoopHandle<'_, AppletRunner>,
    ) -> Self {
        let is_running = Arc::new(AtomicBool::new(true));
        let ui_content = TrackedEncapsulatedLock::new(initial_content);
        let (tx, rx) = calloop::channel::channel();

        // I don't think order matters,
        // but just in-case I should start listening before I make the display
        runner_event_loop
            .insert_source(rx, |event, &mut (), runner| {
                let calloop::channel::Event::Msg(event) = event else {
                    // the window was closed or quit, so `AppletRunner::run` returns
                    runner.loop_signal.stop();
                    return;
                };
                runner.handle_ui_event(event);
            })
            .unwrap();

        let (notification_tx, notification_rx) = calloop::channel::channel();
        runner_event_loop
            .insert_source(notification_rx, |event, &mut (), runner| {
                let calloop::channel::Event::Msg(notification) = event else {
                    return;
                };
                match notification {
                    DisplayNotification::Presented => runner.smithay_state.frame_presented(),
                    DisplayNotification::Info(info) => {
                        runner.smithay_state.set_display_info(info);
                        runner.needs_redraw = true;
                    }
                    DisplayNotification::Ime(ime) => runner.smithay_state.ime_input(&ime),
                    DisplayNotification::RelativeMotion(delta) => {
                        runner.relative_pointer_motion(delta);
                    }
                }
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

        let (proxy_tx, proxy_rx) = mpsc::channel();
        {
            let is_running = is_running.clone();
            let ui_content = ui_content.clone();
            let device = device.clone();
            let queue = queue.clone();
            let instance = instance.clone();
            thread::spawn(|| {
                UIDisplay::run_display(
                    is_running,
                    tx,
                    ui_content,
                    device,
                    queue,
                    instance,
                    proxy_tx,
                    notification_tx,
                );
            });

            dbg!("Started UI thread");
        }

        Self {
            is_running,
            ui_content,
            display_proxy: proxy_rx.recv().unwrap(),

            device,
            queue,
            instance,
        }
    }

    pub fn set_ui_content(&self, new_content: PrimitiveScene) {
        self.ui_content.set(new_content);
        // errs only if the display already closed
        let _ = self.display_proxy.send_event(DisplayRequest::Redraw);
    }

    /// Closes the display, which then ends the runner's event loop
    pub fn quit(&self) {
        self.is_running
            .store(false, std::sync::atomic::Ordering::Relaxed);
        // wakes the display so it notices
        let _ = self.display_proxy.send_event(DisplayRequest::Redraw);
    }

    /// `position` is in window px
    pub fn warp_pointer(&self, position: [f64; 2]) {
        // errs only if the display already closed
        let _ = self
            .display_proxy
            .send_event(DisplayRequest::WarpPointer(position));
    }

    pub fn set_pointer_grab(&self, grab: PointerGrab) {
        // errs only if the display already closed
        let _ = self
            .display_proxy
            .send_event(DisplayRequest::SetPointerGrab(grab));
    }
}
