// Recursive expansion of singularity_macros::Datable macro
// =========================================================

const _: () = {
    #[automatically_derived]
    impl ToData for UIElement {
        fn to_data(&self) -> Vec<u8> {
            let (id, inner_data) = match self {
                Self::Container(inner_packet) => (0usize, inner_packet.to_data()),
                Self::Contained(inner_packet) => (1usize, inner_packet.to_data()),
                Self::Bordered(inner_packet) => (2usize, inner_packet.to_data()),
                Self::Backgrounded(inner_packet) => (3usize, inner_packet.to_data()),
                Self::Text(inner_packet) => (4usize, inner_packet.to_data()),
                Self::CharGrid(inner_packet) => (5usize, inner_packet.to_data()),
                Self::Nothing(inner_packet) => (6usize, inner_packet.to_data()),
            };
            let id_bytes: &[u8] = &id.to_be_bytes();
            [id_bytes, &inner_data].concat()
        }
    }
    #[automatically_derived]
    impl TryFromData for UIElement {
        fn try_from_data(data: &[u8]) -> Option<Self> {
            let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
            let id = usize::from_be_bytes(id_bytes.try_into().unwrap());
            match id {
                0usize => Some(Self::Container(
                    <Vec<UIElement> as TryFromData>::try_from_data(inner_data)?,
                )),
                1usize => Some(Self::Contained(<(
                    Box<UIElement>,
                    singularity_ui::display_units::DisplayArea,
                ) as TryFromData>::try_from_data(
                    inner_data
                )?)),
                2usize => Some(Self::Bordered(<(
                    Box<UIElement>,
                    singularity_ui::color::Color,
                ) as TryFromData>::try_from_data(
                    inner_data
                )?)),
                3usize => Some(Self::Backgrounded(<(
                    Box<UIElement>,
                    singularity_ui::color::Color,
                ) as TryFromData>::try_from_data(
                    inner_data
                )?)),
                4usize => Some(Self::Text(<String as TryFromData>::try_from_data(
                    inner_data,
                )?)),
                5usize => Some(Self::CharGrid(
                    <singularity_ui::ui_element::CharGrid as TryFromData>::try_from_data(
                        inner_data,
                    )?,
                )),
                6usize => Some(Self::Nothing(<() as TryFromData>::try_from_data(
                    inner_data,
                )?)),
                _ => None,
            }
        }
    }
};
