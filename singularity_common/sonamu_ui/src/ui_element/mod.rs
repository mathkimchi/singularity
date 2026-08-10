use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayAreaPx, DisplayContainerSize, DisplayCoord, DisplayUnits},
};

mod char_grid;
pub use char_grid::*;

use wgpu::TextureView;

#[derive(Debug, Clone, Copy)]
pub struct RoundRect {
    pub corner_radius: f32,
    /// Eats into content
    pub border_width: f32,
    /// I think this should be rgba?
    /// REVIEW: confirm or deny above
    pub main_color: [f32; 4],
    pub border_color: [f32; 4],
}

/// On their own, a UI Primitive fills up the whole space
#[derive(Debug, Clone)]
pub enum UIPrimitiveElement {
    RoundRect(RoundRect),
    Text(Vec<(String, glyphon::AttrsOwned)>),
    CharGrid(CharGrid),
    /// Just general holder
    Texture(TextureView),
}

#[derive(Debug, Clone)]
pub struct PrimitiveScene {
    pub elements: Vec<(UIPrimitiveElement, DisplayAreaPx)>,
}
impl PrimitiveScene {
    pub fn new_empty() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn add_primitive(
        &mut self,
        primitive_element: UIPrimitiveElement,
        container_area: DisplayArea,
        screen_size: DisplayContainerSize,
    ) {
        self.elements.push((
            primitive_element,
            container_area.map_onto_px_size(screen_size),
        ));
    }

    fn add_ui_element(
        &mut self,
        element: UIElement,
        container_area: DisplayArea,
        screen_size: DisplayContainerSize,
    ) {
        match element {
            UIElement::Container(children) => {
                for child_element in children {
                    // draw the inner widget
                    self.add_ui_element(child_element, container_area, screen_size);
                }
            }
            UIElement::Contained(inner_element, area) => {
                self.add_ui_element(*inner_element, area.map_onto(container_area), screen_size);
            }
            // FIXME: there are weird border lines
            UIElement::Bordered(inner_element, border_color) => {
                self.add_primitive(
                    UIPrimitiveElement::RoundRect(RoundRect {
                        corner_radius: 1.0,
                        border_width: 1.0,
                        main_color: Color::TRANSPARENT.to_rgba_f32_array(),
                        border_color: border_color.to_rgba_f32_array(),
                    }),
                    container_area,
                    screen_size,
                );

                let inner_area = DisplayArea(
                    DisplayCoord::new(1.into(), 1.into()),
                    DisplayCoord::new(
                        DisplayUnits::from_mixed(-1, 1.0),
                        DisplayUnits::from_mixed(-1, 1.0),
                    ),
                )
                .map_onto(container_area);

                // dbg!(&container_area);
                // dbg!(&container_area.size());
                // dbg!(&inner_area);

                // draw the inner widget
                self.add_ui_element(*inner_element, inner_area, screen_size);
            }
            UIElement::Backgrounded(inner_element, bg_color) => {
                // clear the inside of the border
                // ^^^ no idea what I meant by this, just keeping it
                self.add_primitive(
                    UIPrimitiveElement::RoundRect(RoundRect {
                        corner_radius: 0.0,
                        border_width: 0.0,
                        main_color: bg_color.to_rgba_f32_array(),
                        // shouldn't matter
                        border_color: [0.; 4],
                    }),
                    container_area,
                    screen_size,
                );

                // draw the inner widget
                self.add_ui_element(*inner_element, container_area, screen_size);
            }
            UIElement::Text(text) => {
                self.add_primitive(UIPrimitiveElement::Text(text), container_area, screen_size);
            }
            UIElement::CharGrid(char_grid) => {
                self.add_primitive(
                    UIPrimitiveElement::CharGrid(char_grid),
                    container_area,
                    screen_size,
                );
            }
            UIElement::Image(_image_buffer) => {
                todo!()
            }
            UIElement::Texture(texture_view) => {
                self.add_primitive(
                    UIPrimitiveElement::Texture(texture_view),
                    container_area,
                    screen_size,
                );
            }
            UIElement::Subsurface(_) => {
                // TODO: take a function that maps subsurface ids to content or something idk
                todo!()
            }
            UIElement::Nothing => {}
        }
    }

    pub fn from_ui_element(content: UIElement, screen_size: DisplayContainerSize) -> Self {
        let mut scene = PrimitiveScene::new_empty();

        scene.add_ui_element(content, DisplayArea::FULL, screen_size);

        scene
    }
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

    #[deprecated]
    Image(image::RgbaImage),

    /// TODO: replace image with this
    Texture(TextureView),

    /// Reference to an embedded UI element
    Subsurface(u32),

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
    pub const fn inner_size_of_bordered(
        container_size: DisplayContainerSize,
    ) -> DisplayContainerSize {
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
