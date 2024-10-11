// TODO: themes

/// RGBA
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Color([0, 0, 0, 0]);
    pub const BLACK: Self = Color([0, 0, 0, 0xFF]);
    pub const WHITE: Self = Color([0xFF, 0xFF, 0xFF, 0xFF]);
    pub const DARK_GRAY: Self = Color([0x1F, 0x1F, 0x1F, 0xFF]);
    pub const LIGHT_YELLOW: Self = Color([0xFF, 0xFF, 0, 0xFF]);
    pub const LIGHT_GREEN: Self = Color([0, 0xFF, 0, 0xFF]);
    pub const LIGHT_BLUE: Self = Color([0, 0, 0xFF, 0xFF]);
    pub const ORANGE: Self = Color([0xFF, 0xA5, 0, 0xFF]);
    pub const CYAN: Self = Color([0, 0xFF, 0xFF, 0xFF]);
}
#[cfg(feature = "wayland_backend")]
impl From<Color> for raqote::Color {
    fn from(value: Color) -> Self {
        // raqote is argb, but our color is rgba
        raqote::Color::new(value.0[3], value.0[0], value.0[1], value.0[2])
    }
}
#[cfg(feature = "wayland_backend")]
impl From<Color> for raqote::SolidSource {
    fn from(value: Color) -> Self {
        raqote::SolidSource {
            r: value.0[0],
            g: value.0[1],
            b: value.0[2],
            a: value.0[3],
        }
    }
}
