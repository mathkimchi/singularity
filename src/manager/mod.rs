use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    subapp::{
        std_subapps::{editor::Editor, file_manager::FileManager, TextReader},
        Subapp, SubappData, SubappUI,
    },
};
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        style::Color,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Rect,
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::{Clear, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

pub struct Manager {
    subapps: RootedTree<Subapp>,

    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    focused_subapp_path: TreeNodePath,
    is_running: bool,
}
impl Manager {
    pub fn run_demo() -> io::Result<()> {
        // create demo manager
        let manager = {
            let mut subapps = RootedTree::from_root(Subapp {
                manager_proxy: Default::default(),
                subapp_data: SubappData {},
                user_interface: Box::new(FileManager::new("examples/project")),
            });
            subapps.add_node(
                Subapp {
                    manager_proxy: Default::default(),
                    subapp_data: SubappData {},
                    user_interface: TextReader::subapp_from_file(
                        "examples/project/lorem_ipsum.txt",
                    ),
                },
                &TreeNodePath::from([]),
            );
            subapps.add_node(
                Subapp {
                    manager_proxy: Default::default(),
                    subapp_data: SubappData {},
                    user_interface: Box::new(Editor::new("examples/project/file_to_edit.txt")),
                },
                &TreeNodePath::from([]),
            );

            Self {
                subapps,
                app_focuser_index: None,
                focused_subapp_path: TreeNodePath::new_root(),
                is_running: true,
            }
        };

        manager.run()
    }

    pub fn run(mut self) -> io::Result<()> {
        // set up terminal stuff
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        std::panic::set_hook(Box::new(|panic_info| {
            // In case there is a panic, revert the terminal to its original state
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);

            println!("{}", panic_info);
        }));

        while self.is_running {
            terminal.draw(|f| self.draw_app(f))?;
            self.handle_input(crossterm::event::read()?);
            self.process_subapp_commands();
        }

        // revert the terminal to its original state
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn draw_app(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.size());

        for (index, subapp_path) in self
            .subapps
            .iter_paths_dfs()
            .enumerate()
            .collect::<Vec<(usize, TreeNodePath)>>()
        {
            let subapp = &mut self.subapps[&subapp_path];

            subapp.user_interface.render(
                Rect::new(2 * subapp_path.depth() as u16, (12 * index) as u16, 50, 12),
                frame.buffer_mut(),
                &mut subapp.manager_proxy,
                subapp_path == self.focused_subapp_path,
            );
        }

        if let Some(focusing_index) = &self.app_focuser_index {
            let num_subapps = self.subapps.num_nodes();

            frame.render_widget(Clear, Rect::new(13, 5, 30, (2 + num_subapps) as u16));
            frame.render_widget(
                Paragraph::new("")
                    .block(ratatui::widgets::Block::bordered().title("Choose Subapp")),
                Rect::new(13, 5, 30, (2 + num_subapps) as u16),
            );

            for (index, subapp_path) in self.subapps.iter_paths_dfs().enumerate() {
                let subapp = &self.subapps[&subapp_path];

                let mut widget = Paragraph::new(subapp.user_interface.get_title().clone());

                if subapp_path == self.focused_subapp_path {
                    widget = widget.light_yellow().bold();
                }

                if subapp_path == focusing_index.clone() {
                    widget = widget.on_cyan();
                }

                frame.render_widget(
                    widget,
                    Rect::new(
                        (13 + 1 + 2 * subapp_path.depth()) as u16,
                        (5 + 1 + index) as u16,
                        subapp.user_interface.get_title().len() as u16,
                        1,
                    ),
                );
            }
        }
    }

    fn handle_input(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("Sayonara!");
                sleep(Duration::from_secs_f32(0.2));
                self.is_running = false;
            }

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::ALT,
                code,
                kind: KeyEventKind::Press,
                ..
            }) => {
                // Alt + arrows should be like alt tab for Windows and Linux but tree based
                // Alt + Enter either opens the subapp chooser or closes it and chooses the subapp

                if code == KeyCode::Enter && self.app_focuser_index.is_some() {
                    // save tree index and close window

                    let new_focus_index = self.app_focuser_index.take().unwrap();

                    self.focused_subapp_path = new_focus_index;
                } else {
                    let mut new_focus_index = self
                        .app_focuser_index
                        .clone()
                        .unwrap_or(self.focused_subapp_path.clone());

                    self.app_focuser_index = match code {
                        KeyCode::Enter => Some(new_focus_index),
                        KeyCode::Char('a') => {
                            new_focus_index = new_focus_index
                                .traverse_to_parent()
                                .unwrap_or(new_focus_index);
                            Some(new_focus_index)
                        }
                        KeyCode::Char('d') => {
                            new_focus_index = new_focus_index
                                .traverse_to_first_child(&self.subapps)
                                .unwrap_or(new_focus_index);
                            Some(new_focus_index)
                        }
                        KeyCode::Char('w') => {
                            new_focus_index = new_focus_index
                                .traverse_to_previous_sibling()
                                .unwrap_or(new_focus_index);
                            Some(new_focus_index)
                        }
                        KeyCode::Char('s') => {
                            new_focus_index = new_focus_index
                                .traverse_to_next_sibling(&self.subapps)
                                .unwrap_or(new_focus_index);
                            Some(new_focus_index)
                        }
                        _ => None,
                    };
                }
            }

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::SHIFT,
                code,
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("[Shift + {:?}] pressed.", code);
            }

            Event::Key(KeyEvent {
                modifiers: KeyModifiers::SHIFT,
                code,
                kind: KeyEventKind::Release,
                ..
            }) => {
                // Doesn't work
                println!("[Shift + {:?}] released.", code);
            }

            event => {
                let focused_subapp = &mut self.subapps[&self.focused_subapp_path];

                focused_subapp
                    .user_interface
                    .handle_input(&mut focused_subapp.manager_proxy, event);
            }
        }
    }

    /// Commands from subapp to manager
    fn process_subapp_commands(&mut self) {
        for subapp_path in self.subapps.iter_paths_dfs().collect::<Vec<TreeNodePath>>() {
            let commands = std::mem::take(&mut self.subapps[&subapp_path].manager_proxy.commands);

            for command in commands {
                match command {
                    ManagerCommand::SpawnSubapp(subapp_interface) => {
                        self.focused_subapp_path = self
                            .subapps
                            .add_node(
                                Subapp {
                                    manager_proxy: Default::default(),
                                    subapp_data: SubappData {},
                                    user_interface: subapp_interface,
                                },
                                &subapp_path,
                            )
                            .unwrap();
                    }
                }
            }
        }
    }
}

/// This allows subapps to request commands to the manager,
/// but is limited.
///
/// REVIEW: Maybe a box or mutex is better
#[derive(Default)]
pub struct ManagerProxy {
    commands: Vec<ManagerCommand>,
}
pub enum ManagerCommand {
    SpawnSubapp(Box<dyn SubappUI>),
}
impl ManagerProxy {
    pub fn request_spawn_child(&mut self, child_subapp_interface: Box<dyn SubappUI>) {
        self.commands
            .push(ManagerCommand::SpawnSubapp(child_subapp_interface));
    }
}
