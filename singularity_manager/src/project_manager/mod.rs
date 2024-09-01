use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Rect,
    prelude::CrosstermBackend,
    style::Stylize,
    widgets::{Clear, Paragraph},
    Frame, Terminal,
};
use singularity_common::{
    project::Project,
    subapp::{Subapp, SubappData, SubappUI},
    utils::tree::{
        rooted_tree::RootedTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use std::{
    io::{self, stdout},
    thread::sleep,
    time::Duration,
};

pub struct ProjectManager {
    project: Project,

    running_subapps: RootedTree<Subapp>,

    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    focused_subapp_path: TreeNodePath,
    is_running: bool,
}
impl ProjectManager {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path> + Clone,
        std::path::PathBuf: From<P>,
    {
        let project = Project::new(project_directory.clone());

        Self {
            project,
            // running_subapps: RootedTree::from_root(Subapp::new(FileManager::new(
            //     project_directory,
            // ))),
            running_subapps: RootedTree::from_root(todo!()),
            app_focuser_index: None,
            focused_subapp_path: TreeNodePath::new_root(),
            is_running: false,
        }
    }

    pub fn run_demo() -> io::Result<()> {
        // create demo manager
        let mut manager = Self::new("examples/root-project");

        // manager.running_subapps.add_node(
        //     Subapp::new(TaskOrganizer::new(
        //         "examples/root-project/.project/tasks.json",
        //     )),
        //     &TreeNodePath::new_root(),
        // );

        manager.run()
    }

    pub fn run(mut self) -> io::Result<()> {
        self.is_running = true;

        // set up terminal stuff
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        while self.is_running {
            terminal.draw(|f| self.draw_app(f))?;
            self.handle_input(crossterm::event::read()?);
            self.process_subapp_commands();
        }

        Ok(())
    }

    fn draw_app(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());

        for (index, subapp_path) in self
            .running_subapps
            .iter_paths_dfs()
            .enumerate()
            .collect::<Vec<(usize, TreeNodePath)>>()
        {
            let subapp = &mut self.running_subapps[&subapp_path];

            subapp.user_interface.render(
                Rect::new(2 * subapp_path.depth() as u16, (12 * index) as u16, 50, 12),
                frame.buffer_mut(),
                subapp_path == self.focused_subapp_path,
            );
        }

        if let Some(focusing_index) = &self.app_focuser_index {
            let num_subapps = self.running_subapps.num_nodes();

            frame.render_widget(Clear, Rect::new(13, 5, 30, (2 + num_subapps) as u16));
            frame.render_widget(
                Paragraph::new("")
                    .block(ratatui::widgets::Block::bordered().title("Choose Subapp")),
                Rect::new(13, 5, 30, (2 + num_subapps) as u16),
            );

            for (index, subapp_path) in self.running_subapps.iter_paths_dfs().enumerate() {
                let subapp = &self.running_subapps[&subapp_path];

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
                        KeyCode::Char(traverse_key)
                            if matches!(traverse_key, 'w' | 'a' | 's' | 'd') =>
                        {
                            new_focus_index = new_focus_index.clamped_traverse_based_on_wasd(
                                &self.running_subapps,
                                traverse_key,
                            );
                            Some(new_focus_index)
                        }
                        _ => None,
                    };
                }
            }

            event => {
                let focused_subapp = &mut self.running_subapps[&self.focused_subapp_path];

                focused_subapp.user_interface.handle_input(event);
            }
        }
    }

    /// Commands from subapp to manager
    fn process_subapp_commands(&mut self) {
        // for subapp_path in self
        //     .running_subapps
        //     .iter_paths_dfs()
        //     .collect::<Vec<TreeNodePath>>()
        // {
        //     let commands =
        //         std::mem::take(&mut self.running_subapps[&subapp_path].manager_proxy.commands);

        //     for command in commands {
        //         match command {
        //             ManagerCommand::SpawnSubapp(subapp_interface) => {
        //                 self.focused_subapp_path = self
        //                     .running_subapps
        //                     .add_node(
        //                         Subapp {
        //                             manager_proxy: Default::default(),
        //                             subapp_data: SubappData {},
        //                             user_interface: subapp_interface,
        //                         },
        //                         &subapp_path,
        //                     )
        //                     .unwrap();
        //             }
        //         }
        //     }
        // }
    }
}
impl Drop for ProjectManager {
    fn drop(&mut self) {
        // revert the terminal to its original state
        // bc of drop, this is called even on panic
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
    }
}
