use std::io::{Read, Write};

/// I didn't make this usize because usize might be different per build (?) idk for sure
type MessageLength = u64;

pub struct TryIter<'a, R: 'a + ByteReader + Sized> {
    reader: &'a mut R,
}
impl<'a, R: 'a + ByteReader + Sized> Iterator for TryIter<'a, R> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.try_read_bytes()
    }
}

pub trait ByteReader {
    fn try_read_bytes(&mut self) -> Option<Vec<u8>>;

    fn try_iter_bytes(&mut self) -> TryIter<Self>
    where
        Self: Sized;
}
impl<R: Read> ByteReader for R {
    /// NOTE: Though technically abstract,
    /// this is really just for unix socket.
    ///
    /// Assumes: The socket should have been set to nonblocking already,
    /// so that this can be nonblocking.
    ///
    /// Returns [`None`] if the socket was timed out,
    /// if it is some other error, it panics.
    ///
    /// I want to handle timeouts like [`std::sync::mpsc::Receiver`]
    fn try_read_bytes(&mut self) -> Option<Vec<u8>> {
        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            self.read_exact(&mut raw_message_length_buffer)
                // .map_err(|e| e.) // TODO
                .expect("failed to read message length");

            MessageLength::from_be_bytes(raw_message_length_buffer) as usize
        };

        let mut raw_message_buffer = vec![0u8; raw_message_length];
        self.read_exact(&mut raw_message_buffer)
            .expect("failed to read message");

        Some(raw_message_buffer)
    }

    /// Look at [`std::sync::mpsc::Receiver::try_iter`]
    fn try_iter_bytes(&mut self) -> TryIter<Self>
    where
        Self: Sized,
    {
        TryIter { reader: self }
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
