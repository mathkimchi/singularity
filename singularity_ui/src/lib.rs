#[cfg(feature = "egui_backend")]
mod egui_backend;
#[cfg(feature = "iced_backend")]
mod iced_backend;
#[cfg(not(any(feature = "egui_backend", feature = "iced_backend")))]
compile_error!("need to choose a gui backend");

#[cfg(feature = "egui_backend")]
pub use egui_backend::UIDisplay;

pub type DisplayArea = (usize, usize);
// pub type DisplayBuffer = Vec<u8>;
pub enum UIEvent {
    KeyPress {
        key_char: char,
        alt: bool,
        ctrl: bool,
        shift: bool,
    },
}

#[derive(Debug, Clone)]
pub enum UIElement {
    Container(Vec<(UIElement, DisplayArea)>),
    Bordered(Box<UIElement>),
    Text(String),
}
