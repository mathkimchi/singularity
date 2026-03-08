use crate::{
    nodular_applet::{NodularApplet, NodularEvent, NodularRunnerHook},
    standard_keybinds::handle_standard_keybinds,
};
use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::BasicApplet;
use singularity_ui::{
    color::Color,
    ui_element::{CharGrid, UIElement},
    ui_event::{KeyTrait, UIEvent},
};
use std::fmt::Write as _;

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

    fn execute_command(&mut self, command: &str) -> Result<String, String> {
        let tokens: Vec<_> = command.split_whitespace().collect();

        let (verb, args) = tokens
            .split_first()
            .ok_or("Couldn't split commands.".to_string())?;

        match *verb {
            "addition" => Ok(args
                .iter()
                .filter_map(|a| a.parse::<f32>().ok())
                .sum::<f32>()
                .to_string()),
            "add_child" => match args.first() {
                None | Some(&"command_hub") => {
                    self.hook
                        .add_child(Box::new(CommandHubApplet::get_boxed_initiator()));
                    Ok("Great success!".to_string())
                }
                Some(app_name) => Err(format!("Couldn't find app {app_name}.")),
            },
            "set_title" => {
                self.title = command
                    .split_once(' ')
                    .ok_or("Couldn't get title from \"{command}\".")?
                    .1
                    .to_string();
                Ok("Done!".to_string())
            }

            "help" => Ok("Currently working commands: `add`, `set_title`, `$`".to_string()),

            // TODO: help function

            // FIXME: currently waits for command to finish.
            // This is stupid, figure out another way.
            "$" => {
                let mut command = std::process::Command::new(args[0]);
                command.args(&args[1..]);

                // for some reason, String::from_utf8 doesn't work but debug does
                String::from_utf8(command.output().map_err(|err| err.to_string())?.stdout)
                    .map_err(|err| err.to_string())
            }
            verb => Err(format!("`{verb}` is an unknown command.")),
        }
    }

    fn handle_enter(&mut self) {
        let command = std::mem::take(&mut self.current_prompt);

        let output = self.execute_command(&command);
        self.history.push((
            command,
            match output {
                Ok(ok) => ok,
                Err(err) => format!("Err: {err}"),
            },
        ));
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

        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

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
