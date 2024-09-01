use singularity_common::subapp_handler::ipc_subapp_handler::SOCKET_PATH;
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    thread::sleep,
    time::Duration,
};

// TODO: move the unix stuff to a ipc subapp client tool
pub struct DemoIpcClient {
    manager_stream: UnixStream,
}
impl Default for DemoIpcClient {
    fn default() -> Self {
        Self::new(SOCKET_PATH)
    }
}
impl DemoIpcClient {
    pub fn new(path: &str) -> Self {
        let manager_stream = UnixStream::connect(path).unwrap();

        Self { manager_stream }
    }

    pub fn get_string(&mut self) -> String {
        let mut buffer = String::new();
        self.manager_stream
            .read_to_string(&mut buffer)
            .expect("failed to read message");
        buffer
    }

    pub fn send_message(&mut self, message: &[u8]) {
        let message_len = (message.len() as u16).to_be_bytes();

        dbg!(message_len);

        self.manager_stream
            .write_all(&message_len)
            .expect("failed to send message length");

        self.manager_stream
            .write_all(message)
            .expect("failed to send message");

        // bruh, write_all doesn't automatically flush
        self.manager_stream.flush().unwrap();
    }
}

fn main() {
    // this will print directly to terminal - for now
    println!("Hi direct to stdio from subapp");

    let mut client = DemoIpcClient::default();

    dbg!();

    dbg!(client.send_message(b"Hi from subapp via socket\n"));
    // dbg!(client.get_string());

    // dbg!(UNIX_SOCKET_PATH);
    // let mut stream = UnixStream::connect(UNIX_SOCKET_PATH).unwrap();

    // dbg!(stream.set_read_timeout(Some(Duration::new(1, 0))));
    // dbg!(stream.write_all(b"Hi from subapp via socket\n"));

    // just a logged infinite loop
    // FIXME: currently, children run forever.
    for i in 0.. {
        dbg!("Subapp infinite loop", i);

        sleep(Duration::from_secs(1));
    }
}
