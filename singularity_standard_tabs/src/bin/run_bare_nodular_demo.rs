use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_common::utils::tree::world_tree::world_tree_traversal::WorldTreeTraversalOperation;
use singularity_sar::applet::BasicRunnerHook;
use singularity_sar::{applet::BasicApplet, runner::AppletRunner};
use singularity_sttk::nodular_applet::recursive_node_applet::RecursiveNodeApplet;
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
    focused: bool,
}
impl TextBoxApplet {
    pub fn get_initiator(text: String) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        |hook: Box<dyn NodularRunnerHook>| TextBoxApplet {
            hook,
            textbox: TextBox::from(text),
            focused: true,
        }
    }
    pub fn get_boxed_initiator(
        text: String,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet> {
        |hook: Box<dyn NodularRunnerHook>| {
            Box::new(TextBoxApplet {
                hook,
                textbox: TextBox::from(text),
                focused: true,
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
            // self.hook
            //     .add_child(Box::new(TextBoxApplet::get_boxed_initiator(String::new())));
            self.hook
                .add_child(Box::new(RecursiveNodeApplet::get_boxed_initializer(
                    TextBoxApplet::get_boxed_initiator(String::new()),
                )));
            return;
        }

        if let UIEvent::KeyPress(
            key,
            KeyModifiers {
                ctrl: false,
                alt: true,
                shift: false,
                caps_lock: false,
                logo: false,
            },
        ) = &ui_event
            && key.to_char() == Some('q')
        {
            self.hook
                .change_focus(WorldTreeTraversalOperation::PrevLayer);
            return;
        }

        self.textbox.handle_event(ui_event);

        self.hook.damage_window();
        self.hook.damage_treeview();
    }

    fn get_window(&self) -> UIElement {
        let cursor_color = if self.focused {
            Color::LIGHT_YELLOW
        } else {
            Color::MEDIUM_GRAY
        };

        self.textbox
            .render_grid_with_color((Color::BLACK, cursor_color))
            .element()
            .fill_bg(Color::BLACK)
    }
}
impl NodularApplet for TextBoxApplet {
    fn handle_nodular_event(
        &mut self,
        nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        match nodular_event {
            singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
            singularity_sttk::nodular_applet::NodularEvent::Focused(focus) => {
                self.focused = focus;
                println!("Yay focus {focus}!");
                println!("The text is: {}", &self.textbox.get_text_as_string());

                self.hook.damage_window();
            }
        }
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        let mut title = self
            .textbox
            .get_text_as_string()
            .split_once('\n')
            .map_or("Bare Nodular Placeholder", |t| t.0.trim())
            .to_string();

        if title.is_empty() {
            title = "Bare Nodular Placeholder".to_string();
        }

        singularity_common::utils::tree::world_tree::WorldTree::Base(title)
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        // WorldTreePath::new_empty()
        WorldTreePath::new_into()
    }
}

fn main() {
    AppletRunner::run(
        singularity_sttk::nodular_applet::root_node_applet::RootNodeApplet::get_initializer(
            RecursiveNodeApplet::get_initializer(TextBoxApplet::get_boxed_initiator(String::new())),
        ),
    )
}
