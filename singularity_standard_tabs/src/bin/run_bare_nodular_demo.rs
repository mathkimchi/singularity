use singularity_sar::applet::BasicRunnerHook;
use singularity_sar::{applet::BasicApplet, runner::AppletRunner};
use singularity_sttk::nodular_applet::recursive_node_applet::DividedApplet;
use singularity_sttk::{
    components::text_box::TextBox,
    nodular_applet::{NodularApplet, NodularRunnerHook},
};
use singularity_ui::{
    color::Color,
    ui_element::UIElement,
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};

pub struct TextBoxApplet {
    hook: Box<dyn NodularRunnerHook>,
    textbox: TextBox,
}
impl TextBoxApplet {
    pub fn get_initiator(text: String) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        |hook: Box<dyn NodularRunnerHook>| TextBoxApplet {
            hook,
            textbox: TextBox::from(text),
        }
    }
    pub fn get_boxed_initiator(
        text: String,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet> {
        |hook: Box<dyn NodularRunnerHook>| {
            Box::new(TextBoxApplet {
                hook,
                textbox: TextBox::from(text),
            })
        }
    }
}
impl BasicApplet for TextBoxApplet {
    // type InitializingData = String;

    // fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self {
    //     Self {
    //         hook,
    //         textbox: TextBox::from(initializing_data),
    //     }
    // }

    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        // println!("{ui_event:?}");
        // self.hook.update_display(&UIElement::Backgrounded(
        //     Box::new(UIElement::CharGrid(CharGrid::from(format!("{ui_event:?}")))),
        //     Color::BLACK,
        // ));
        // self.hook.update_display(&UIElement::Text("a".to_string()));

        if let UIEvent::KeyPress(
            key,
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: true,
                caps_lock: false,
                logo: false,
            },
        ) = &ui_event
            && key.to_char() == Some('Q')
        {
            self.hook.close();
            return;
        }

        if let UIEvent::KeyPress(
            key,
            KeyModifiers {
                ctrl: true,
                alt: false,
                shift: true,
                caps_lock: false,
                logo: false,
            },
        ) = &ui_event
            && key.to_char() == Some('+')
        {
            self.hook
                .add_child(Box::new(TextBoxApplet::get_boxed_initiator(String::new())));
            return;
        }

        self.textbox.handle_event(ui_event);

        self.hook.update_display(&UIElement::Backgrounded(
            Box::new(self.textbox.render()),
            Color::BLACK,
        ));
    }
}
impl NodularApplet for TextBoxApplet {
    fn handle_nodular_event(
        &mut self,
        nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        match nodular_event {
            singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
            singularity_sttk::nodular_applet::NodularEvent::Focused(_) => {
                println!("Yay focus!");
                println!("The text is: {}", &self.textbox.get_text_as_string());
            }
        }
    }
}

fn main() {
    AppletRunner::<DividedApplet>::run(DividedApplet::get_initiator(
        TextBoxApplet::get_boxed_initiator(String::new()),
    ))
}
