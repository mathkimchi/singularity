use std::{io::Write, thread};
use usock_demo::{
    bytes_stream::{ByteReader, ByteWriter},
    connection::{ClientHandler, ServerHost},
};

fn main() {
    println!("Starting unix socket chat server...");
    let listening = ServerHost::bind_new().unwrap();
    println!("Started unix socket chat server.");

    let is_done = thread::spawn(move || loop {
        // Ctrl+C doesn't call drop

        let user_input = {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            buffer.trim_end().to_string()
        };

        dbg!(&user_input);
        if user_input.starts_with("QUIT") {
            return;
        }
    });

    listening
        .listener
        .set_nonblocking(true)
        .expect("Could not set listener to nonblocking");

    let mut client_number = 0;
    loop {
        let connection = listening.listener.accept();

        if let Ok((unix_stream, _)) = connection {
            let mut client = ClientHandler {
                stream: unix_stream,
            };
            println!("Client `{client_number}` connected.");
            thread::spawn(move || loop {
                let message = String::from_utf8(client.stream.read_bytes()).unwrap();
                println!("Client `{client_number}` sent message: `{message}`.");

                if message == "QUIT" {
                    println!("Client `{client_number}` quit.");
                    return;
                } else if message.starts_with("POST: ") {
                    println!(
                        "Client `{client_number}` posted: {}.",
                        message.split_at("POST: ".len()).1
                    );
                } else if message.starts_with("QUERY_SQUARE: ") {
                    // works with scientific notation, eg: ``
                    let x_str = message.split_at("QUERY_SQUARE: ".len()).1;
                    println!("Squaring `{x_str}`...");
                    let x: f32 = x_str.parse().unwrap();
                    let x2 = x * x;
                    println!("Sending client `{client_number}` {x}^2={x2}.");
                    client.stream.write_bytes(&x2.to_be_bytes());
                }
            });

            client_number += 1;
        } else {
            // there is no accept request rn
            if is_done.is_finished() {
                break;
            }
        }
    }
}
