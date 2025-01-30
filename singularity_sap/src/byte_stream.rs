use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

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

    /// Blocks the thread until the first object is sent
    fn wait_read_bytes(&mut self) -> Vec<u8>;

    /// Look at [`std::sync::mpsc::Receiver::try_iter`]
    fn try_iter_bytes(&mut self) -> TryIter<Self>
    where
        Self: Sized,
    {
        TryIter { reader: self }
    }

    /// Maybe not the most performant, but whatever
    fn collect_try_iter_bytes(&mut self) -> Vec<Vec<u8>>
    where
        Self: Sized,
    {
        self.try_iter_bytes().collect()
    }
}
impl ByteReader for UnixStream {
    /// Returns [`None`] if the socket was timed out,
    /// if it is some other error, it panics.
    ///
    /// I want to handle timeouts like [`std::sync::mpsc::Receiver`]
    fn try_read_bytes(&mut self) -> Option<Vec<u8>> {
        self.set_nonblocking(true)
            .expect("Couldn't set nonblocking");

        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            // TODO: I am just going to assume Err return is just
            // because of timeout.
            self.read_exact(&mut raw_message_length_buffer).ok()?;

            MessageLength::from_be_bytes(raw_message_length_buffer) as usize
        };

        let mut raw_message_buffer = vec![0u8; raw_message_length];
        // I am fine with panicing in this case,
        // because we are pretty screwed if only the message length was sent
        // TODO: add actual error handling and correction
        // also, unix sockets should be pretty stable, I don't think anything other
        // than the nonblocking will cause an error, and if it does, that is beyond
        // my current scope
        self.read_exact(&mut raw_message_buffer)
            .expect("failed to read message");

        Some(raw_message_buffer)
    }

    fn wait_read_bytes(&mut self) -> Vec<u8> {
        self.set_nonblocking(false)
            .expect("Couldn't set nonblocking to false");

        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            // TODO: I am just going to assume Err return is just
            // because of timeout.
            self.read_exact(&mut raw_message_length_buffer)
                .expect("Couldn't read in wait read bytes");

            MessageLength::from_be_bytes(raw_message_length_buffer) as usize
        };

        let mut raw_message_buffer = vec![0u8; raw_message_length];
        // I am fine with panicing in this case,
        // because we are pretty screwed if only the message length was sent
        // TODO: add actual error handling and correction
        // also, unix sockets should be pretty stable, I don't think anything other
        // than the nonblocking will cause an error, and if it does, that is beyond
        // my current scope
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

pub trait ByteStream: ByteReader + ByteWriter {}
impl<S: ByteReader + ByteWriter> ByteStream for S {}

pub trait ToData {
    /// NOTE: using the name to_data instead of to_bytes to decrease chance of name collision
    fn to_data(&self) -> Vec<u8>;
}
pub trait TryFromData: Sized {
    fn try_from_data(data: &[u8]) -> Option<Self>;
}
/// Word I made up
pub trait Datable: ToData + TryFromData {}
impl<D: ToData + TryFromData> Datable for D {}

mod std_impls {
    use super::{ToData, TryFromData};

    impl ToData for String {
        fn to_data(&self) -> Vec<u8> {
            self.as_bytes().to_vec()
        }
    }
    impl TryFromData for String {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            String::from_utf8(data.to_vec()).ok()
        }
    }
}
#[cfg(feature = "singularity_ui")]
mod singularity_ui_impls {
    use super::{ToData, TryFromData};

    impl ToData for singularity_ui::display_units::DisplayArea {
        fn to_data(&self) -> Vec<u8> {
            todo!()
        }
    }
    impl TryFromData for singularity_ui::display_units::DisplayArea {
        fn try_from_data(_data: &[u8]) -> Option<Self> {
            todo!()
        }
    }
}
