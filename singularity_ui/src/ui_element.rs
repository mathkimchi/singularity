use crate::{color::Color, display_units::DisplayArea};

#[derive(Debug, Clone)]
pub enum UIElement {
    Container(Vec<UIElement>),
    /// contains inner element within a certain area
    ///
    /// elements that aren't contained should be assumed to take the entire space
    Contained(Box<UIElement>, DisplayArea),
    Bordered(Box<UIElement>),
    Text(String),

    /// should display like a terminal
    ///
    /// most important feature is that each character is the same size
    CharGrid(CharGrid),
}
impl UIElement {
    pub fn contain(self, area: DisplayArea) -> Self {
        Self::Contained(Box::new(self), area)
    }
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
