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
    use paste::paste;

    impl<T: ToData> ToData for &T {
        fn to_data(&self) -> Vec<u8> {
            <T as ToData>::to_data(self)
        }
    }

    impl<T: ToData> ToData for Box<T> {
        fn to_data(&self) -> Vec<u8> {
            <T as ToData>::to_data(self)
        }
    }
    impl<T: TryFromData> TryFromData for Box<T> {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            Some(Box::new(T::try_from_data(data)?))
        }
    }

    impl<T: ToData> ToData for Vec<T> {
        fn to_data(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            for item in self.iter() {
                let item_bytes = item.to_data();
                let item_len = item_bytes.len().to_be_bytes();
                bytes.extend(item_len);
                bytes.extend(item_bytes);
            }
            bytes
        }
    }
    impl<T: TryFromData> TryFromData for Vec<T> {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let mut constructed_self = Self::new();
            while index < data.len() {
                let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                index += 8;
                let inner_data = &data[index..(index + len)];
                index += len;
                let constructed_item = T::try_from_data(inner_data)?;
                constructed_self.push(constructed_item);
            }
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    // TODO: this is the same as for Vec<T>
    impl<T: ToData, const N: usize> ToData for [T; N] {
        fn to_data(&self) -> Vec<u8> {
            let mut bytes = Vec::new();
            for item in self.iter() {
                let item_bytes = item.to_data();
                let item_len = item_bytes.len().to_be_bytes();
                bytes.extend(item_len);
                bytes.extend(item_bytes);
            }
            bytes
        }
    }
    impl<T: TryFromData, const N: usize> TryFromData for [T; N] {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            // NOTE: currently, this is the safest option I found, despite possibly being slow.
            // Mapping `[(); N]` might be faster, but to make it safe, I needed to use Option which I will have to reallocate anyways.
            Vec::try_from_data(data)?.try_into().ok()
        }
    }

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

    macro_rules! number_data_impl {
        ($T:ty) => {
            impl ToData for $T {
                fn to_data(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }
            }
            impl TryFromData for $T {
                fn try_from_data(data: &[u8]) -> Option<Self> {
                    Some(<$T>::from_le_bytes(data.try_into().unwrap()))
                }
            }
        };
        ($($T:ty),*) => {
            $(
                number_data_impl!($T);
            )*
        }
    }
    number_data_impl!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, f32, f64);

    /// FIXME: Extremely mildly annoying; I couldn't figure out a way to do this in order.
    macro_rules! tuple_impl {
        // Look at: https://doc.rust-lang.org/src/core/fmt/mod.rs.html#2628
        ($($REM_I:tt,)*) => {

            paste!(
                impl<$([<T $REM_I>]: ToData,)*> ToData for ($([<T $REM_I>],)*) {
                    fn to_data(&self) -> Vec<u8> {
                        $(
                            let [<bytes_ $REM_I>] = self.[<$REM_I>].to_data();
                            let [<len_ $REM_I>] = [<bytes_ $REM_I>].len().to_be_bytes();
                        )*
                        let slices: &[&[u8]] = &[
                            $(
                                &[<len_ $REM_I>],
                                &[<bytes_ $REM_I>],
                            )*
                        ];
                        slices.concat()
                    }
                }
            );
            paste!(
                impl<$([<T $REM_I>]: TryFromData,)*> TryFromData for ($([<T $REM_I>],)*) {
                    fn try_from_data(data: &[u8]) -> Option<Self> {
                        #[allow(unused_mut)] // for the `()` case
                        let mut index = 0;
                        let constructed_self = (
                            $({
                                let len = usize::from_be_bytes(data[index..(index + (usize::BITS as usize / 8))].try_into().ok()?);
                                index += (usize::BITS as usize / 8);
                                let inner_data = &data[index..(index + len)];
                                index += len;
                                [<T $REM_I>]::try_from_data(inner_data)?
                            },)*
                        );
                        if index != data.len() {
                            return None;
                        }
                        Some(constructed_self)
                    }
                }
            );
        };
    }

    tuple_impl!();
    tuple_impl!(0,);
    tuple_impl!(0, 1,);
    tuple_impl!(0, 1, 2,);
    tuple_impl!(0, 1, 2, 3,);
}
/// TODO: automate this somehow. Annoying I can't use derive for outer classes.
#[cfg(feature = "singularity_ui")]
mod singularity_ui_impls {
    use super::{ToData, TryFromData};
    use crate::packet::PacketTrait;

