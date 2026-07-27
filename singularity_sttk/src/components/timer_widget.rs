use crate::{components::Component, utils::timer::Timer};
use std::{time::Instant, vec};

/// REVIEW: keep track of (log) all the stop and start times?
pub struct TimerWidget {
    timer: Timer,

    /// running or paused
    running: bool,
    most_recent: Instant,

    button: super::EnclosedComponent<super::button::Button>,
}
impl TimerWidget {
    pub fn new(timer: Timer, running: bool) -> Self {
        TimerWidget {
            timer,
            running,
            most_recent: Instant::now(),
            button: super::EnclosedComponent::new(
                super::button::Button::new(
                    sonamu_ui::ui_element::UIElement::CharGrid("Toggle Running".to_string().into())
                        .bordered(sonamu_ui::color::Color::LIGHT_GREEN),
                ),
                sonamu_ui::display_units::DisplayArea::from_center_half_size(
                    sonamu_ui::display_units::DisplayCoord::new(0.5.into(), 0.75.into()),
                    sonamu_ui::display_units::DisplaySize::new(0.4.into(), 0.1.into()),
                ),
            ),
        }
    }

    pub fn set_running(&mut self, is_running: bool) {
        self.running = is_running
    }

    pub fn tick(&mut self) {
        if self.is_done() {
            return;
        }

        let new_recent = Instant::now();

        if self.running {
            self.timer
                .increment(new_recent.duration_since(self.most_recent));
        }

        self.most_recent = new_recent;
    }

    pub fn is_done(&self) -> bool {
        self.timer.is_done()
    }

    pub fn get_timer(&self) -> &Timer {
        &self.timer
    }
}
impl Component for TimerWidget {
    fn render(&mut self) -> sonamu_ui::ui_element::UIElement {
        self.tick();

        let fg = if self.is_done() {
            sonamu_ui::color::Color::LIGHT_GREEN
        } else if self.running {
            sonamu_ui::color::Color::WHITE
        } else {
            sonamu_ui::color::Color::ORANGE
        };

        let elapsed = sonamu_ui::ui_element::CharGrid::new_monostyled(
            format!("{:.2?}", self.timer.elapsed),
            fg,
            sonamu_ui::color::Color::BLACK,
        );

        sonamu_ui::ui_element::UIElement::Container(vec![
            sonamu_ui::ui_element::UIElement::CharGrid(elapsed)
                .fill_bg(sonamu_ui::color::Color::BLACK)
                .bordered(sonamu_uior::Color::LIGHT_GREEN),
            self.button.render(),
        ])
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use sonamu_uievent::{KeyModifiers, KeyTrait, UIEvent};
        match event {
            Event::UIEvent(ui_event) => match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::NONE) if key.to_char() == Some(' ') => {
                    // toggle running
                    self.running ^= true;
                }
                UIEvent::MousePress([mouse, window_px], container) => {
                    self.button.handle_event(Event::UIEvent(UIEvent::MousePress(
                        [mouse, window_px],
                        container,
                    )));

                    if self.button.inner_component.was_clicked() {
                        // toggle running
                        self.running ^= true;
                    }
                }
                _ => {}
            },
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }

        self.tick();
    }
}
