use crate::runner::AppletRunner;
use calloop::LoopHandle;
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{UIDisplay, ui_element::PrimitiveScene};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};

/// For the runner/main app being UI'd to hold
///
/// Instead of storing the event receiver in here,
/// it is registered in the calloop.
pub(crate) struct UIHandle {
    pub is_running: Arc<AtomicBool>,
    pub ui_content: EncapsulatedLock<PrimitiveScene>,
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

        {
            let is_running = is_running.clone();
            let ui_content = ui_content.clone();
            thread::spawn(|| {
                UIDisplay::run_display(is_running, tx, ui_content);
            });
        }

        Self {
            is_running,
            ui_content,
        }
    }

    pub fn set_ui_content(&self, new_content: PrimitiveScene) {
        self.ui_content.set(new_content);
    }
}
