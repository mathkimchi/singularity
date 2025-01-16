use std::{
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::Duration,
};

#[test]
fn thread_unix_socket_test() {
    let server_thread = thread::spawn(|| {
        println!("Starting client thread");
        std::io::stdout().flush().unwrap();

        // need to manually remove `./test_socket.sock` between tests
        let server_listener = UnixListener::bind("./test_socket.sock").unwrap();
        let (conn, _addr) = server_listener.accept().unwrap();

        // thread::sleep(Duration::from_nanos(1));
    });

    let client_thread = thread::spawn(|| {
        println!("Starting client thread");
        std::io::stdout().flush().unwrap();

        let conn = UnixStream::connect("./test_socket.sock").unwrap();

        println!("Client side: connected");
        std::io::stdout().flush().unwrap();
    });

    server_thread.join().unwrap();
    client_thread.join().unwrap();
}
