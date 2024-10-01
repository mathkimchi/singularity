#[cfg(feature = "egui_backend")]
mod egui_backend;
#[cfg(feature = "iced_backend")]
mod iced_backend;
#[cfg(feature = "wayland_backend")]
mod wayland_backend;
#[cfg(not(any(
    feature = "egui_backend",
    feature = "iced_backend",
    feature = "wayland_backend"
)))]
compile_error!("need to choose a gui backend");

#[cfg(feature = "egui_backend")]
pub use egui_backend::*;

#[cfg(feature = "wayland_backend")]
pub use wayland_backend::*;

pub mod display_units {
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub enum DisplayUnits {
        Pixels(i32),
        /// 0 to 1
        /// REVIEW: there would be some benefits to making this an uint and dividing my the max int each time
        Proportional(f32),
        MixedUnits {
            pixels: i32,
            proportion: f32,
        },
    }
    mod display_units_impls {
        use super::DisplayUnits;
        use std::ops::{Add, Neg, Sub};

        impl DisplayUnits {
            pub const ZERO: DisplayUnits = DisplayUnits::Pixels(0);
            pub const FULL: DisplayUnits = DisplayUnits::Proportional(1.0);

            pub fn pixels(&self, container_size: i32) -> i32 {
                match self {
                    Self::Pixels(pixels) => *pixels,
                    Self::Proportional(proportion) => (container_size as f32 * proportion) as i32,
                    Self::MixedUnits { pixels, proportion } => {
                        Self::Pixels(*pixels).pixels(container_size)
                            + Self::Proportional(*proportion).pixels(container_size)
                    }
                }
            }

            pub fn mixed(&self) -> (i32, f32) {
                match self {
                    Self::Pixels(pixels) => (*pixels, 0.),
                    Self::Proportional(proportion) => (0, *proportion),
                    Self::MixedUnits { pixels, proportion } => (*pixels, *proportion),
                }
            }
        }
        impl From<f32> for DisplayUnits {
            fn from(value: f32) -> Self {
                Self::Proportional(value)
            }
        }
        impl From<i32> for DisplayUnits {
            fn from(value: i32) -> Self {
                Self::Pixels(value)
            }
        }
        impl Neg for DisplayUnits {
            type Output = Self;
            fn neg(self) -> Self::Output {
                match self {
                    Self::Pixels(pixels) => Self::Pixels(-pixels),
                    Self::Proportional(proportion) => Self::Proportional(-proportion),
                    Self::MixedUnits { pixels, proportion } => Self::MixedUnits {
                        pixels: -pixels,
                        proportion: -proportion,
                    },
                }
            }
        }
        impl Add for DisplayUnits {
            type Output = Self;
            fn add(self, rhs: Self) -> Self::Output {
                // If they are both same type, we can leave it like that but otherwise convert to mixed then add
                match (self, rhs) {
                    (Self::Pixels(l), Self::Pixels(r)) => Self::Pixels(l + r),
                    (Self::Proportional(l), Self::Proportional(r)) => Self::Proportional(l + r),
                    (l, r) => {
                        let (lpx, lpr) = l.mixed();
                        let (rpx, rpr) = r.mixed();
                        Self::MixedUnits {
                            pixels: lpx + rpx,
                            proportion: lpr + rpr,
                        }
                    }
                }
            }
        }
        impl Sub for DisplayUnits {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self::Output {
                // kinda dumb that implementing Add and Neg doesn't auto impl this
                self + (-rhs)
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct DisplaySize {
        pub width: DisplayUnits,
        pub height: DisplayUnits,
    }
    impl DisplaySize {
        pub const fn new(width: DisplayUnits, height: DisplayUnits) -> Self {
            DisplaySize { width, height }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct DisplayCoord {
        pub x: DisplayUnits,
        pub y: DisplayUnits,
    }
    impl DisplayCoord {
        pub const fn new(x: DisplayUnits, y: DisplayUnits) -> Self {
            DisplayCoord { x, y }
        }
    }

    /// technically, any opposite extremes should work,
    /// but usually do (upper left, lower right)
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct DisplayArea(pub DisplayCoord, pub DisplayCoord);
    impl DisplayArea {
        pub fn size(&self) -> DisplaySize {
            DisplaySize::new(self.1.x - self.0.x, self.1.y - self.0.y)
        }

        pub fn from_coord_size(coord: DisplayCoord, size: DisplaySize) -> Self {
            Self(
                coord,
                DisplayCoord::new(coord.x + size.width, coord.y + size.height),
            )
        }
    }

    #[cfg(feature = "egui_backend")]
    impl From<DisplaySize> for egui::Vec2 {
        fn from(value: DisplaySize) -> Self {
            egui::Vec2::new(value.width, value.height)
        }
    }

    #[cfg(feature = "egui_backend")]
    impl From<DisplayCoord> for egui::Pos2 {
        fn from(value: DisplayCoord) -> Self {
            egui::Pos2::new(value.x, value.y)
        }
    }

    #[cfg(feature = "egui_backend")]
    impl From<DisplayArea> for egui::Rect {
        fn from(value: DisplayArea) -> Self {
            egui::Rect::from_two_pos(value.0.into(), value.1.into())
        }
    }
}

#[derive(Debug, Clone)]
pub enum UIElement {
    Container(Vec<(UIElement, display_units::DisplayArea)>),
    // Horizontal(Vec<UIElement>),
    Bordered(Box<UIElement>),
    Text(String),

    /// should display like a terminal
    ///
    /// most important feature is that each character is the same size
    CharGrid(CharGrid),
}
impl UIElement {
    pub fn bordered(self) -> Self {
        Self::Bordered(Box::new(self))
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct CharCell {
    pub character: char,
    pub fg: Color,
    pub bg: Color,
}
impl CharCell {
    pub fn new(character: char) -> Self {
        CharCell {
            character,
            fg: Color::LIGHT_YELLOW,
            bg: Color::TRANSPARENT,
        }
    }
}

#[derive(Debug, Clone, Hash, Default)]
pub struct CharGrid {
    pub content: Vec<Vec<CharCell>>,
}
impl From<String> for CharGrid {
    fn from(raw_content: String) -> Self {
        let mut content = Vec::new();
        for line_str in raw_content.split('\n') {
            let mut line = Vec::new();
            for c in line_str.chars() {
                line.push(CharCell::new(c));
            }
            content.push(line);
        }

        CharGrid { content }
    }
}
impl CharGrid {
    pub fn get_text_as_string(&self) -> String {
        self.content
            .iter()
            .map(|line| {
                line.iter()
                    .map(|c| c.character.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
