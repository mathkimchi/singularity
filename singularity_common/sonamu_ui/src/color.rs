// TODO: themes

/// RGBA
/// NOTE: NOT ARGB!
#[repr(C)]
#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, bytemuck::Pod, bytemuck::Zeroable,
)]
pub struct Color(pub [u8; 4]);
impl Color {
    pub const TRANSPARENT: Self = Self([0, 0, 0, 0]);
    pub const BLACK: Self = Self([0, 0, 0, 0xFF]);
    pub const WHITE: Self = Self([0xFF, 0xFF, 0xFF, 0xFF]);
    pub const LIGHT_GRAY: Self = Self([0xDF, 0xDF, 0xDF, 0xFF]);
    pub const MEDIUM_GRAY: Self = Self([0x7F, 0x7F, 0x7F, 0xFF]);
    pub const DARK_GRAY: Self = Self([0x1F, 0x1F, 0x1F, 0xFF]);
    pub const LIGHT_YELLOW: Self = Self([0xFF, 0xFF, 0, 0xFF]);
    pub const LIGHT_GREEN: Self = Self([0, 0xFF, 0, 0xFF]);
    pub const LIGHT_BLUE: Self = Self([0, 0, 0xFF, 0xFF]);
    pub const ORANGE: Self = Self([0xFF, 0xA5, 0, 0xFF]);
    pub const CYAN: Self = Self([0, 0xFF, 0xFF, 0xFF]);
    pub const RED: Self = Self([0xFF, 0x00, 0x00, 0xFF]);

    #[must_use]
    pub const fn to_argb_u32(self) -> u32 {
        let Self([r, g, b, a]) = self;
        u32::from_be_bytes([a, r, g, b])
    }

    #[must_use]
    pub const fn to_rgba_u32(self) -> u32 {
        u32::from_be_bytes(self.0)
    }
}
#[cfg(feature = "winit_backend")]
impl From<Color> for glyphon::Color {
    /// TODO: figure out most idiomatic way of dealing with these references for copy-able types.
    fn from(value: Color) -> Self {
        Self(value.to_argb_u32())
    }
}
