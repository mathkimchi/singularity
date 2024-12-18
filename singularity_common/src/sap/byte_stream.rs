use std::io::{Read, Write};

/// I didn't make this usize because usize might be different per build (?) idk for sure
type MessageLength = u64;

pub trait ByteReader {
    fn read_bytes(&mut self) -> Vec<u8>;
}
impl<R: Read> ByteReader for R {
    fn read_bytes(&mut self) -> Vec<u8> {
        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            self.read_exact(&mut raw_message_length_buffer)
                .expect("failed to read message length");

            MessageLength::from_be_bytes(raw_message_length_buffer) as usize
        };

        let mut raw_message_buffer = vec![0u8; raw_message_length];
        self.read_exact(&mut raw_message_buffer)
            .expect("failed to read message");

        raw_message_buffer
    }
}

pub trait ByteWriter {
    fn write_bytes(&mut self, bytes: &[u8]);
}
impl<W: Write> ByteWriter for W {
    fn write_bytes(&mut self, bytes: &[u8]) {
        let raw_object_len = (bytes.len() as MessageLength).to_be_bytes();

        // write bytes length
        self.write_all(&raw_object_len)
            .expect("failed to write message length");

        // Need to flush for buffered writers
        self.flush().unwrap();

        // write bytes
        self.write_all(bytes).expect("failed to write message");

        // Need to flush for buffered writers
        self.flush().unwrap();
    }
}
