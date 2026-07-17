use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize, DisplayCoord},
};

/// TODO: rename most everything here
/// TODO: have a UI Element vs UI Primitive (to allow for easier chunking in GPU) (Or just have a private UI Primitives and do a conversion in Winit drawing impls?)
#[derive(Debug, Clone, PartialEq)]
pub enum UIElement {
    /// Contains a list of inner elements.
    /// Overlapping is allowed.
    Container(Vec<Self>),

    /// contains inner element within a certain area
    ///
    /// elements that aren't contained should be assumed to take the entire space
    Contained(Box<Self>, DisplayArea),
    /// TODO: Fix the right and bottom borders.
    Bordered(Box<Self>, Color),
    /// TODO: better name
    Backgrounded(Box<Self>, Color),

    Text(Vec<(String, glyphon::AttrsOwned)>),

    /// should display like a terminal
    ///
    /// most important feature is that each character is the same size
    CharGrid(CharGrid),

    Image(image::RgbaImage),

    Nothing,
}
impl UIElement {
    #[deprecated(note = "Use `LayoutBuilder` instead")]
    #[must_use]
    pub fn contain(self, area: DisplayArea) -> Self {
        Self::Contained(Box::new(self), area)
    }
    #[deprecated(note = "Use `LayoutBuilder` instead")]
    #[must_use]
    pub fn bordered(self, border: Color) -> Self {
        Self::Bordered(Box::new(self), border)
    }
    #[must_use]
    pub fn fill_bg(self, bg: Color) -> Self {
        Self::Backgrounded(Box::new(self), bg)
    }

    /// Takes in a list of full-size elements and returns a combined ui element where they are equally spaced
    /// across the horizontal axis and take full height.
    /// TODO: take in horizontal vs vertical as input or make this into two functions
    pub fn combine_displays(subdisplays: impl ExactSizeIterator<Item = Self>) -> Self {
        // proportional units so widths out of 1
        let widths = 1. / subdisplays.len() as f32;
        Self::Container(
            subdisplays
                .into_iter()
                .enumerate()
                .map(|(i, subdisplay)| {
                    subdisplay
                        .bordered(Color::LIGHT_GREEN)
                        .contain(DisplayArea::new(
                            (widths * (i as f32), 0.),
                            (widths * ((i + 1) as f32), 1.),
                        ))
                })
                .collect(),
        )
    }

    /// For background, you must draw a rectangle
    #[must_use]
    pub fn glyphon_attr(fg: Color) -> glyphon::AttrsOwned {
        glyphon::AttrsOwned::new(
            &glyphon::Attrs::new()
                .family(glyphon::Family::Monospace)
                .color(fg.into()),
        )
    }

    #[must_use]
    pub fn inner_size_of_bordered(container_size: DisplayContainerSize) -> DisplayContainerSize {
        DisplayContainerSize {
            // TODO: figure out all the edge cases like this
            width: container_size.width.saturating_sub(2),
            height: container_size.height.saturating_sub(2),
        }
    }
}
impl From<Option<Self>> for UIElement {
    fn from(value: Option<Self>) -> Self {
        value.unwrap_or(Self::Nothing)
    }
}
impl From<String> for UIElement {
    fn from(raw_content: String) -> Self {
        Self::Text(vec![(raw_content, Self::glyphon_attr(Color::WHITE))])
    }
}

/// 0 and 1 are planned to be bold and italic
/// 2 is cursor line
#[repr(C)]
#[derive(
    Copy,
    Clone,
    bytemuck::Pod,
    bytemuck::Zeroable,
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
)]
pub struct CharCellStyle(pub u32);
impl CharCellStyle {
    pub const UNSTYLED: Self = Self(0);
    pub const CURSOR_LINE: Self = Self(1 << 2);
}
/// For me, add makes more sense than bitwise OR
impl std::ops::Add for CharCellStyle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        #[allow(clippy::suspicious_arithmetic_impl)]
        Self(self.0 | rhs.0)
    }
}
impl std::ops::AddAssign for CharCellStyle {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Made so this can instantly be turned to bytes
/// REVIEW: rename to just `CharCell`?
#[repr(C)]
#[derive(
    Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct InternalCharCell {
    pub character: u32,
    /// RGBA
    pub fg: Color,
    /// RGBA
    pub bg: Color,
    pub style: CharCellStyle,
}
impl InternalCharCell {
    pub const BYTES: usize = 16;

    pub fn set_char(&mut self, c: char) -> &mut Self {
        self.character = c as u32;
        self
    }
    pub fn set_fg(&mut self, fg: Color) -> &mut Self {
        self.fg = fg;
        self
    }
    pub fn set_bg(&mut self, bg: Color) -> &mut Self {
        self.bg = bg;
        self
    }

