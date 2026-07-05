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
    #[deprecated]
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
                .color((&fg).into()),
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

#[deprecated]
#[derive(Debug, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
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
    pub fn new_monostyled(raw_content: String, fg: Color, bg: Color) -> Self {
        let mut content = Vec::new();
        for line_str in raw_content.split('\n') {
            let mut line = Vec::new();
            for character in line_str.chars() {
                line.push(CharCell { character, fg, bg });
            }
            content.push(line);
        }

        Self { content }
    }

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

    pub fn element(self) -> UIElement {
        UIElement::CharGrid(self)
    }
}
