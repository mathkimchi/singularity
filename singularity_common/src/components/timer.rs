use std::time::{Duration, Instant};

/// Kind of like range
///
/// REVIEW: keep track of (log) all the stop and start times?
pub struct Timer {
    total: Duration,

    /// running or paused
    running: bool,
    elapsed: Duration,
    most_recent: Instant,
}
impl Timer {
    pub fn new(total: Duration, running: bool) -> Self {
        Timer {
            total,
            running,
            elapsed: Duration::ZERO,
            most_recent: Instant::now(),
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
            self.elapsed += new_recent.duration_since(self.most_recent);
        }

        self.most_recent = new_recent;

        // clamp
        if self.is_done() {
            self.elapsed = self.total;
        }
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.total
    }

    pub fn render(&self) -> singularity_ui::ui_element::CharGrid {
        let fg = if self.is_done() {
            singularity_ui::color::Color::LIGHT_GREEN
        } else if self.running {
            singularity_ui::color::Color::WHITE
        } else {
            singularity_ui::color::Color::ORANGE
        };

        singularity_ui::ui_element::CharGrid::new_monostyled(
            format!("{:.2?}", self.elapsed),
            fg,
            singularity_ui::color::Color::BLACK,
        )
    }

    pub fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};
        match event {
            Event::UIEvent(ui_event) => match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::NONE) if key.to_char() == Some(' ') => {
                    // toggle running
                    self.running ^= true;
                }
                _ => {}
            },
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}

/// NOTE: this is here just for the sake of debugging the timer
impl crate::tab::BasicTab<(Duration, bool)> for Timer {
    fn initialize(
        init_args: &mut (Duration, bool),
        manager_handler: &crate::tab::ManagerHandler,
    ) -> Self {
        manager_handler.send_request(crate::tab::packets::Request::ChangeName(
            "Timer".to_string(),
        ));

        Self::new(init_args.0, init_args.1)
    }

    fn render(
        &mut self,
        _manager_handler: &crate::tab::ManagerHandler,
    ) -> Option<singularity_ui::ui_element::UIElement> {
        self.tick();

        Some(
            singularity_ui::ui_element::UIElement::CharGrid(Timer::render(self))
                .fill_bg(singularity_ui::color::Color::BLACK)
                .bordered(singularity_ui::color::Color::LIGHT_GREEN),
        )
    }

    fn handle_event(
        &mut self,
        event: crate::tab::packets::Event,
        _manager_handler: &crate::tab::ManagerHandler,
    ) {
        self.tick();

        self.handle_event(event);
    }
}
