use std::fmt::Write as _;

use singularity_common::utils::tree::world_tree::{
    WorldTreePath, world_tree_traversal::WorldTreeTraversalOperation,
};
use singularity_sar::applet::BasicApplet;
use singularity_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
};

use crate::nodular_applet::{
    NodularApplet, NodularEvent, NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet,
};

/// Technically, this is a standard app, but it is so essential
/// that I am considering it a tool for sttk.
///
/// This is a like cross between a terminal and any fast command palette
/// like Windows' start menu (the thing that pops up when you press Windows).
pub struct CommandHubApplet {
    title: String,

    /// Vec of the previous (command, output)
    history: Vec<(String, String)>,
    /// The thing that is currently being typed.
    current_prompt: String,

    hook: Box<dyn NodularRunnerHook>,
}
impl CommandHubApplet {
    pub fn new(hook: Box<dyn NodularRunnerHook>) -> Self {
        Self {
            title: "Command Hub".to_string(),
            history: Vec::new(),
            current_prompt: String::new(),
            hook,
        }
    }
    pub fn get_initiator() -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self {
        |hook: Box<dyn NodularRunnerHook>| Self::new(hook)
    }
    pub fn get_boxed_initiator() -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>
    {
        |hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(hook))
    }

    fn execute_command(command: &str) -> Option<String> {
        let tokens: Vec<_> = command.split_whitespace().collect();

        let (verb, args) = tokens.split_first()?;

        match *verb {
            "add" => Some(
                args.iter()
                    .filter_map(|a| a.parse::<f32>().ok())
                    .sum::<f32>()
                    .to_string(),
            ),

            // TODO: run command
            "$" => todo!(),
            _ => None,
        }
    }

    fn handle_enter(&mut self) {
        let command = std::mem::take(&mut self.current_prompt);

        let output = Self::execute_command(&command);
        self.history
            .push((command, output.unwrap_or("Err".to_string())));
    }
}
impl BasicApplet for CommandHubApplet {
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

        if let UIEvent::KeyPress(key, KeyModifiers::SHIFT) = &ui_event
            && key.to_char() == Some('+')
        {
            // self.hook
            //     .add_child(Box::new(TextBoxApplet::get_boxed_initiator(String::new())));
            self.hook
                .add_child(Box::new(RecursiveNodeApplet::get_boxed_initializer(
                    Self::get_boxed_initiator(),
                )));
            return;
        }

        if let UIEvent::KeyPress(key, KeyModifiers::ALT) = &ui_event
            && key.to_char() == Some('q')
        {
            self.hook
                .change_focus(WorldTreeTraversalOperation::PrevLayer);
            return;
        }

        // above are special cases
        // now actually handle events for self

        if let UIEvent::KeyPress(key, _) = &ui_event
            && let Some(key_char) = key.to_char()
        {
            match key_char {
                '\n' => self.handle_enter(),
                '\u{8}' => {
                    self.current_prompt.pop();
                }
                _ => self.current_prompt.push(key_char),
            }
        }

        self.hook.damage_window();
        self.hook.damage_treeview();
    }

    fn get_window(&self) -> UIElement {
        let mut display_string = String::new();

        // print the last 10 from history
        for (prompt, output) in &self.history[self.history.len().saturating_sub(10)..] {
            writeln!(&mut display_string, "> {prompt}").unwrap();
            writeln!(&mut display_string, "{output}").unwrap();
        }
        writeln!(&mut display_string, "> {}", self.current_prompt).unwrap();

        CharGrid::from(display_string)
            .element()
            .fill_bg(Color::BLACK)
    }
}
impl NodularApplet for CommandHubApplet {
    fn handle_nodular_event(&mut self, _nodular_event: NodularEvent) {
        println!("Warning: ignorinng this for now");
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.title.clone())
    }

    fn get_focus_path(&self) -> singularity_common::utils::tree::world_tree::WorldTreePath {
        // WorldTreePath::new_empty()
        WorldTreePath::new_into()
    }
}
