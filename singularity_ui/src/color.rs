// TODO: themes

/// RGBA
/// NOTE: NOT ARGB!
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub [u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Color([0, 0, 0, 0]);
    pub const BLACK: Self = Color([0, 0, 0, 0xFF]);
    pub const WHITE: Self = Color([0xFF, 0xFF, 0xFF, 0xFF]);
    pub const LIGHT_GRAY: Self = Color([0xDF, 0xDF, 0xDF, 0xFF]);
    pub const MEDIUM_GRAY: Self = Color([0x7F, 0x7F, 0x7F, 0xFF]);
    pub const DARK_GRAY: Self = Color([0x1F, 0x1F, 0x1F, 0xFF]);
    pub const LIGHT_YELLOW: Self = Color([0xFF, 0xFF, 0, 0xFF]);
    pub const LIGHT_GREEN: Self = Color([0, 0xFF, 0, 0xFF]);
    pub const LIGHT_BLUE: Self = Color([0, 0, 0xFF, 0xFF]);
    pub const ORANGE: Self = Color([0xFF, 0xA5, 0, 0xFF]);
    pub const CYAN: Self = Color([0, 0xFF, 0xFF, 0xFF]);
    pub const RED: Self = Color([0xFF, 0x00, 0x00, 0xFF]);

    pub const fn to_argb_u32(self) -> u32 {
        let Self([r, g, b, a]) = self;
        u32::from_be_bytes([a, r, g, b])
    }

    pub const fn to_rgba_u32(self) -> u32 {
        u32::from_be_bytes(self.0)
    }
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
#[cfg(feature = "winit_backend")]
impl From<Color> for glyphon::Color {
    /// TODO: figure out most idiomatic way of dealing with these references for copy-able types.
    fn from(value: Color) -> Self {
        glyphon::Color(value.to_argb_u32())
    }
}
