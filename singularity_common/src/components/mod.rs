//! NOTE: Widget kind of just means it is more complicated than a bare bones component

pub mod button;
pub mod text_box;
pub mod timer_widget;
// pub mod tree_viewer;

pub trait Component: Send {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement;

    fn handle_event(&mut self, event: crate::tab::packets::Event);
}

/// REVIEW: naming
/// REVIEW: is this a good idea? (feels kind of bulky to have everything like `EnclosedComponent<InnerComponent>`)
pub struct EnclosedComponent<InnerComponent: Component + ?Sized> {
    pub area: singularity_ui::display_units::DisplayArea,
    pub inner_component: InnerComponent,
}
impl<InnerComponent: Component> EnclosedComponent<InnerComponent> {
    pub fn new(
        inner_component: InnerComponent,
        area: singularity_ui::display_units::DisplayArea,
    ) -> Self {
        Self {
            area,
            inner_component,
        }
    }

    /// remap mouseclick
    pub fn forward_event(
        inner_component: &mut InnerComponent,
        area: singularity_ui::display_units::DisplayArea,
        event: crate::tab::packets::Event,
    ) {
        use crate::tab::packets::Event;
        use singularity_ui::{display_units::DisplayCoord, ui_event::UIEvent};

        if let Event::UIEvent(singularity_ui::ui_event::UIEvent::MousePress(
            [[click_x, click_y], [tot_width, tot_height]],
            container,
        )) = event
        {
            if area.map_onto(container).contains(
                DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                [tot_width as i32, tot_height as i32],
            ) {
                inner_component.handle_event(Event::UIEvent(UIEvent::MousePress(
                    [[click_x, click_y], [tot_width, tot_height]],
                    area.map_onto(container),
                )));
            }
        } else {
            inner_component.handle_event(event);
        }
    }
}
impl<InnerComponent: Component> Component for EnclosedComponent<InnerComponent> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        self.inner_component.render().contain(self.area)
    }

    /// currently, only special behavior is mouseclick
    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        EnclosedComponent::forward_event(&mut self.inner_component, self.area, event);
    }
}

impl<T: Component> Component for Option<T> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        self.as_mut().map(|inner| inner.render()).into()
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        if let Some(inner) = self.as_mut() {
            inner.handle_event(event)
        }
    }
}
impl<T: Component> Component for Box<T> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        T::render(self)
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        T::handle_event(self, event)
    }
}
impl<T: Component> Component for std::sync::Arc<std::sync::Mutex<T>> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        T::render(self.lock().as_mut().unwrap())
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        T::handle_event(self.lock().as_mut().unwrap(), event)
    }
}
