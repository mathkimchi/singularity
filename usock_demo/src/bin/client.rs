use std::io::Write;
use usock_demo::{
    bytes_stream::{ByteReader, ByteWriter},
    connection::ServerHandle,
};

fn main() {
    let mut server = ServerHandle::connect_from_env().unwrap();

    loop {
        let user_input = {
            print!("> ");
            std::io::stdout().flush().unwrap();
            let mut buffer = String::new();
            std::io::stdin().read_line(&mut buffer).unwrap();
            buffer.trim_end().to_string()
        };

        server.stream.write_bytes(user_input.as_bytes());

        if user_input.starts_with("QUIT") {
            println!("Quittin");
            return;
        } else if user_input.starts_with("POST: ") {
        } else if user_input.starts_with("QUERY_SQUARE: ") {
            // query result is float on purpose to test non-string
            let query_result = f32::from_be_bytes(server.stream.read_bytes().try_into().unwrap());

            println!("SQUARE RESULT: {}", query_result);
        } else {
            println!("DON'T KNOW WHAT USER SAID...");
        }
    }
}
