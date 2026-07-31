//! an example of how to make a basic client with sttk,
//! as well as code to actually run it

use calloop::{EventLoop, channel::Sender};
use singularity_common::sap::{
    packets::StandardRequest, raw_client_initializer::RawClientInitializer,
};
use singularity_sar::runner::AppletRunner;
use sonamu_sync::EncapsulatedLock;
use sonamu_ui::{
    ui_element::{CharGrid, UIElement},
    ui_event::{self, Key, KeyModifiers, KeyTrait},
};

struct BasicApp {
    content_str: String,
    content: EncapsulatedLock<UIElement>,
    request_sender: Sender<StandardRequest>,
}
impl BasicApp {
    fn handle_key_press(&mut self, key: Key, mods: KeyModifiers) {
        match (key, mods) {
            (Key::Char('Q'), KeyModifiers::CTRL_SHIFT) => {
                self.request_sender.send(StandardRequest::Quit).unwrap();
            }
            (Key::Backspace, _) => {
                self.content_str.pop();
                self.content
                    .set(CharGrid::from(self.content_str.as_str()).element());
                self.request_sender
                    .send(StandardRequest::DamageSurface)
                    .unwrap();
            }
            _ => {
                match key.to_char() {
                    // \b is not supported by rust bruh
                    Some('\x08') => {}
                    Some(key_char) => {
                        self.content_str.push(key_char);
                        self.content
                            .set(CharGrid::from(self.content_str.as_str()).element());
                        self.request_sender
                            .send(StandardRequest::DamageSurface)
                            .unwrap();
                    }
                    _ => (),
                }
            }
        }
    }
}

struct BasicInitializer;
impl RawClientInitializer for BasicInitializer {
    fn init(
        // smth smth box needs to know size
        self: Box<Self>,
        content: EncapsulatedLock<UIElement>,
        event_queue: calloop::channel::Channel<singularity_common::sap::packets::StandardEvent>,
        // yeah, ik the naming is inconsistent bc I'm not saying "event_receiver" or "event_rx", but it's calm
        // (I am really trying to convince myself this is fine, I am the strawman)
        request_sender: Sender<StandardRequest>,
    ) {
        let mut event_loop = EventLoop::try_new().unwrap();

        let mut app = BasicApp {
            content_str: "placeholder".to_string(),
            content,
            request_sender,
        };

        event_loop
            .handle()
            .insert_source(
                event_queue,
                |event, &mut (), app_state: &mut BasicApp| match event {
                    calloop::channel::Event::Msg(
                        singularity_common::sap::packets::StandardEvent::UIEvent(
                            ui_event::UIEvent::KeyPress(key, mods),
                        ),
                    ) => {
                        app_state.handle_key_press(key, mods);
                    }
                    calloop::channel::Event::Msg(_) => {}
                    calloop::channel::Event::Closed => {
                        println!("Goodbye!");
                    }
                },
            )
            .unwrap();

        event_loop.run(None, &mut app, |_| {}).unwrap();
    }
}

pub fn main() {
    AppletRunner::run(Box::new(BasicInitializer));
}
