use super::SubappHandler;
use std::{
    ffi::OsStr,
    io::Read,
    process::{Child, Command, Stdio},
};

/// Uses pipes for basic communication.
pub struct ExecutableSubappHandler {
    subapp_process: Child,
}
impl ExecutableSubappHandler {
    pub fn from_executable_path(subapp_command: &mut Command) -> Self {
        let subapp_process = subapp_command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn subapp process from executable path");

        Self { subapp_process }
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
        let mut output_buffer = String::new();

        self.subapp_process
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(&mut output_buffer)
            .expect("failed to read output from subapp process");

        vec![output_buffer]
    }
}
