use std::{thread, time::Duration};

use super::{Event, Request, TabCreator};

pub struct TempTab {}

impl TabCreator for TempTab {
    fn create_tab(self, manager_channel: super::ManagerChannels) {
        manager_channel
            .request_tx
            .send(Request::ChangeName("Hi".to_string()))
            .expect("Failed to send request.");

        // for event in manager_channel.event_rx.iter() {
        //     match event {
        //         Event::KeyPress(c) => {
        //             dbg!(c);
        //         }
        //         Event::Close => break,
        //     }
        // }

        loop {
            thread::sleep(Duration::from_secs(1));

            dbg!("Hello from loop");

            match manager_channel.event_rx.try_recv() {
                Ok(Event::KeyPress(c)) => {
                    dbg!(format!("Keypress: {}", c));
                }
                Ok(Event::Close) => break,
                Err(_) => {}
            }
        }
    }
}
