//! The goal of this demo is to run a simple textbox that is standalone
//! (without the tree hierarchy stuff).

// use singularity_sar::{
//     applet::{BasicApplet, BasicRunnerHook},
//     runner::AppletRunner,
// };
// use singularity_sttk::components::text_box::TextBox;
// use sonamu_ui::{
//     color::Color,
//     display_units::DisplayContainerSize,
//     ui_element::UIElement,
//     ui_event::{KeyModifiers, KeyTrait, UIEvent},
// };

// struct TextBoxApplet {
//     hook: Box<dyn BasicRunnerHook>,
//     textbox: TextBox,
// }
// impl BasicApplet for TextBoxApplet {
//     // type InitializingData = String;

//     // fn initialize(initializing_data: Self::InitializingData, hook: Box<dyn RunnerHook>) -> Self {
//     //     Self {
//     //         hook,
//     //         textbox: TextBox::from(initializing_data),
//     //     }
//     // }

//     fn handle_ui_event(&mut self, ui_event: UIEvent) {
//         // println!("{ui_event:?}");
//         // self.hook.update_display(&UIElement::Backgrounded(
//         //     Box::new(UIElement::CharGrid(CharGrid::from(format!("{ui_event:?}")))),
//         //     Color::BLACK,
//         // ));
//         // self.hook.update_display(&UIElement::Text("a".to_string()));

//         if let UIEvent::KeyPress(
//             key,
//             KeyModifiers {
//                 ctrl: true,
//                 alt: false,
//                 shift: true,
//                 caps_lock: false,
//                 logo: false,
//             },
//         ) = &ui_event
//             && key.to_char() == Some('Q')
//         {
//             self.hook.close();
//             return;
//         }

//         self.textbox.handle_event(ui_event);

//         self.hook.damage_window();

//         // self.hook.update_display(&UIElement::Backgrounded(
//         //     Box::new(self.textbox.render()),
//         //     Color::BLACK,
//         // ));
//     }

//     fn get_window(&self, _container_size: DisplayContainerSize) -> UIElement {
//         self.textbox.render().fill_bg(Color::BLACK)
//     }
// }

fn main() {
    // AppletRunner::<TextBoxApplet>::run(|hook: Box<dyn BasicRunnerHook>| TextBoxApplet {
    //     hook,
    //     textbox: TextBox::default(),
    // })
}
