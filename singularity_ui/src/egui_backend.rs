use crate::{display_units::DisplaySize, ui_event::UIEvent, CharGrid, UIElement};
use egui::{widget_text, Color32, Widget};
use std::sync::{Arc, Mutex};

pub const FRAME_RATE: f32 = 5.;
pub const FRAME_DELTA_SECONDS: f32 = 1. / FRAME_RATE;

impl egui::Widget for &UIElement {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            UIElement::Container(children) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                // idk the point of response
                let mut response = ui.interact(
                    ui.available_rect_before_wrap(),
                    ui.id(),
                    egui::Sense::hover(),
                );

                for (child, rect) in children {
                    // NOTE: I am jankily generating a new idea based on the rectangle to avoid clash
                    // TODO: figure out an actual way to do ids, or just don't do ids

                    response = egui::containers::Window::new(
                        ui.next_auto_id()
                            .with(rect.0.x.to_bits())
                            .with(rect.0.y.to_bits())
                            .with(rect.1.x.to_bits())
                            .with(rect.1.y.to_bits())
                            .value()
                            .to_string(),
                    )
                    .collapsible(false)
                    .title_bar(false)
                    .scroll(false)
                    .fixed_rect((*rect).into())
                    .show(ui.ctx(), |ui| {
                        egui::ScrollArea::new(false).show(ui, |ui| child.ui(ui))
                    })
                    .unwrap()
                    .response;

                    // response = egui::containers::Window::new(
                    //     ui.next_auto_id()
                    //         .with(rect.0.x.to_bits())
                    //         .with(rect.0.y.to_bits())
                    //         .with(rect.1.x.to_bits())
                    //         .with(rect.1.y.to_bits())
                    //         .value()
                    //         .to_string(),
                    // )
                    // .collapsible(false)
                    // .title_bar(false)
                    // .scroll(false)
                    // .fixed_rect((*rect).into())
                    // .show(ui.ctx(), |ui| child.ui(ui))
                    // .unwrap()
                    // .response;
                    // response = response.union(ui.put((*rect).into(), child));
                }

                response
            }
            UIElement::Horizontal(children) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.horizontal(|ui| {
                    for child in children {
                        ui.add(child);
                    }
                })
                .response
            }
            UIElement::Bordered(inner) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                egui::Frame::none()
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .fill(ui.visuals().panel_fill)
                    .show(ui, move |ui| inner.ui(ui))
                    .response
            }
            UIElement::Text(s) => {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.label(s)
            }
            UIElement::CharGrid(CharGrid { content }) => {
                // FIXME: heights not constant for some reason
                const CHAR_SIZE: DisplaySize = DisplaySize::new(8.0, 16.0);
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.spacing_mut().window_margin = egui::Margin::ZERO;
                ui.spacing_mut().indent = 0.0;

                egui::Grid::new(content)
                    .min_col_width(CHAR_SIZE.width)
                    .max_col_width(CHAR_SIZE.width)
                    .spacing(egui::Vec2::ZERO)
                    .show(ui, |ui| {
                        for line in content {
                            for c in line.iter() {
                                // dbg!(ui.spacing());
                                ui.add_sized(
                                    egui::Vec2::new(CHAR_SIZE.width, CHAR_SIZE.height),
                                    egui::Label::new(
                                        widget_text::RichText::monospace(
                                            c.character.to_string().into(),
                                        )
                                        .size(CHAR_SIZE.height)
                                        .color(c.fg)
                                        .background_color(c.bg)
                                        .extra_letter_spacing(0.0),
                                    ),
                                );
                            }
                            ui.end_row();
                        }
                    })
                    .response
            }
        }
    }
}

pub struct UIDisplay {
    root_element: Arc<Mutex<UIElement>>,

    event_queue: Arc<Mutex<Vec<UIEvent>>>,
}
impl UIDisplay {
    pub fn create_display(
        root_element: Arc<Mutex<UIElement>>,
        event_queue: Arc<Mutex<Vec<UIEvent>>>,
    ) -> UIDisplay {
        UIDisplay {
            root_element,
            event_queue,
        }
    }

    // pub fn collect_events(&mut self) -> Vec<UIEvent> {
    //     std::mem::take(&mut self.event_queue)
    // }

    /// Returns when display is closed.
    pub fn run_display(root_element: Arc<Mutex<UIElement>>, event_queue: Arc<Mutex<Vec<UIEvent>>>) {
        eframe::run_native(
            "Singularity",
            eframe::NativeOptions {
                event_loop_builder: Some(Box::new(|event_loop_builder| {
                    use winit::platform::wayland::EventLoopBuilderExtWayland;
                    // NOTE: eframe 28 uses winit 0.29, and this doesn't work with winit 0.30
                    event_loop_builder.with_any_thread(true);
                })),
                ..Default::default()
            },
            Box::new(move |_cc| {
                Ok(Box::new(UIDisplay::create_display(
                    root_element,
                    event_queue,
                )))
            }),
        )
        .unwrap();
    }
}

impl eframe::App for UIDisplay {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.root_element.lock().unwrap().ui(ui);
        });

        for event in ctx.input(|i| i.events.clone()) {
            self.event_queue.lock().unwrap().push(event);
        }

        ctx.request_repaint_after_secs(FRAME_DELTA_SECONDS);
    }
}

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

pub type Color = Color32;
