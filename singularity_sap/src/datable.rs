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
    use std::{
        ffi::OsString,
        os::unix::ffi::{OsStrExt, OsStringExt},
    };

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

    impl<T: ToData> ToData for Option<T> {
        fn to_data(&self) -> Vec<u8> {
            let (id, inner_data) = match self {
                // Technically inefficient
                Self::None => (0usize, Vec::new()),
                Self::Some(inner_packet) => (1usize, inner_packet.to_data()),
            };
            let id_bytes: &[u8] = &id.to_be_bytes();
            [id_bytes, &inner_data].concat()
        }
    }
    impl<T: TryFromData> TryFromData for Option<T> {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
            let id = usize::from_be_bytes(id_bytes.try_into().unwrap());
            match id {
                0usize => Some(Self::None),
                1usize => Some(Self::Some(<T as TryFromData>::try_from_data(inner_data)?)),
                _ => None,
            }
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

    impl ToData for OsString {
        fn to_data(&self) -> Vec<u8> {
            self.as_bytes().to_vec()
        }
    }
    impl TryFromData for OsString {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            Some(OsString::from_vec(data.to_vec()))
        }
    }

    impl ToData for bool {
        fn to_data(&self) -> Vec<u8> {
            match self {
                true => vec![0x01],
                false => vec![0x00],
            }
        }
    }
    impl TryFromData for bool {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            match data.iter().as_slice() {
                [0x01] => Some(true),
                [0x00] => Some(false),
                _ => None,
            }
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

    impl ToData for char {
        fn to_data(&self) -> Vec<u8> {
            (*self as u32).to_data()
        }
    }
    impl TryFromData for char {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            char::from_u32(u32::try_from_data(data)?)
        }
    }

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
#[cfg(feature = "singularity_common")]
mod singularity_common_impls {
    use super::{ToData, TryFromData};

