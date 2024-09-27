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
        fn to_digit(&self) -> Option<u8>;
        fn to_char(&self) -> Option<char>;
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

        fn to_digit(&self) -> Option<u8> {
            match self {
                egui::Key::Num0 => Some(0),
                egui::Key::Num1 => Some(1),
                egui::Key::Num2 => Some(2),
                egui::Key::Num3 => Some(3),
                egui::Key::Num4 => Some(4),
                egui::Key::Num5 => Some(5),
                egui::Key::Num6 => Some(6),
                egui::Key::Num7 => Some(7),
                egui::Key::Num8 => Some(8),
                egui::Key::Num9 => Some(9),
                _ => None,
            }
        }

        fn to_char(&self) -> Option<char> {
            if let Some(c) = self.to_alphabet() {
                Some(c)
            } else if let Some(d) = self.to_digit() {
                Some(d.to_string().pop().unwrap())
            } else {
                // special characters
                match self {
                    egui::Key::Enter => Some('\n'),
                    egui::Key::Space => Some(' '),
                    egui::Key::Colon => Some(':'),
                    egui::Key::Comma => Some(','),
                    egui::Key::Backslash => Some('\\'),
                    egui::Key::Slash => Some('/'),
                    egui::Key::Pipe => Some('|'),
                    egui::Key::Questionmark => Some('?'),
                    egui::Key::OpenBracket => Some('['),
                    egui::Key::CloseBracket => Some(']'),
                    egui::Key::Backtick => Some('`'),
                    egui::Key::Minus => Some('-'),
                    egui::Key::Period => Some('.'),
                    egui::Key::Plus => Some('+'),
                    egui::Key::Equals => Some('='),
                    egui::Key::Semicolon => Some(';'),
                    egui::Key::Quote => Some('\''),
                    _ => None,
                }
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
    CharGrid(CharGrid),
}

pub use egui::Color32;
#[derive(Debug, Clone, Copy, Hash)]
pub struct CharCell {
    pub character: char,
    pub fg: Color32,
    pub bg: Color32,
}
impl CharCell {
    pub fn new(character: char) -> Self {
        CharCell {
            character,
            fg: Color32::BLUE,
            bg: Color32::TRANSPARENT,
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct CharGrid {
    pub content: Vec<Vec<CharCell>>,
}
impl From<String> for CharGrid {
    fn from(raw_content: String) -> Self {
        let mut content = Vec::new();
        for line_str in raw_content.split('\n') {
            let mut line = Vec::new();
            for c in line_str.chars() {
                line.push(CharCell::new(c));
            }
            content.push(line);
        }

        CharGrid { content }
    }
}
impl CharGrid {
    pub fn get_text_as_string(&self) -> String {
        self.content
            .iter()
            .map(|line| {
                line.iter()
                    .map(|c| c.character.to_string())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
