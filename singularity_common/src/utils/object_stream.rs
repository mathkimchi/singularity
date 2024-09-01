use serde::{Deserialize, Serialize};

/// Bi-directional stream for sending objects over a stream.
/// Objects need to be serde serializable and deserializable.
/// The reciever's deserializer should be compatible with
/// the sender's serializer.
///
/// ## Details:
///
/// Once the object is serialized into an array of u8,
/// two u8s are sent, representing the array's u16 size in big
/// endian. Then, the array is sent.
pub struct ObjectStream {}
impl ObjectStream {
    pub fn new() -> Self {
        todo!()
    }

    pub fn send<T: Serialize>(&mut self, object: T) {
        todo!()
    }

    pub fn get<'a, T: Deserialize<'a>>(&mut self) -> Option<T> {
        todo!()
    }
}
