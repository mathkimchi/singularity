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

    pub fn pixels(&self, container_pixels: i32) -> i32 {
        match self {
            Self::Pixels(pixels) => *pixels,
            Self::Proportional(proportion) => (container_pixels as f32 * proportion) as i32,
            Self::MixedUnits { pixels, proportion } => {
                // in essense, you can think of the proportional being applied first
                Self::Pixels(*pixels).pixels(container_pixels)
                    + Self::Proportional(*proportion).pixels(container_pixels)
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

        let (container_len_px, container_len_pr) = (container_max - container_min).components();
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
impl std::ops::Neg for DisplayUnits {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Pixels(pixels) => Self::Pixels(-pixels),
            Self::Proportional(proportion) => Self::Proportional(-proportion),
            Self::MixedUnits { pixels, proportion } => Self::from_mixed(-pixels, -proportion),
        }
    }
}
impl std::ops::Add for DisplayUnits {
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
impl std::ops::Sub for DisplayUnits {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        // kinda dumb that implementing Add and Neg doesn't auto impl this
        self + (-rhs)
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

    /// Use: `child_area.map_onto(parent_area)`
    pub fn map_onto(&self, container_area: Self) -> Self {
        Self(
            self.0.map_onto(container_area),
            self.1.map_onto(container_area),
        )
    }

    pub fn contains(&self, coord: DisplayCoord, container_pixels: [i32; 2]) -> bool {
        let coord_x = coord.x.pixels(container_pixels[0]);
        let coord_y = coord.y.pixels(container_pixels[1]);

        // REVIEW: I think doing the == instead of && should handle cases where 0 and 1 aren't min and max respectively
        let contains_x = (self.0.x.pixels(container_pixels[0]) <= coord_x)
            == (coord_x <= self.1.x.pixels(container_pixels[0]));
        let contains_y = (self.0.y.pixels(container_pixels[1]) <= coord_y)
            == (coord_y <= self.1.y.pixels(container_pixels[1]));

        contains_x && contains_y
    }
}
