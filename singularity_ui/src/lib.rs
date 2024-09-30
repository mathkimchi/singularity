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
    pub type DisplayUnits = f32;
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
    }

    #[cfg(feature = "wayland_backend")]
    impl From<DisplayArea> for raqote::IntRect {
        fn from(value: DisplayArea) -> Self {
            Self::new(value.0.into(), value.1.into())
        }
    }
    #[cfg(feature = "wayland_backend")]
    impl From<DisplayCoord> for raqote::IntPoint {
        fn from(value: DisplayCoord) -> Self {
            Self::new(value.x as i32, value.y as i32)
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
    impl DisplayArea {
        pub fn from_coord_size(coord: DisplayCoord, size: DisplaySize) -> Self {
            Self(
                coord,
                DisplayCoord::new(coord.x + size.width, coord.y + size.height),
            )
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
