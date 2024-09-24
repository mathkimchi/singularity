use std::sync::{Arc, Mutex};

pub type DisplayArea = (usize, usize);
// pub type DisplayBuffer = Vec<u8>;
pub type UIEvent = ();

#[derive(Debug, Clone)]
pub enum UIElement {
    Div(Vec<UIElement>),
    Letter(char),
}

pub struct Display {
    root_element: Arc<Mutex<UIElement>>,
}
impl Display {
    pub fn create_display(root_element: Arc<Mutex<UIElement>>) -> Display {
        Display { root_element }
    }

    // pub fn render_display_buffer(&self, display_area: DisplayArea, display_buffer: DisplayBuffer) {}

    pub fn try_iter_events(&self) -> Vec<UIEvent> {
        todo!()
    }
}

#[cfg(feature = "iced_backend")]
impl iced::Sandbox for Display {
    type Message = ();

    fn new() -> Self {
        Display {
            root_element: Arc::new(Mutex::new(UIElement::Div(Vec::new()))),
        }
    }

    fn title(&self) -> String {
        String::from("idk")
    }

    fn update(&mut self, message: Self::Message) {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        iced::widget::column![].into()
    }
}
#[cfg(not(feature = "iced_backend"))]
compile_error!("");
