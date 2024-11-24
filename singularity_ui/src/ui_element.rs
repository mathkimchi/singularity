use crate::{color::Color, display_units::DisplayArea};

/// TODO: rename most everything here
#[derive(Debug, Clone)]
pub enum UIElement {
    Container(Vec<UIElement>),

    /// contains inner element within a certain area
    ///
    /// elements that aren't contained should be assumed to take the entire space
    Contained(Box<UIElement>, DisplayArea),
    Bordered(Box<UIElement>, Color),
    /// TODO: better name
    Backgrounded(Box<UIElement>, Color),

    /// FIXME: literally just doesn't work
    Text(String),

    /// should display like a terminal
    ///
    /// most important feature is that each character is the same size
    CharGrid(CharGrid),

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
}
impl From<Option<UIElement>> for UIElement {
    fn from(value: Option<UIElement>) -> Self {
        value.unwrap_or(UIElement::Nothing)
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
}
