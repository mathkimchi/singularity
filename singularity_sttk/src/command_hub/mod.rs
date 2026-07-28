//! TODO: move to sttk

use crate::{
    nodular_applet::{
        AppletSpawner, AppletSpawnerTrait, NodularApplet, NodularAppletInitializer, NodularEvent,
        NodularRunnerHook, recursive_node_applet::RecursiveNodeApplet,
    },
    standard_keybinds::handle_standard_keybinds,
};
use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::BasicApplet;
use sonamu_ui::{
    color::Color, display_units::DisplayContainerSize, ui_element::UIElement, ui_event::UIEvent,
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
    #[must_use]
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
    #[must_use]
    pub fn get_boxed_initiator() -> NodularAppletInitializer {
        Box::new(|hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(hook)))
    }
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct CommandHubSpawner;
        impl AppletSpawnerTrait for CommandHubSpawner {
            fn create_initializer(
                &self,
                args: &[&str],
            ) -> Option<NodularAppletInitializer> {
                if !args.is_empty() {
                    println!("Warning: command hub doesn't use spawn args.");
                }

                Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
                    CommandHubApplet::get_boxed_initiator(),
                ))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(CommandHubSpawner)
    }

    fn execute_command(&mut self, command: &str) -> Result<String, String> {
        let tokens: Vec<_> = command.split_whitespace().collect();

        let (verb, args) = tokens
            .split_first()
            .ok_or_else(|| "Couldn't split commands.".to_string())?;

        match *verb {
            "addition" => Ok(args
                .iter()
                .filter_map(|a| a.parse::<f32>().ok())
                .sum::<f32>()
                .to_string()),
            "add_child" => match args.first() {
                None => {
                    self.hook
                        .add_child(RecursiveNodeApplet::boxed_get_boxed_initializer(
                            Self::get_boxed_initiator(),
                        ));
                    Ok("Unspecified child defaulting to command_hub.".to_string())
                }
                Some(app_name) => match self.hook.find_applet_spawner(app_name.to_string()) {
                    Some(applet_spawner) => match applet_spawner.create_initializer(&args[1..]) {
                        Some(applet_initializer) => {
                            self.hook.add_child(applet_initializer);
                            Ok("Great success!".to_string())
                        }
                        None => Err(format!(
                            "Couldn't create initializer from args {:?}.",
                            &args[1..]
                        )),
                    },
                    None => Err(format!(
                        "Couldn't find app {app_name}. Known applets: {:?}.",
                        self.hook.get_applet_spawners().keys()
                    )),
                },
            },
            "set_title" => {
                self.title = command
                    .split_once(' ')
                    .ok_or_else(|| format!("Couldn't get title from \"{command}\"."))?
                    .1
                    .to_string();
                Ok("Done!".to_string())
            }

            "test" => match args.first() {
                Some(&"image") => {
                    self.execute_command("add_child image_viewer examples/sonamu.jpg")
                }
                Some(&"wl") => self.execute_command("add_child wl_app"),
                Some(&"editor") => self.execute_command(
                    "add_child code_editor examples/root-project/file_to_edit.txt",
                ),
                Some(&"text_editor") => self.execute_command(
                    "add_child text_editor examples/root-project/file_to_edit.txt",
                ),
                Some(&"char") => self.execute_command("add_child char_render_test"),
                Some(test) => Err(format!(
                    "`{test}` is an unknown test. Currently working tests: `image`"
                )),
                None => Err("Must specify test.".to_string()),
            },

            "help" => Ok(
                "Currently working commands: `add_child`, `set_title`, `help`, `addition`, `$`"
                    .to_string(),
            ),

            // This is stupid, figure out another way.
            "$" => {
                let mut command = std::process::Command::new(args[0]);
                command.args(&args[1..]);

                // for some reason, String::from_utf8 doesn't work but debug does
                String::from_utf8(command.output().map_err(|err| err.to_string())?.stdout)
                    .map_err(|err| err.to_string())
            }
            verb => Err(format!(
                "`{verb}` is an unknown command. Currently working commands: `add_child`, `set_title`, `help`, `addition`, `$`"
            )),
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
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

        if let UIEvent::KeyPress(key, _) = &ui_event {
            match key {
                sonamu_ui::ui_event::Key::Enter => self.handle_enter(),
                sonamu_ui::ui_event::Key::Backspace => {
                    self.current_prompt.pop();
                }
                sonamu_ui::ui_event::Key::Char(key_char) => {
                    self.current_prompt.push(*key_char);
                }
                _ => {}
            }
        }

        self.hook.damage_window();
        self.hook.damage_treeview();
    }

    fn get_window(&self, _: DisplayContainerSize) -> UIElement {
        let mut display_string = String::new();

        // print the last 10 from history
        for (prompt, output) in &self.history[self.history.len().saturating_sub(10)..] {
            writeln!(&mut display_string, "> {prompt}").unwrap();
            writeln!(&mut display_string, "{output}").unwrap();
        }
        writeln!(&mut display_string, "> {}", self.current_prompt).unwrap();

        UIElement::from(display_string).fill_bg(Color::BLACK)
    }
}
impl NodularApplet for CommandHubApplet {
    fn handle_nodular_event(&mut self, _nodular_event: NodularEvent) {
        println!("Warning: ignorinng this for now");
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.title.clone())
    }

    fn get_focus_path(&self) -> WorldTreePath {
        // WorldTreePath::new_empty()
        WorldTreePath::new_into()
    }
}
