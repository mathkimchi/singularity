//! The goal of this demo is to run a simple textbox that is standalone
//! (without the tree hierarchy stuff).

use singularity_sar::{
    applet::{BasicApplet, RunnerHook},
    runner::AppletRunner,
};
use singularity_sttk::components::text_box::TextBox;
use singularity_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
};

struct TextBoxApplet {
    hook: Box<dyn RunnerHook>,
    textbox: TextBox,
}
impl BasicApplet for TextBoxApplet {
    type InitializingData = String;

    fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self {
        Self {
            hook,
            textbox: TextBox::from(initializing_data),
        }
    }

    fn handle_ui_event(&mut self, ui_event: singularity_ui::ui_event::UIEvent) {
        // println!("{ui_event:?}");
        // self.hook.update_display(&UIElement::Backgrounded(
        //     Box::new(UIElement::CharGrid(CharGrid::from(format!("{ui_event:?}")))),
        //     Color::BLACK,
        // ));
        // self.hook.update_display(&UIElement::Text("a".to_string()));

        self.textbox
            .handle_event(singularity_common::sap::packets::DisplayEvent::UIEvent(
                ui_event,
            ));

        self.hook.update_display(&UIElement::Backgrounded(
            Box::new(self.textbox.render()),
            Color::BLACK,
        ));
    }
}

fn main() {
    AppletRunner::<TextBoxApplet>::run(String::new())
}
