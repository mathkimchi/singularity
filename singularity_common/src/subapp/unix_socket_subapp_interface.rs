use super::SubappInterface;
use crate::utils::object_stream::{ObjectInputStream, ObjectOutputStream};
use std::{
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
    process::{Child, Command, Stdio},
};

pub const SOCKET_PATH: &str = "/tmp/singularity_demo_socket.sock";

/// Uses pipes for basic communication.
pub struct UnixSocketSubappInterface {
    subapp_process: Child,
    unix_listener: UnixListener,
    subapp_stream: UnixStream,
}
impl UnixSocketSubappInterface {
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
impl SubappInterface for UnixSocketSubappInterface {
    fn inform_event(&mut self, event: super::Event) {
        self.subapp_stream.write_object(&event);
    }

    fn dump_requests(&mut self) -> Vec<super::Request> {
        // TODO
        vec![self.subapp_stream.read_object()]
    }
}
impl Drop for UnixSocketSubappInterface {
    fn drop(&mut self) {
        self.subapp_process
            .kill()
            .expect("command could not be killed");
    }
}
