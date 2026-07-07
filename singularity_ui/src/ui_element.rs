use crate::{color::Color, display_units::DisplayArea};

/// TODO: rename most everything here
#[derive(Debug, Clone, PartialEq)]
pub enum UIElement {
    /// Contains a list of inner elements.
    /// Overlapping is allowed.
    Container(Vec<UIElement>),

    /// contains inner element within a certain area
    ///
    /// elements that aren't contained should be assumed to take the entire space
    Contained(Box<UIElement>, DisplayArea),
    /// TODO: Fix the right and bottom borders.
    Bordered(Box<UIElement>, Color),
    /// TODO: better name
    Backgrounded(Box<UIElement>, Color),

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
    pub fn combine_displays(subdisplays: impl ExactSizeIterator<Item = UIElement>) -> UIElement {
        // proportional units so widths out of 1
        let widths = 1. / subdisplays.len() as f32;
        UIElement::Container(
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
impl From<Option<UIElement>> for UIElement {
    fn from(value: Option<UIElement>) -> Self {
        value.unwrap_or(UIElement::Nothing)
    }
}
impl From<String> for UIElement {
    fn from(raw_content: String) -> Self {
        UIElement::Text(vec![(raw_content, Self::glyphon_attr(Color::WHITE))])
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharCell {
    pub character: char,
    pub fg: Color,
    pub bg: Color,
    // TODO: just store glyphon::AttrsOwned?
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

/// #[deprecated]
/// Idk why I deprecated this. If anything, CharGrid should be better than Glyphon Richtext
/// bc Glyphon doesn't support background colors.
/// Assumed invariant: `len(content)==width*height`
///
/// Immutable
#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharGrid {
    width: usize,
    height: usize,
    content: Vec<CharCell>,
}
impl From<String> for CharGrid {
    /// Makes width and height the smallest necessary to fit everything.
    fn from(raw_content: String) -> Self {
        let width = if let Some(width) = raw_content.lines().map(|line| line.len()).max() {
            width
        } else {
            return CharGrid {
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
                content.push(CharCell::new(chars.get(i).cloned().unwrap_or(' ')));
            }
        }

        CharGrid {
            width,
            height,
            content,
        }
    }
}
impl CharGrid {
    pub fn new_monostyled(raw_content: String, fg: Color, bg: Color) -> Self {
        let width = if let Some(width) = raw_content.lines().map(|line| line.len()).max() {
            width
        } else {
            return CharGrid {
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
                content.push(CharCell {
                    fg,
                    bg,
                    character: chars.get(i).cloned().unwrap_or(' '),
                });
            }
        }

        CharGrid {
            width,
            height,
            content,
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
        self.content[row * self.width + col]
    }

    pub fn content(&self) -> &[CharCell] {
        &self.content
    }
}
