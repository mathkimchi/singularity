use super::Component;
use singularity_ui::ui_element::UIElement;

pub struct Button {
    inner_element: UIElement,

    /// because unpress isn't an event, clicked only represents whether it has been clicked since the most recent clicked query
    clicked: bool,
}
impl Button {
    pub fn new(inner_element: UIElement) -> Self {
        Self {
            inner_element,
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
        self.inner_element.clone()
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::ui_event::UIEvent;
        match event {
            Event::UIEvent(ui_event) => {
                if let UIEvent::MousePress(..) = ui_event {
                    dbg!("clicked");
                    self.clicked = true;
                }
            }
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
