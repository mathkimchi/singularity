use singularity_ui::{display_units::DisplayArea, ui_element::UIElement};

use super::Component;

pub struct Button {
    inner_element: UIElement,

    area: DisplayArea,

    /// because unpress isn't an event, clicked only represents whether it has been clicked since the most recent clicked query
    clicked: bool,
}
impl Button {
    pub fn new(inner_element: UIElement, area: DisplayArea) -> Self {
        Self {
            inner_element,
            area,
            clicked: false,
        }
    }

    pub fn was_clicked(&mut self) -> bool {
        let clicked = self.clicked;

        self.clicked = false;

        clicked
    }
}
impl Component for Button {
    fn render(&mut self) -> UIElement {
        self.inner_element.clone().contain(self.area)
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::ui_event::UIEvent;
        match event {
            Event::UIEvent(ui_event) => {
                if let UIEvent::MousePress([mouse, window_px], container_area) = ui_event {
                    if self.area.map_onto(container_area).contains(
                        singularity_ui::display_units::DisplayCoord::new(
                            (mouse[0] as i32).into(),
                            (mouse[1] as i32).into(),
                        ),
                        [window_px[0] as i32, window_px[1] as i32],
                    ) {
                        dbg!("clicked");
                        self.clicked = true;
                    }
                }
            }
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
