use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Objects need to be serde serializable and deserializable.
/// The reciever's deserializer should be compatible with
/// the sender's serializer.
///
/// ## Details:
///
/// Once the object is serialized into an array of u8,
/// two u8s are sent, representing the array's u16 size in big
/// endian. Then, the array is sent.
///
/// The array is currently in JSON format.
pub trait ObjectInputStream {
    fn read_object<T: for<'de> Deserialize<'de>>(&mut self) -> T;
}
impl<R: Read> ObjectInputStream for R {
    fn read_object<T: for<'de> Deserialize<'de>>(&mut self) -> T {
        let raw_message_length = {
            let mut raw_message_length_buffer = [0; 2];

            self.read_exact(&mut raw_message_length_buffer)
                .expect("failed to read message length from subapp process");

            usize::from(u16::from_be_bytes(raw_message_length_buffer))
        };

        let mut raw_message_buffer = vec![0; raw_message_length];
        self.read_exact(&mut raw_message_buffer)
            .expect("failed to read message from subapp process");

        serde_json::from_slice(&raw_message_buffer).expect("failed to deserialize object")
    }
}

pub trait ObjectOutputStream {
    fn write_object<T: Serialize>(&mut self, object: &T);
}
impl<W: Write> ObjectOutputStream for W {
    fn write_object<T: Serialize>(&mut self, object: &T) {
        let raw_object = serde_json::to_vec(object).expect("failed to serialize object");

        // send raw object length
        raw_object.len();

        // send raw object
        let raw_object_len = (raw_object.len() as u16).to_be_bytes();

        self.write_all(&raw_object_len)
            .expect("failed to send message length");

        self.write_all(&raw_object).expect("failed to send message");

        // Need to flush for buffered writers
        self.flush().unwrap();
    }
}
