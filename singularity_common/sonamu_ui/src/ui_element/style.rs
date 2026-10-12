//! Own version of `taffy::Style` because taffy's lengths are `!Send` (they store a `*const ()`).
//! Taffy's plain enums and generic containers are `Send`, so those are reused.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Length {
    Px(f32),
    /// 0 to 1
    Percent(f32),
}
impl From<Length> for taffy::LengthPercentage {
    fn from(value: Length) -> Self {
        match value {
            Length::Px(px) => Self::length(px),
            Length::Percent(percent) => Self::percent(percent),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthAuto {
    Px(f32),
    /// 0 to 1
    Percent(f32),
    Auto,
}
impl From<LengthAuto> for taffy::LengthPercentageAuto {
    fn from(value: LengthAuto) -> Self {
        match value {
            LengthAuto::Px(px) => Self::length(px),
            LengthAuto::Percent(percent) => Self::percent(percent),
            LengthAuto::Auto => Self::auto(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Px(f32),
    /// 0 to 1
    Percent(f32),
    Auto,
}
impl From<Dimension> for taffy::Dimension {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Px(px) => Self::length(px),
            Dimension::Percent(percent) => Self::percent(percent),
            Dimension::Auto => Self::auto(),
        }
    }
}

fn map_size<T, U: From<T>>(size: taffy::Size<T>) -> taffy::Size<U> {
    taffy::Size {
        width: size.width.into(),
        height: size.height.into(),
    }
}
fn map_rect<T, U: From<T>>(rect: taffy::Rect<T>) -> taffy::Rect<U> {
    taffy::Rect {
        left: rect.left.into(),
        right: rect.right.into(),
        top: rect.top.into(),
        bottom: rect.bottom.into(),
    }
}

/// Subset of `taffy::Style`, look at taffy docs for what each field does.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub display: taffy::Display,
    pub position: taffy::Position,
    pub inset: taffy::Rect<LengthAuto>,
    pub size: taffy::Size<Dimension>,
    // TODO: border
    // pub border: taffy::Rect<Length>,
    pub flex_direction: taffy::FlexDirection,
    pub flex_grow: f32,
}
impl Style {
    /// Takes up the full size of the container
    #[must_use]
    pub fn full_size() -> Self {
        Self {
            size: taffy::Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            ..Default::default()
        }
    }
}
/// Same as taffy's defaults
impl Default for Style {
    fn default() -> Self {
        Self {
            display: taffy::Display::default(),
            position: taffy::Position::default(),
            inset: taffy::Rect {
                left: LengthAuto::Auto,
                right: LengthAuto::Auto,
                top: LengthAuto::Auto,
                bottom: LengthAuto::Auto,
            },
            size: taffy::Size {
                width: Dimension::Auto,
                height: Dimension::Auto,
            },
            flex_direction: taffy::FlexDirection::default(),
            flex_grow: 0.0,
        }
    }
}
impl From<&Style> for taffy::Style {
    fn from(style: &Style) -> Self {
        Self {
            display: style.display,
            position: style.position,
            inset: map_rect(style.inset),
            size: map_size(style.size),
            flex_direction: style.flex_direction,
            flex_grow: style.flex_grow,
            ..Self::DEFAULT
        }
    }
}
