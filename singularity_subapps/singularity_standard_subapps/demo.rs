// use singularity_common::utils::object_stream::{ObjectInputStream, ObjectOutputStream};
// use std::{os::unix::net::UnixStream, thread::sleep, time::Duration};

// // TODO: move the unix stuff to a ipc subapp client tool
// pub struct DemoIpcClient {
//     manager_stream: UnixStream,
// }
// impl Default for DemoIpcClient {
//     fn default() -> Self {
//         Self::new(SOCKET_PATH)
//     }
// }
// impl DemoIpcClient {
//     pub fn new(path: &str) -> Self {
//         let manager_stream = UnixStream::connect(path).unwrap();

//         Self { manager_stream }
//     }

//     pub fn get_string(&mut self) -> String {
//         self.manager_stream.read_object()
//     }

//     pub fn send_message(&mut self, message: String) {
//         self.manager_stream.write_object(&message)
//     }
// }

fn main() {
    //     // this will print directly to terminal - for now
    //     println!("Hi direct to stdio from subapp");

    //     let mut client = DemoIpcClient::default();

    //     dbg!();

    //     dbg!(client.send_message(String::from("Hi\n")));
    //     // dbg!(client.send_message(b"Hi from subapp via socket\n"));
    //     // dbg!(client.get_string());

    //     // dbg!(UNIX_SOCKET_PATH);
    //     // let mut stream = UnixStream::connect(UNIX_SOCKET_PATH).unwrap();

    //     // dbg!(stream.set_read_timeout(Some(Duration::new(1, 0))));
    //     // dbg!(stream.write_all(b"Hi from subapp via socket\n"));

    //     // just a logged infinite loop
    //     // FIXME: currently, children run forever.
    //     for i in 0.. {
    //         dbg!(format!("Subapp infinite loop {}", i));

    //         sleep(Duration::from_secs(1));
    //     }
}
