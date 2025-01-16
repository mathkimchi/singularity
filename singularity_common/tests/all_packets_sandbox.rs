use std::os::unix::net::UnixStream;

pub struct ClientConnection {
    connection: UnixStream,

    event_queue: Vec<Vec<u8>>,
}
