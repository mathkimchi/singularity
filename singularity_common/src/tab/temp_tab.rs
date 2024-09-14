use super::{Event, Request, TabCreator};
use ratatui::buffer::Cell;
use std::{thread, time::Duration};

pub struct TempTab {}

impl TabCreator for TempTab {
    fn create_tab(self, mut manager_channel: super::ManagerChannels) {
        manager_channel.send_request(Request::ChangeName("Hi".to_string()));

        for i in 0.. {
            thread::sleep(Duration::from_secs(1));

            // dbg!(format!("Tab Loop tick: {i}"));

            let mut new_display_buffer = vec![Cell::default(); 10];
            for (i, c) in format!("Hello: {}!", i % 100).chars().enumerate() {
                new_display_buffer[i].set_char(c);
            }
            manager_channel.update_display_buffer(new_display_buffer);

            for event in manager_channel.event_rx.try_iter() {
                match event {
                    Event::KeyPress(_c) => {
                        // dbg!(format!("Keypress: {c}"));
                    }
                    Event::Close => break,
                }
            }
        }
    }
}
