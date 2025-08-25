use singularity_sap::{
    dylib_applet::applet_context::AppletContext,
    standard_packets::{
        StandardEvent,
        display_packets::{CloseWarningEvent, DisplayEvent, ResizeEvent},
    },
};
use singularity_sttk::{ratk::ReactiveApplet, register_applet};
use singularity_ui::ui_event::UIEvent;

pub struct MathGameApplet {
    terms: [u8; 2],
}
impl MathGameApplet {
    fn handle_ui_event(ui_event: UIEvent) {
        match ui_event {
            UIEvent::KeyPress(key_event, key_modifiers) => {}
            UIEvent::WindowResized(_) => {}
            UIEvent::MousePress(_, _display_area) => {}
        }
    }
}
impl ReactiveApplet<StandardEvent> for MathGameApplet {
    fn handle_event(&mut self, event: StandardEvent, applet_context: &AppletContext) {
        match event {
            StandardEvent::DisplayEvent(DisplayEvent::Close(CloseWarningEvent)) => {
                println!("Math game: Goodbye!")
            }
            StandardEvent::DisplayEvent(DisplayEvent::Resize(ResizeEvent(display_area))) => {
                println!("Dylib math game: resized to {display_area:?}")
            }
            StandardEvent::DisplayEvent(DisplayEvent::UIEvent(ui_event)) => {
                MathGameApplet::handle_ui_event(ui_event);
            }
            StandardEvent::DisplayEvent(DisplayEvent::Focused(_)) => {}
            StandardEvent::DisplayEvent(DisplayEvent::Unfocused(_)) => {}
        }
    }

    fn handle_global_event(event: StandardEvent, applet_context: &AppletContext) {
        match event {
            StandardEvent::DisplayEvent(DisplayEvent::Close(CloseWarningEvent)) => {
                println!("Math game: Goodbye!")
            }
            StandardEvent::DisplayEvent(DisplayEvent::Resize(ResizeEvent(display_area))) => {
                println!("Dylib math game: resized to {display_area:?}")
            }
            StandardEvent::DisplayEvent(DisplayEvent::UIEvent(ui_event)) => {
                MathGameApplet::handle_ui_event(ui_event);
            }
            StandardEvent::DisplayEvent(DisplayEvent::Focused(_)) => {}
            StandardEvent::DisplayEvent(DisplayEvent::Unfocused(_)) => {}
        }
    }
}
register_applet!(MathGameApplet);
