use super::SubappHandler;
use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    process::{Child, Command, Stdio},
};

pub const SOCKET_PATH: &str = "/tmp/singularity_demo_socket.sock";

/// Uses pipes for basic communication.
pub struct ExecutableSubappHandler {
    subapp_process: Child,
    unix_listener: UnixListener,
    subapp_stream: UnixStream,
}
impl ExecutableSubappHandler {
    pub fn from_executable_path(subapp_command: &mut Command) -> Self {
        if Path::new(SOCKET_PATH).exists() {
            // `Path::is_file` doesn't work for some reason

            std::fs::remove_file(SOCKET_PATH).expect("UNIX SOCKET PATH could not be removed");
        }

        let unix_listener =
            UnixListener::bind(SOCKET_PATH).expect("failed to create Unix Listener");

        let subapp_process = subapp_command
            .stdout(Stdio::inherit()) // subapp prints to same place as this
            .stderr(Stdio::inherit()) // subapp prints errors to same place as this
            .spawn()
            .expect("failed to spawn subapp process from executable path");

        let (subapp_stream, _) = unix_listener
            .accept()
            .expect("failed to accept connection with subapp process");

        Self {
            subapp_process,
            unix_listener,
            subapp_stream,
        }
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

    fn inform_event(&mut self, event: super::Event) {
        // self.subapp_stream
        //     .write_all(event)
        //     .expect("failed write to subapp process");
        todo!()
    }

    fn get_request(&mut self) -> super::Request {
        let mut message_length = [0; 2];

        self.subapp_stream
            .read_exact(&mut message_length)
            .expect("failed to read message length from subapp process");

        dbg!(message_length);
        let message_length = usize::from(u16::from_be_bytes(message_length));

        let mut message_buffer = vec![0; message_length];

        dbg!();

        self.subapp_stream
            .read_exact(&mut message_buffer)
            .expect("failed to read output from subapp process");
        dbg!();

        String::from_utf8(message_buffer).unwrap()
    }
}
impl Drop for ExecutableSubappHandler {
    fn drop(&mut self) {
        self.subapp_process
            .kill()
            .expect("command could not be killed");
    }
}
