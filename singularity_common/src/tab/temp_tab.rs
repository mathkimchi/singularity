use std::{thread, time::Duration};

use super::{packets::Query, Event, Request, TabCreator};

pub struct TempTab {}

impl TabCreator for TempTab {
    fn create_tab(self, manager_channel: super::ManagerChannels) {
        manager_channel
            .request_tx
            .send(Request::ChangeName("Hi".to_string()))
            .expect("Failed to send request.");

        for _ in 0..10 {
            thread::sleep(Duration::from_secs(1));

            dbg!("Tab Loop tick");

            for event in manager_channel.event_rx.try_iter() {
                match event {
                    Event::KeyPress(c) => {
                        dbg!(format!("Keypress: {}", c));
                    }
                    Event::Close => break,
                }
            }
        }

        dbg!(manager_channel.query(Query::Name));

        loop {
            thread::sleep(Duration::from_secs(1));

            dbg!("Tab Loop tick");

            for event in manager_channel.event_rx.try_iter() {
                match event {
                    Event::KeyPress(c) => {
                        dbg!(format!("Keypress: {}", c));
                    }
                    Event::Close => break,
                }
            }
        }
    }
}
