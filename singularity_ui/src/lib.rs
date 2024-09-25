use std::sync::{Arc, Mutex};

#[cfg(feature = "egui_backend")]
pub mod egui_backend;
#[cfg(feature = "iced_backend")]
pub mod iced_backend;
#[cfg(not(any(feature = "egui_backend", feature = "iced_backend")))]
compile_error!("need to choose a gui backend");

pub type DisplayArea = (usize, usize);
// pub type DisplayBuffer = Vec<u8>;
pub type UIEvent = ();

#[derive(Debug, Clone)]
pub enum UIElement {
    Div(Vec<UIElement>),
    Text(String),
}

pub struct UIDisplay {
    root_element: Arc<Mutex<UIElement>>,
}
impl UIDisplay {
    pub fn create_display(root_element: Arc<Mutex<UIElement>>) -> UIDisplay {
        UIDisplay { root_element }
    }

    pub fn try_iter_events(&self) -> Vec<UIEvent> {
        todo!()
    }

    pub fn run_display(root_element: Arc<Mutex<UIElement>>) {
        #[cfg(feature = "egui_backend")]
        {
            egui_backend::run_display(root_element);
        }
    }
}
