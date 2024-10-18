//! NOTE: Widget kind of just means it is more complicated than a bare bones component

pub mod button;
pub mod text_box;
pub mod timer_widget;
// pub mod tree_viewer;

pub trait Component {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement;

    fn handle_event(&mut self, event: crate::tab::packets::Event);
}

/// REVIEW: naming
/// REVIEW: is this a good idea? (feels kind of bulky to have everything like `EnclosedComponent<InnerComponent>`)
pub struct EnclosedComponent<InnerComponent: Component> {
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
}
impl<InnerComponent: Component> Component for EnclosedComponent<InnerComponent> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        self.inner_component.render().contain(self.area)
    }

    /// currently, only special behavior is mouseclick
    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::{display_units::DisplayCoord, ui_event::UIEvent};

        if let Event::UIEvent(singularity_ui::ui_event::UIEvent::MousePress(
            [[click_x, click_y], [tot_width, tot_height]],
            container,
        )) = event
        {
            if self.area.map_onto(container).contains(
                DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                [tot_width as i32, tot_height as i32],
            ) {
                self.inner_component
                    .handle_event(Event::UIEvent(UIEvent::MousePress(
                        [[click_x, click_y], [tot_width, tot_height]],
                        self.area.map_onto(container),
                    )));
            }
        } else {
            self.inner_component.handle_event(event);
        }
    }
}
