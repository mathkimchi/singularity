use std::{
    io::{Read, Stdin, Write},
    os::unix::net::UnixStream,
    sync::mpsc::{channel, Receiver},
    thread::{spawn, JoinHandle},
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
    /// TODO: rename to `poll`
    fn try_read_bytes(&mut self) -> Option<Vec<u8>>;

    /// Blocks the thread until the first object is sent
    fn wait_read_bytes(&mut self) -> Vec<u8>;

    /// Look at [`std::sync::mpsc::Receiver::try_iter`]
    /// TODO: rename to `poll_iter_bytes`
    fn try_iter_bytes(&mut self) -> impl Iterator<Item = Vec<u8>>
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

/// A wrapper for read that allows for polling.
/// WARNING: Creating this creates a new thread and uses mpsc.
pub struct ByteReaderWrapper {
    message_queue: Receiver<Vec<u8>>,
    _thread_handle: JoinHandle<()>,
}
impl ByteReaderWrapper {
    pub fn new<R: 'static + Read + Send>(mut read: R) -> Self {
        // recieve send is kind of like queues
        let (sender, reciever) = channel();
        let thread_handle = spawn(move || loop {
            // NOTE: I think this loop doesn't waste too much cpu,
            // since most time should be spent on read_exact.

            let raw_message_length = {
                let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];
                read.read_exact(&mut raw_message_length_buffer).unwrap();
                MessageLength::from_be_bytes(raw_message_length_buffer) as usize
            };

            let mut raw_message_buffer = vec![0u8; raw_message_length];
            read.read_exact(&mut raw_message_buffer)
                .expect("failed to read message");

            sender.send(raw_message_buffer).unwrap();
        });
        Self {
            message_queue: reciever,
            _thread_handle: thread_handle,
        }
    }
}
impl ByteReader for ByteReaderWrapper {
    fn try_read_bytes(&mut self) -> Option<Vec<u8>> {
        self.message_queue.try_recv().ok()
    }

    fn wait_read_bytes(&mut self) -> Vec<u8> {
        self.message_queue.recv().unwrap()
    }

    fn try_iter_bytes(&mut self) -> impl Iterator<Item = Vec<u8>>
    where
        Self: Sized,
    {
        self.message_queue.try_iter()
    }
}
impl ByteReader for Stdin {
    fn try_read_bytes(&mut self) -> Option<Vec<u8>> {
        // self.l
        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            // TODO: I am just going to assume Err return is just
            // because of timeout.
            self.read_exact(&mut raw_message_length_buffer).ok()?;

            MessageLength::from_be_bytes(raw_message_length_buffer) as usize
        };

        let mut raw_message_buffer = vec![0u8; raw_message_length];
        self.read_exact(&mut raw_message_buffer)
            .expect("failed to read message");

        Some(raw_message_buffer)
    }

    fn wait_read_bytes(&mut self) -> Vec<u8> {
        let raw_message_length = {
            let mut raw_message_length_buffer = [0; (MessageLength::BITS / 8) as usize];

            self.read_exact(&mut raw_message_length_buffer)
                .expect("Couldn't read in wait read bytes");

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

pub trait ByteStream: ByteReader + ByteWriter {}
impl<S: ByteReader + ByteWriter> ByteStream for S {}