    impl ToData for singularity_ui::display_units::DisplayUnits {
        fn to_data(&self) -> Vec<u8> {
            let (id, inner_data) = match self {
                Self::Pixels(inner_packet) => (0usize, inner_packet.to_data()),
                Self::Proportional(inner_packet) => (1usize, inner_packet.to_data()),
                Self::MixedUnits { pixels, proportion } => (2usize, (pixels, proportion).to_data()),
            };
            let id_bytes: &[u8] = &id.to_be_bytes();
            [id_bytes, &inner_data].concat()
        }
    }
    impl TryFromData for singularity_ui::display_units::DisplayUnits {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
            let id = usize::from_be_bytes(id_bytes.try_into().unwrap());
            match id {
                0usize => Some(Self::Pixels(<i32 as TryFromData>::try_from_data(
                    inner_data,
                )?)),
                1usize => Some(Self::Proportional(<f32 as TryFromData>::try_from_data(
                    inner_data,
                )?)),
                2usize => {
                    let (pixels, proportion) =
                        <(i32, f32) as TryFromData>::try_from_data(inner_data)?;
                    Some(Self::MixedUnits { pixels, proportion })
                }
                _ => None,
            }
        }
    }

    impl ToData for singularity_ui::display_units::DisplayCoord {
        fn to_data(&self) -> Vec<u8> {
            let bytes_x = self.x.to_data();
            let len_x = bytes_x.len().to_be_bytes();
            let bytes_y = self.y.to_data();
            let len_y = bytes_y.len().to_be_bytes();
            [
                len_x.as_slice(),
                bytes_x.as_slice(),
                len_y.as_slice(),
                bytes_y.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_ui::display_units::DisplayCoord {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                x: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::display_units::DisplayUnits::try_from_data(inner_data)?
                },
                y: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::display_units::DisplayUnits::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::display_units::DisplayArea {
        fn to_data(&self) -> Vec<u8> {
            let bytes_0 = self.0.to_data();
            let len_0 = bytes_0.len().to_be_bytes();
            let bytes_1 = self.1.to_data();
            let len_1 = bytes_1.len().to_be_bytes();
            [
                len_0.as_slice(),
                bytes_0.as_slice(),
                len_1.as_slice(),
                bytes_1.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_ui::display_units::DisplayArea {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self(
                {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::display_units::DisplayCoord::try_from_data(inner_data)?
                },
                {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::display_units::DisplayCoord::try_from_data(inner_data)?
                },
            );
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_element::UIElement {
        fn to_data(&self) -> Vec<u8> {
            todo!()
        }
    }
    impl TryFromData for singularity_ui::ui_element::UIElement {
        fn try_from_data(_data: &[u8]) -> Option<Self> {
            todo!()
        }
    }

    impl ToData for singularity_ui::ui_event::UIEvent {
        fn to_data(&self) -> Vec<u8> {
            todo!()
        }
    }
    impl TryFromData for singularity_ui::ui_event::UIEvent {
        fn try_from_data(_data: &[u8]) -> Option<Self> {
            todo!()
        }
    }
    impl PacketTrait for singularity_ui::ui_event::UIEvent {
        /// I just mashed my keyboard
        const PACKET_TYPE_ID: crate::packet::IdType = 3159320418745789;
    }
}
