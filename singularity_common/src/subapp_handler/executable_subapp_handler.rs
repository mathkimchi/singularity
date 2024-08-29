use std::process::Child;

use super::SubappHandler;

pub struct ExecutableSubappHandler {
    subapp_process: Child,
}
impl ExecutableSubappHandler {
    pub fn new() -> Self {
        todo!()
    }
}
impl SubappHandler for ExecutableSubappHandler {
    fn give_display_buffer(
        &self,
        display_buffer: &mut std::sync::Arc<std::sync::Mutex<super::DisplayBuffer>>,
    ) {
        todo!()
    }

    fn peek_display_buffer(&self) -> &std::sync::Arc<std::sync::Mutex<super::DisplayBuffer>> {
        todo!()
    }

    fn inform_event(&self, event: super::Event) {
        todo!()
    }

    fn dump_requests(&mut self) -> Vec<super::Request> {
        todo!()
    }
}
