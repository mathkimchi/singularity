use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
};

mod char_grid;
pub use char_grid::*;
use wgpu::TextureView;

pub struct RoundRect {
    pub corner_radius: f32,
    /// Eats into content
    pub border_width: f32,
    pub main_color: [f32; 4],
    pub border_color: [f32; 4],
}

/// On their own, a UI Primitive fills up the whole space
pub enum UIPrimitiveElement {
    RoundRect(RoundRect),
    Text(Vec<(String, glyphon::AttrsOwned)>),
    CharGrid(CharGrid),
    /// Just general holder
    Texture(TextureView),
}

/// TODO: rename most everything here
/// TODO: have a UI Element vs UI Primitive (to allow for easier chunking in GPU) (Or just have a private UI Primitives and do a conversion in Winit drawing impls?)
#[derive(Debug, Clone, PartialEq)]
pub enum UIElement {
    /// Contains a list of inner elements.
    /// Overlapping is allowed.
    /// First element is bottom.
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
    pub const fn inner_size_of_bordered(container_size: DisplayContainerSize) -> DisplayContainerSize {
        DisplayContainerSize {
            // TODO: figure out all the edge cases like this
            width: container_size.width.saturating_sub(2),
            height: container_size.height.saturating_sub(2),
        }
    }

    /// First element at bottom
    /// TODO: UI element user should give in this form
    #[must_use]
    pub fn as_primitives(&self) -> Vec<(UIPrimitiveElement, DisplayArea)> {
        todo!()
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
