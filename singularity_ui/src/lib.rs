#[cfg(feature = "egui_backend")]
mod egui_backend;
#[cfg(feature = "iced_backend")]
mod iced_backend;
pub mod task_logger;
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

#[cfg(test)]
mod test;

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
            pub const HALF: DisplayUnits = DisplayUnits::Proportional(0.5);
            pub const FULL: DisplayUnits = DisplayUnits::Proportional(1.0);

            pub fn from_mixed(pixels: i32, proportion: f32) -> Self {
                match (pixels, proportion) {
                    (pixels, 0.0) => Self::Pixels(pixels),
                    (0, proportion) => Self::Proportional(proportion),
                    (pixels, proportion) => Self::MixedUnits { pixels, proportion },
                }
            }

            pub fn pixels(&self, container_size: i32) -> i32 {
                match self {
                    Self::Pixels(pixels) => *pixels,
                    Self::Proportional(proportion) => (container_size as f32 * proportion) as i32,
                    Self::MixedUnits { pixels, proportion } => {
                        // in essense, you can think of the proportional being applied first
                        Self::Pixels(*pixels).pixels(container_size)
                            + Self::Proportional(*proportion).pixels(container_size)
                    }
                }
            }

            pub const fn components(&self) -> (i32, f32) {
                match self {
                    Self::Pixels(pixels) => (*pixels, 0.),
                    Self::Proportional(proportion) => (0, *proportion),
                    Self::MixedUnits { pixels, proportion } => (*pixels, *proportion),
                }
            }

            /// REVIEW
            pub fn map_onto(&self, container_min: Self, container_max: Self) -> Self {
                if let Self::Pixels(_) = self {
                    return container_min + *self;
                }

                let (container_len_px, container_len_pr) =
                    (container_max - container_min).components();
                let (unmapped_px, unmapped_pr) = self.components();

                // Math:
                // if we knew the `tot_px` and `container_len.pixels(tot_px)`, then we can the difference from min is:
                // `unmapped_px+unmapped_pr*container_len.pixels(tot_px)`
                // which =`unmapped_px+unmapped_pr*(container_len_px+container_len_pr*tot_px)`
                // =`unmapped_px+unmapped_pr*container_len_px+unmapped_pr*container_len_pr*tot_px`
                // =`(unmapped_px+unmapped_pr*container_len_px)+(unmapped_pr*container_len_pr)*tot_px`
                // =`(unmapped_px+unmapped_pr*container_len_px)+(unmapped_pr*container_len_pr)*tot_px`
                // so: delta_px=`unmapped_px+unmapped_pr*container_len_px` and
                // delta_pr=`unmapped_pr*container_len_pr`

                let (container_min_px, container_min_pr) = container_min.components();
                let delta_px = unmapped_px + ((unmapped_pr * container_len_px as f32) as i32);
                let delta_pr = unmapped_pr * container_len_pr;
                let final_px = container_min_px + delta_px;
                let final_pr = container_min_pr + delta_pr;

                Self::from_mixed(final_px, final_pr)
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
                    Self::MixedUnits { pixels, proportion } => {
                        Self::from_mixed(-pixels, -proportion)
                    }
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
                        let (lpx, lpr) = l.components();
                        let (rpx, rpr) = r.components();
                        Self::from_mixed(lpx + rpx, lpr + rpr)
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

        pub fn map_onto(&self, container_area: DisplayArea) -> Self {
            Self::new(
                self.x.map_onto(container_area.0.x, container_area.1.x),
                self.y.map_onto(container_area.0.y, container_area.1.y),
            )
        }

        #[cfg(feature = "wayland_backend")]
        pub fn into_point(&self, dt: &raqote::DrawTarget) -> raqote::Point {
            raqote::Point::new(
                self.x.pixels(dt.width()) as f32,
                self.y.pixels(dt.height()) as f32,
            )
        }
    }

    /// technically, any opposite extremes should work,
    /// but usually do (upper left, lower right)
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    pub struct DisplayArea(pub DisplayCoord, pub DisplayCoord);
    impl DisplayArea {
        pub const FULL: Self = Self(
            DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
            DisplayCoord::new(DisplayUnits::FULL, DisplayUnits::FULL),
        );

        pub fn size(&self) -> DisplaySize {
            DisplaySize::new(self.1.x - self.0.x, self.1.y - self.0.y)
        }

        pub fn from_corner_size(corner: DisplayCoord, size: DisplaySize) -> Self {
            Self(
                corner,
                DisplayCoord::new(corner.x + size.width, corner.y + size.height),
            )
        }

        pub fn from_center_half_size(center: DisplayCoord, half_size: DisplaySize) -> Self {
            Self(
                DisplayCoord::new(center.x - half_size.width, center.y - half_size.height),
                DisplayCoord::new(center.x + half_size.width, center.y + half_size.height),
            )
        }

        // pub fn make_absolute(
        //     &self,
        //     parent_absolute_area: ((i32, i32), (i32, i32)),
        //     absolute_size: (i32, i32),
        // ) -> ((i32, i32), (i32, i32)) {
        //     let (parent_top_left, parent_bot_right) = parent_absolute_area;
        //     let parent_size = (
        //         parent_bot_right.0 - parent_top_left.0,
        //         parent_bot_right.1 - parent_top_left.1,
        //     );
        //     let self_size = self.size();

        //     ((), (todo!(), todo!()))
        // }
        pub fn map_onto(&self, container_area: Self) -> Self {
            Self(
                self.0.map_onto(container_area),
                self.1.map_onto(container_area),
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
