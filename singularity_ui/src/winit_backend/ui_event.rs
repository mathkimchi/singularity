use crate::display_units::{DisplayArea, DisplayContainerSize};

/// TODO: not great that I am reexporting smithay's event, given that the goal is to be backend agnostic.
/// I am doing it right now because I'd rather get something working sooner, even if I have to compromise a bit
///
/// TODO: also, figure out a way to easily match keypresses and shortcuts
///
/// TODO: figure out a standard way of "forwarding" events to child
#[derive(Debug, Clone)]
pub enum UIEvent {
    KeyPress(Key, KeyModifiers),
    WindowResized(DisplayContainerSize),
    /// ([mouse location [x, y], window size [w h]], container)
    ///
    /// REVIEW: definitely redundant, but might be helpful?
    ///
    /// NOTE: container should always be FULL for the outermost, but is helpful when trying to forward it to children:
    /// the forwarded area should be: `child_area.map_onto(parent_area)`
    MousePress([[u32; 2]; 2], DisplayArea),
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub logo: bool,
    // pub num_lock: bool,
}
#[derive(Debug, Clone)]
pub enum Key {
    ArrowKeyUp,
    ArrowKeyDown,
    ArrowKeyLeft,
    ArrowKeyRight,
    Enter,
    Backspace,
    Char(char),
}

/// TODO: Get rid of keytrait and just implement directly?
pub trait KeyTrait {
    fn to_alphabet(&self) -> Option<char>;
    fn to_digit(&self) -> Option<u8>;
    fn to_char(&self) -> Option<char>;
}
impl KeyTrait for Key {
    fn to_alphabet(&self) -> Option<char> {
        let c = self.to_char()?;
        if c.is_ascii() { Some(c) } else { None }
    }

    fn to_digit(&self) -> Option<u8> {
        let c = self.to_char()?;
        c.to_digit(10).map(|c| c as u8)
    }

    fn to_char(&self) -> Option<char> {
        // if self.raw_code == 28 {
        //     // FIXME: I added this bc I thought ENTER had no char, but it is actually `\r` already.
        //     return Some('\n');
        // }
        // self.logical_key.to_text().and_then(|s| s.chars().nth(0))
        match self {
            Key::Enter => Some('\n'),
            Key::Backspace => Some('\u{8}'),
            Key::Char(c) => Some(*c),
            _ => None,
        }
    }
}

impl KeyModifiers {
    pub const NONE: Self = KeyModifiers {
        ctrl: false,
        alt: false,
        shift: false,
        caps_lock: false,
        logo: false,
        // num_lock: false,
    };

    pub const CTRL: Self = KeyModifiers {
        ctrl: true,
        alt: false,
        shift: false,
        caps_lock: false,
        logo: false,
        // num_lock: false,
    };

    pub const ALT: Self = KeyModifiers {
        ctrl: false,
        alt: true,
        shift: false,
        caps_lock: false,
        logo: false,
        // num_lock: false,
    };

    pub const SHIFT: Self = KeyModifiers {
        ctrl: false,
        alt: false,
        shift: true,
        caps_lock: false,
        logo: false,
        // num_lock: false,
    };

    pub const LOGO: Self = KeyModifiers {
        ctrl: false,
        alt: false,
        shift: false,
        caps_lock: false,
        logo: true,
        // num_lock: false,
    };

    pub const CTRL_SHIFT: Self = KeyModifiers::both(Self::CTRL, Self::SHIFT);

    /// For example, combine(CTRL, SHIFT) is CTRL_SHIFT
    pub const fn both(self, rhs: Self) -> Self {
        Self {
            ctrl: self.ctrl | rhs.ctrl,
            alt: self.alt | rhs.alt,
            shift: self.shift | rhs.shift,
            caps_lock: self.caps_lock | rhs.caps_lock,
            logo: self.logo | rhs.logo,
            // num_lock: self.num_lock | rhs.num_lock,
        }
    }
}
impl From<winit::event::Modifiers> for KeyModifiers {
    fn from(value: winit::event::Modifiers) -> Self {
        Self {
            ctrl: value.state().control_key(),
            alt: value.state().alt_key(),
            shift: value.state().shift_key(),
            // TODO
            caps_lock: false,
            logo: value.state().super_key(),
            // num_lock,
        }
    }
}
impl std::ops::BitOr for KeyModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::both(self, rhs)
    }
}
impl std::ops::BitAnd for KeyModifiers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            ctrl: self.ctrl & rhs.ctrl,
            alt: self.alt & rhs.alt,
            shift: self.shift & rhs.shift,
            caps_lock: self.caps_lock & rhs.caps_lock,
            logo: self.logo & rhs.logo,
            // num_lock: self.num_lock & rhs.num_lock,
        }
    }
}

impl TryFrom<winit::event::KeyEvent> for Key {
    type Error = ();

    fn try_from(value: winit::event::KeyEvent) -> Result<Self, Self::Error> {
        if value.state.is_pressed() {
            match value.physical_key {
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft) => {
                    Ok(Key::ArrowKeyLeft)
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight) => {
                    Ok(Key::ArrowKeyRight)
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown) => {
                    Ok(Key::ArrowKeyDown)
                }
                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp) => {
                    Ok(Key::ArrowKeyUp)
                }

                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Enter) => {
                    Ok(Key::Enter)
                }

                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace) => {
                    Ok(Key::Backspace)
                }

                _ => value
                    .logical_key
                    .to_text()
                    .and_then(|s| s.chars().next())
                    .map(Key::Char)
                    .ok_or(()),
            }
        } else {
            Err(())
        }
    }
}