    pub fn add_style(&mut self, new_style: CharCellStyle) -> &mut Self {
        self.style += new_style;
        self
    }
    pub fn set_style(&mut self, new_style: CharCellStyle) -> &mut Self {
        self.style = new_style;
        self
    }
}
impl Default for InternalCharCell {
    fn default() -> Self {
        Self {
            character: u32::from(' '),
            fg: Color::WHITE,
            bg: Color::TRANSPARENT,
            style: CharCellStyle::UNSTYLED,
        }
    }
}

/// think this is height in pixels
/// TODO: make this not-so-hardcoded...
pub const FONT_SIZE_U: u32 = 46;
pub const FONT_SIZE: i32 = FONT_SIZE_U as i32;
pub const FONT_SIZE_F: f32 = FONT_SIZE as f32;
/// The actual aspect ratio of DejaVu Mono is 1:1.933
pub const FONT_WIDTH_U: u32 = 24;
pub const FONT_WIDTH: i32 = FONT_WIDTH_U as i32;
pub const FONT_WIDTH_F: f32 = FONT_WIDTH as f32;

/// #[deprecated]
/// Idk why I deprecated this. If anything, CharGrid should be better than Glyphon Richtext
/// bc Glyphon doesn't support background colors.
/// Assumed invariant: `len(content)==width*height`
///
/// Immutable
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharGrid {
    width: usize,
    height: usize,
    content: Vec<InternalCharCell>,
}
impl From<&str> for CharGrid {
    /// Makes width and height the smallest necessary to fit everything.
    fn from(raw_content: &str) -> Self {
        Self::new_monostyled(
            raw_content,
            Color::LIGHT_YELLOW,
            Color::TRANSPARENT,
            CharCellStyle::UNSTYLED,
        )
    }
}
impl CharGrid {
    #[must_use]
    pub fn new(width: usize, height: usize, content: Vec<InternalCharCell>) -> Self {
        debug_assert_eq!(width * height, content.len());

        Self {
            width,
            height,
            content,
        }
    }

    /// TODO: make a CharGridSize struct?
    #[must_use]
    pub fn new_empty(width: usize, height: usize) -> Self {
        Self::new(
            width,
            height,
            vec![InternalCharCell::default(); width * height],
        )
    }

    /// Makes width and height the smallest necessary to fit everything.
    #[must_use]
    pub fn new_monostyled(raw_content: &str, fg: Color, bg: Color, style: CharCellStyle) -> Self {
        let Some(width) = raw_content.lines().map(str::len).max() else {
            return Self {
                width: 0,
                height: 0,
                content: Vec::new(),
            };
        };
        let height = raw_content.lines().count();

        let mut s = Self {
            width,
            height,
            content: vec![
                InternalCharCell {
                    fg,
                    bg,
                    style,
                    character: u32::from(' ')
                };
                width * height
            ],
        };
        for (row, line_str) in raw_content.split('\n').enumerate() {
            for (col, c) in line_str.chars().enumerate() {
                s.set_char(c, row, col);
            }
        }

        s
    }

    /// Returns (width, height)
    #[must_use]
    pub fn largest_fittable_size(container_size: DisplayContainerSize) -> (usize, usize) {
        // log::debug!("Largest fittable size called, container size: {container_size:?}");
        (
            // Font size is height, width is twice the height
            // I was gonna do size.width / (fontsize / 2) bc it is what is happening logically, but this is actually safer
            (container_size.width / FONT_WIDTH_U) as usize,
            (container_size.height / FONT_SIZE_U) as usize,
        )
    }

    #[must_use]
    pub const fn display_size(&self) -> DisplayContainerSize {
        DisplayContainerSize {
            width: (self.width as u32) * FONT_WIDTH_U,
            height: (self.height as u32) * FONT_SIZE_U,
        }
    }

    // pub fn get_text_as_string(&self) -> String {
    //     self.content
    //         .iter()
    //         .map(|line| {
    //             line.iter()
    //                 .map(|c| c.character.to_string())
    //                 .collect::<Vec<_>>()
    //                 .join("")
    //         })
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // }

    #[must_use]
    pub const fn element(self) -> UIElement {
        UIElement::CharGrid(self)
    }

    #[must_use]
    pub fn contained_element(self) -> UIElement {
        let size = self.display_size().into();
        self.element()
            .contain(DisplayArea::from_corner_size(DisplayCoord::ZERO, size))
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Returns char cell at index `row * self.width + col`
    #[must_use]
    pub fn get_char(&self, row: usize, col: usize) -> InternalCharCell {
        self.content[row * self.width + col]
    }

    /// Returns char cell at index `row * self.width + col`
    pub fn get_char_mut(&mut self, row: usize, col: usize) -> &mut InternalCharCell {
        &mut self.content[row * self.width + col]
    }

    /// TODO: move these into InternalCharCell
    /// at index `row * self.width + col`
    pub fn set_char(&mut self, c: char, row: usize, col: usize) {
        self.content[row * self.width + col].character = c as u32;
    }
    /// at index `row * self.width + col`
    pub fn set_fg(&mut self, fg: Color, row: usize, col: usize) {
        self.content[row * self.width + col].fg = fg;
    }
    /// at index `row * self.width + col`
    pub fn set_bg(&mut self, bg: Color, row: usize, col: usize) {
        self.content[row * self.width + col].bg = bg;
    }

    #[must_use]
    pub fn content(&self) -> &[InternalCharCell] {
        &self.content
    }
}
