//! an example of how to make a basic client with sttk,
//! as well as code to actually run it

use calloop::EventLoop;
use singularity_common::sap::raw_client_initializer::RawClientInitializer;
use singularity_sar::runner::AppletRunner;
use sonamu_ui::{ui_element::CharGrid, ui_event::KeyTrait};

#[derive(Default)]
struct BasicApp {
    content: String,
}

struct BasicInitializer;
impl RawClientInitializer for BasicInitializer {
    fn init(
        // smth smth box needs to know size
        self: Box<Self>,
        content: sonamu_sync::EncapsulatedLock<sonamu_ui::ui_element::UIElement>,
        event_queue: calloop::channel::Channel<singularity_common::sap::packets::StandardEvent>,
        // yeah, ik the naming is inconsistent bc I'm not saying "event_receiver" or "event_rx", but it's calm
        // (I am really trying to convince myself this is fine, I am the strawman)
        request_sender: calloop::channel::Sender<singularity_common::sap::packets::StandardRequest>,
    ) {
        let mut event_loop = EventLoop::try_new().unwrap();

        let mut app = BasicApp::default();

        event_loop
            .handle()
            .insert_source(
                event_queue,
                |event, &mut (), app_state: &mut BasicApp| match event {
                    calloop::channel::Event::Msg(
                        singularity_common::sap::packets::StandardEvent::UIEvent(
                            sonamu_ui::ui_event::UIEvent::KeyPress(key, _),
                        ),
                    ) => {
                        match key.to_char() {
                            // \b is not supported by rust bruh
                            Some('\x08') => {
                                app_state.content.pop();
                                content.set(CharGrid::from(app_state.content.as_str()).element());
                                request_sender.send(singularity_common::sap::packets::StandardRequest::DamageSurface).unwrap();
                            }
                            Some(key_char) => {
                                app_state.content.push(key_char);
                                content.set(CharGrid::from(app_state.content.as_str()).element());
                                request_sender.send(singularity_common::sap::packets::StandardRequest::DamageSurface).unwrap();
                            }
                            _ => (),
                        }
                    }
                    calloop::channel::Event::Msg(_) => {}
                    calloop::channel::Event::Closed => todo!(),
                },
            )
            .unwrap();

        event_loop.run(None, &mut app, |_| {}).unwrap();
    }
}

pub fn main() {
    AppletRunner::run(Box::new(BasicInitializer));
}
