use super::Component;
use singularity_ui::ui_element::UIElement;

/// TODO: make button's `was_clicked` feature a macro so it is more flexible
pub struct Button {
    pub inner_element: UIElement,

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
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}

pub struct ToggleButton {
    pub on_inner: UIElement,
    pub off_inner: UIElement,
    pub toggle: bool,
}
impl ToggleButton {
    pub fn new(on_inner: UIElement, off_inner: UIElement, toggle: bool) -> Self {
        Self {
            on_inner,
            off_inner,
            toggle,
        }
    }
}
impl Component for ToggleButton {
    fn render(&mut self) -> UIElement {
        match self.toggle {
            true => self.on_inner.clone(),
            false => self.off_inner.clone(),
        }
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::ui_event::UIEvent;
        match event {
            Event::UIEvent(ui_event) => {
                if let UIEvent::MousePress(..) = ui_event {
                    dbg!("toggled");
                    self.toggle = !self.toggle;
                }
            }
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
