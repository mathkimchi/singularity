#[cfg(feature = "egui_backend")]
mod egui_backend;
#[cfg(feature = "iced_backend")]
mod iced_backend;
#[cfg(not(any(feature = "egui_backend", feature = "iced_backend")))]
compile_error!("need to choose a gui backend");

use egui::Color32;
#[cfg(feature = "egui_backend")]
pub use egui_backend::UIDisplay;

pub type DisplayArea = (usize, usize);
// pub type DisplayBuffer = Vec<u8>;
// pub enum UIEvent {
//     KeyPress {
//         key_char: char,
//         alt: bool,
//         ctrl: bool,
//         shift: bool,
//     },
// }

pub mod ui_event {
    /// FIXME: not great that I am reexporting egui's event, given that the goal is to be backend agnostic.
    /// I am doing it right now because I'd rather get something working sooner, even if I have to compromise a bit
    pub type UIEvent = egui::Event;
    pub type KeyModifiers = egui::Modifiers;
    pub type Key = egui::Key;

    pub trait KeyTrait {
        fn to_alphabet(&self) -> Option<char>;
    }
    impl KeyTrait for Key {
        fn to_alphabet(&self) -> Option<char> {
            match self {
                egui::Key::A => Some('a'),
                egui::Key::B => Some('b'),
                egui::Key::C => Some('c'),
                egui::Key::D => Some('d'),
                egui::Key::E => Some('e'),
                egui::Key::F => Some('f'),
                egui::Key::G => Some('g'),
                egui::Key::H => Some('h'),
                egui::Key::I => Some('i'),
                egui::Key::J => Some('j'),
                egui::Key::K => Some('k'),
                egui::Key::L => Some('l'),
                egui::Key::M => Some('m'),
                egui::Key::N => Some('n'),
                egui::Key::O => Some('o'),
                egui::Key::P => Some('p'),
                egui::Key::Q => Some('q'),
                egui::Key::R => Some('r'),
                egui::Key::S => Some('s'),
                egui::Key::T => Some('t'),
                egui::Key::U => Some('u'),
                egui::Key::V => Some('v'),
                egui::Key::W => Some('w'),
                egui::Key::X => Some('x'),
                egui::Key::Y => Some('y'),
                egui::Key::Z => Some('z'),
                _ => None,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum UIElement {
    Container(Vec<(UIElement, DisplayArea)>),
    Bordered(Box<UIElement>),
    Text(String),

    /// should display like a terminal
    ///
    /// most important feature is that each character is the same size
    CharGrid {
        content: Vec<Vec<(char, Color32, Color32)>>,
    },
}
impl UIElement {
    pub fn char_grid(raw_content: impl ToString) -> Self {
        let mut content = Vec::new();
        for line_str in raw_content.to_string().split('\n') {
            let mut line = Vec::new();
            for c in line_str.chars() {
                line.push((c, Color32::LIGHT_BLUE, Color32::DARK_RED));
            }
            content.push(line);
        }

        UIElement::CharGrid { content }
    }
}
