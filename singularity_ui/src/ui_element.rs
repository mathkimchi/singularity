use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
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
    pub fn contain(self, area: DisplayArea) -> Self {
        Self::Contained(Box::new(self), area)
    }
    pub fn bordered(self, border: Color) -> Self {
        Self::Bordered(Box::new(self), border)
    }
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
    pub fn glyphon_attr(fg: Color) -> glyphon::AttrsOwned {
        glyphon::AttrsOwned::new(
            &glyphon::Attrs::new()
                .family(glyphon::Family::Monospace)
                .color(fg.into()),
        )
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

/// Made so this can instantly be turned to bytes
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
    /// NOTE: currently unused
    pub style: u32,
}
impl InternalCharCell {
    pub const BYTES: usize = 16;
}
impl From<CharCell> for InternalCharCell {
    fn from(CharCell { character, fg, bg }: CharCell) -> Self {
        Self {
            character: character as u32,
            fg,
            bg,
            // currently unused
            style: 0,
        }
    }
}
impl From<InternalCharCell> for CharCell {
    fn from(
        InternalCharCell {
            character,
            fg,
            bg,
            style: _,
        }: InternalCharCell,
    ) -> Self {
        Self {
            character: character as u8 as char,
            fg,
            bg,
        }
    }
}
impl Default for InternalCharCell {
    fn default() -> Self {
        CharCell::default().into()
    }
}

/// TODO: Replace this entirely with internal char cell?
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharCell {
    pub character: char,
    pub fg: Color,
    pub bg: Color,
    // TODO: just store glyphon::AttrsOwned?
}
impl CharCell {
    pub fn new(character: char) -> Self {
        Self {
            character,
            fg: Color::LIGHT_YELLOW,
            bg: Color::TRANSPARENT,
        }
    }
}
impl Default for CharCell {
    fn default() -> Self {
        Self::new(' ')
    }
}

/// think this is height in pixels
/// TODO: make this not-so-hardcoded...
pub const FONT_SIZE_U: u32 = 24;
pub const FONT_SIZE: i32 = FONT_SIZE_U as i32;
pub const FONT_SIZE_F: f32 = FONT_SIZE as f32;

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
impl From<String> for CharGrid {
    /// Makes width and height the smallest necessary to fit everything.
    fn from(raw_content: String) -> Self {
        let width = if let Some(width) = raw_content.lines().map(|line| line.len()).max() {
            width
        } else {
            return Self {
                width: 0,
                height: 0,
                content: Vec::new(),
            };
        };
        let height = raw_content.lines().count();

        let mut content = Vec::new();
        for line_str in raw_content.split('\n') {
            let chars = line_str.chars().collect::<Vec<_>>();
            for i in 0..width {
                content.push(CharCell::new(chars.get(i).cloned().unwrap_or(' ')).into());
            }
        }

        Self {
            width,
            height,
            content,
        }
    }
}
impl CharGrid {
    pub fn new(width: usize, height: usize, content: Vec<InternalCharCell>) -> Self {
        debug_assert_eq!(width * height, content.len());

        Self {
            width,
            height,
            content,
        }
    }

    /// TODO: make a CharGridSize struct?
    pub fn new_empty(width: usize, height: usize) -> Self {
        Self::new(
            width,
            height,
            vec![InternalCharCell::default(); width * height],
        )
    }

    pub fn new_monostyled(raw_content: String, fg: Color, bg: Color) -> Self {
        let width = if let Some(width) = raw_content.lines().map(|line| line.len()).max() {
            width
        } else {
            return Self {
                width: 0,
                height: 0,
                content: Vec::new(),
            };
        };
        let height = raw_content.lines().count();

        let mut content = Vec::new();
        for line_str in raw_content.split('\n') {
            let chars = line_str.chars().collect::<Vec<_>>();
            for i in 0..width {
                content.push(
                    CharCell {
                        fg,
                        bg,
                        character: chars.get(i).cloned().unwrap_or(' '),
                    }
                    .into(),
                );
            }
        }

        Self::new(width, height, content)
    }

    /// Returns (width, height)
    pub fn largest_fittable_size(container_size: DisplayContainerSize) -> (usize, usize) {
        (
            // Font size is height, width is twice the height
            // I was gonna do size.width / (fontsize / 2) bc it is what is happening logically, but this is actually safer
            ((container_size.width / FONT_SIZE_U) * 2) as usize,
            (container_size.height / FONT_SIZE_U) as usize,
        )
    }

    pub fn display_size(&self) -> DisplayContainerSize {
        DisplayContainerSize {
            width: (self.width as u32) * FONT_SIZE_U * 2,
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

    pub fn element(self) -> UIElement {
        UIElement::CharGrid(self)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns char cell at index `row * self.width + col`
    pub fn get_char(&self, row: usize, col: usize) -> CharCell {
        self.content[row * self.width + col].into()
    }

    // /// Returns char cell at index `row * self.width + col`
    // pub fn get_char_mut(&mut self, row: usize, col: usize) -> &mut CharCell {
    //     &mut self.content[row * self.width + col]
    // }

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
        self.content[row * self.width + col].fg = bg;
    }

    pub fn content(&self) -> &[InternalCharCell] {
        &self.content
    }
}
