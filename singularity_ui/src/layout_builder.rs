use crate::{
    color::Color,
    display_units::{DisplayArea, DisplayContainerSize},
    ui_element::UIElement,
};

/// This holds off on returning UI elements until the size is given.
/// look at 2026-07-12 DEVLOG for its conception.
///
/// This could be its own crate, but I think there are too many crates.
/// REVIEW: maybe make this a feature?
pub struct LayoutBuilder<'a> {
    inner: Box<dyn FnOnce(DisplayContainerSize) -> UIElement + 'a>,
}
impl<'a> LayoutBuilder<'a> {
    #[must_use]
    pub fn from_fn(f: impl FnOnce(DisplayContainerSize) -> UIElement + 'a) -> Self {
        Self { inner: Box::new(f) }
    }

    #[must_use]
    pub fn get_ui_element(self, container_size: DisplayContainerSize) -> UIElement {
        (self.inner)(container_size)
    }

    /// NOTE: This is different from ui_element.bordered() because that evaluates the ui_elment then imposes a constraint
    /// but here, the inner element is evaluated *after* the bounds are given.
    #[must_use]
    pub fn bordered(self, border_color: Color) -> Self {
        Self::from_fn(move |container_size| {
            let inner_container_size = UIElement::inner_size_of_bordered(container_size);
            UIElement::Bordered(
                Box::new(self.get_ui_element(inner_container_size)),
                border_color,
            )
        })
    }

    #[must_use]
    pub fn contained(self, container_area: DisplayArea) -> Self {
        Self::from_fn(move |container_size| {
            let inner_container_size = container_size.find_subsize(container_area.size());
            UIElement::Contained(
                Box::new(self.get_ui_element(inner_container_size)),
                container_area,
            )
        })
    }
}