    impl ToData for singularity_common::utils::tree::tree_node_path::TreeNodePath {
        fn to_data(&self) -> Vec<u8> {
            self.0.to_data()
        }
    }
    impl TryFromData for singularity_common::utils::tree::tree_node_path::TreeNodePath {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            Some(Self(Vec::<usize>::try_from_data(data)?))
        }
    }
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

    impl ToData for singularity_ui::color::Color {
        fn to_data(&self) -> Vec<u8> {
            let bytes_0 = self.0.to_data();
            let len_0 = bytes_0.len().to_be_bytes();
            [len_0.as_slice(), bytes_0.as_slice()].concat()
        }
    }
    impl TryFromData for singularity_ui::color::Color {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self({
                let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                index += 8;
                let inner_data = &data[index..(index + len)];
                index += len;
                <[u8; 4]>::try_from_data(inner_data)?
            });
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_element::CharCell {
        fn to_data(&self) -> Vec<u8> {
            let bytes_character = self.character.to_data();
            let len_character = bytes_character.len().to_be_bytes();
            let bytes_fg = self.fg.to_data();
            let len_fg = bytes_fg.len().to_be_bytes();
            let bytes_bg = self.bg.to_data();
            let len_bg = bytes_bg.len().to_be_bytes();
            [
                len_character.as_slice(),
                bytes_character.as_slice(),
                len_fg.as_slice(),
                bytes_fg.as_slice(),
                len_bg.as_slice(),
                bytes_bg.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_ui::ui_element::CharCell {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                character: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    char::try_from_data(inner_data)?
                },
                fg: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::color::Color::try_from_data(inner_data)?
                },
                bg: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    singularity_ui::color::Color::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_element::CharGrid {
        fn to_data(&self) -> Vec<u8> {
            let bytes_content = self.content.to_data();
            let len_content = bytes_content.len().to_be_bytes();
            [len_content.as_slice(), bytes_content.as_slice()].concat()
        }
    }
    impl TryFromData for singularity_ui::ui_element::CharGrid {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                content: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <Vec<Vec<singularity_ui::ui_element::CharCell>>>::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_element::UIElement {
        fn to_data(&self) -> Vec<u8> {
            let (id, inner_data) = match self {
                Self::Container(inner_packet) => (0usize, inner_packet.to_data()),
                Self::Contained(inner_packet_0, inner_packet_1) => {
                    (1usize, (inner_packet_0, inner_packet_1).to_data())
                }
                Self::Bordered(inner_packet_0, inner_packet_1) => {
                    (2usize, (inner_packet_0, inner_packet_1).to_data())
                }
                Self::Backgrounded(inner_packet_0, inner_packet_1) => {
                    (3usize, (inner_packet_0, inner_packet_1).to_data())
                }
                Self::Text(inner_packet) => (4usize, inner_packet.to_data()),
                Self::CharGrid(inner_packet) => (5usize, inner_packet.to_data()),
                Self::Nothing => (6usize, ().to_data()),
            };
            let id_bytes: &[u8] = &id.to_be_bytes();
            [id_bytes, &inner_data].concat()
        }
    }
    impl TryFromData for singularity_ui::ui_element::UIElement {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
            let id = usize::from_be_bytes(id_bytes.try_into().unwrap());
            match id {
                0usize => Some(Self::Container(
                    <Vec<singularity_ui::ui_element::UIElement> as TryFromData>::try_from_data(
                        inner_data,
                    )?,
                )),
                1usize => {
                    let (inner_data_0, inner_data_1) =
                        <(
                            Box<singularity_ui::ui_element::UIElement>,
                            singularity_ui::display_units::DisplayArea,
                        ) as TryFromData>::try_from_data(inner_data)?;

                    Some(Self::Contained(inner_data_0, inner_data_1))
                }
                2usize => {
                    let (inner_data_0, inner_data_1) =
                        <(
                            Box<singularity_ui::ui_element::UIElement>,
                            singularity_ui::color::Color,
                        ) as TryFromData>::try_from_data(inner_data)?;

                    Some(Self::Bordered(inner_data_0, inner_data_1))
                }
                3usize => {
                    let (inner_data_0, inner_data_1) =
                        <(
                            Box<singularity_ui::ui_element::UIElement>,
                            singularity_ui::color::Color,
                        ) as TryFromData>::try_from_data(inner_data)?;

                    Some(Self::Backgrounded(inner_data_0, inner_data_1))
                }
                4usize => Some(Self::Text(<String as TryFromData>::try_from_data(
                    inner_data,
                )?)),
                5usize => Some(Self::CharGrid(
                    <singularity_ui::ui_element::CharGrid as TryFromData>::try_from_data(
                        inner_data,
                    )?,
                )),
                6usize => Some(Self::Nothing),
                _ => None,
            }
        }
    }

    impl ToData for singularity_ui::ui_event::Key {
        fn to_data(&self) -> Vec<u8> {
            let bytes_time = self.time.to_data();
            let len_time = bytes_time.len().to_be_bytes();
            let bytes_raw_code = self.raw_code.to_data();
            let len_raw_code = bytes_raw_code.len().to_be_bytes();
            let bytes_keysym = self.keysym.raw().to_data();
            let len_keysym = bytes_keysym.len().to_be_bytes();
            let bytes_utf8 = self.utf8.to_data();
            let len_utf8 = bytes_utf8.len().to_be_bytes();
            [
                len_time.as_slice(),
                bytes_time.as_slice(),
                len_raw_code.as_slice(),
                bytes_raw_code.as_slice(),
                len_keysym.as_slice(),
                bytes_keysym.as_slice(),
                len_utf8.as_slice(),
                bytes_utf8.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_ui::ui_event::Key {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                time: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <u32>::try_from_data(inner_data)?
                },
                raw_code: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <u32>::try_from_data(inner_data)?
                },
                keysym: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <u32>::try_from_data(inner_data)?.into()
                },
                utf8: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <Option<String>>::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_event::KeyModifiers {
        fn to_data(&self) -> Vec<u8> {
            let bytes_ctrl = self.ctrl.to_data();
            let len_ctrl = bytes_ctrl.len().to_be_bytes();
            let bytes_alt = self.alt.to_data();
            let len_alt = bytes_alt.len().to_be_bytes();
            let bytes_shift = self.shift.to_data();
            let len_shift = bytes_shift.len().to_be_bytes();
            let bytes_caps_lock = self.caps_lock.to_data();
            let len_caps_lock = bytes_caps_lock.len().to_be_bytes();
            let bytes_logo = self.logo.to_data();
            let len_logo = bytes_logo.len().to_be_bytes();
            let bytes_num_lock = self.num_lock.to_data();
            let len_num_lock = bytes_num_lock.len().to_be_bytes();
            [
                len_ctrl.as_slice(),
                bytes_ctrl.as_slice(),
                len_alt.as_slice(),
                bytes_alt.as_slice(),
                len_shift.as_slice(),
                bytes_shift.as_slice(),
                len_caps_lock.as_slice(),
                bytes_caps_lock.as_slice(),
                len_logo.as_slice(),
                bytes_logo.as_slice(),
                len_num_lock.as_slice(),
                bytes_num_lock.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_ui::ui_event::KeyModifiers {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                ctrl: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
                alt: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
                shift: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
                caps_lock: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
                logo: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
                num_lock: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <bool>::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_ui::ui_event::UIEvent {
        fn to_data(&self) -> Vec<u8> {
            let (id, inner_data) = match self {
                Self::KeyPress(i0, i1) => (0usize, (i0, i1).to_data()),
                Self::WindowResized(inner_packet) => (1usize, inner_packet.to_data()),
                Self::MousePress(i0, i1) => (2usize, (i0, i1).to_data()),
            };
            let id_bytes: &[u8] = &id.to_be_bytes();
            [id_bytes, &inner_data].concat()
        }
    }
    impl TryFromData for singularity_ui::ui_event::UIEvent {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
            let id = usize::from_be_bytes(id_bytes.try_into().unwrap());
            match id {
                0usize => {
                    // TODO: macro for expanding tuples, python style: `*arg`
                    let comps = <(
                        singularity_ui::ui_event::Key,
                        singularity_ui::ui_event::KeyModifiers,
                    ) as TryFromData>::try_from_data(inner_data)?;
                    Some(Self::KeyPress(comps.0, comps.1))
                }
                1usize => Some(Self::WindowResized(
                    <[u32; 2] as TryFromData>::try_from_data(inner_data)?,
                )),
                2usize => {
                    let comps=   <(
                [[u32; 2]; 2],
                singularity_ui::display_units::DisplayArea,
            ) as TryFromData>::try_from_data(
                inner_data
            )?;
                    Some(Self::MousePress(comps.0, comps.1))
                }
                _ => None,
            }
        }
    }
    impl PacketTrait for singularity_ui::ui_event::UIEvent {
        /// I just mashed my keyboard
        const PACKET_TYPE_ID: crate::packet::IdType = 3159320418745789;
    }
}

#[cfg(feature = "singularity_sporg")]
mod singularity_sporg_impls {
    use super::{ToData, TryFromData};
    use std::ffi::OsString;

    impl ToData for singularity_sporg::project_settings::TabSpawnCommand {
        fn to_data(&self) -> Vec<u8> {
            let bytes_program = self.program.to_data();
            let len_program = bytes_program.len().to_be_bytes();
            let bytes_args = self.args.to_data();
            let len_args = bytes_args.len().to_be_bytes();
            [
                len_program.as_slice(),
                bytes_program.as_slice(),
                len_args.as_slice(),
                bytes_args.as_slice(),
            ]
            .concat()
        }
    }
    #[automatically_derived]
    impl TryFromData for singularity_sporg::project_settings::TabSpawnCommand {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                program: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <OsString>::try_from_data(inner_data)?
                },
                args: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <Vec<OsString>>::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }

    impl ToData for singularity_sporg::project_settings::TabData {
        fn to_data(&self) -> Vec<u8> {
            let bytes_tab_command = self.tab_command.to_data();
            let len_tab_command = bytes_tab_command.len().to_be_bytes();
            let bytes_session_data = self.session_data.to_data();
            let len_session_data = bytes_session_data.len().to_be_bytes();
            [
                len_tab_command.as_slice(),
                bytes_tab_command.as_slice(),
                len_session_data.as_slice(),
                bytes_session_data.as_slice(),
            ]
            .concat()
        }
    }
    impl TryFromData for singularity_sporg::project_settings::TabData {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let mut index = 0;
            let constructed_self = Self {
                tab_command: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <singularity_sporg::project_settings::TabSpawnCommand>::try_from_data(
                        inner_data,
                    )?
                },
                session_data: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;
                    let inner_data = &data[index..(index + len)];
                    index += len;
                    <serde_json::Value>::try_from_data(inner_data)?
                },
            };
            if index != data.len() {
                return None;
            }
            Some(constructed_self)
        }
    }
}

#[cfg(feature = "serde_json")]
mod serde_json_impls {
    use super::{ToData, TryFromData};

    impl ToData for serde_json::Value {
        fn to_data(&self) -> Vec<u8> {
            self.to_string().to_data()
        }
    }
    impl TryFromData for serde_json::Value {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            serde_json::from_str(&String::try_from_data(data)?).ok()?
        }
    }
}
