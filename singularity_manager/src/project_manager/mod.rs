use singularity_common::{
    project::Project,
    tab::{
        basic_tab_creator,
        packets::{Event, Query, Request, Response},
        TabHandler,
    },
    utils::tree::{
        rooted_tree::RootedTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use singularity_standard_tabs::file_manager::FileManager;
use std::{
    io::{self, stdout},
    thread,
    time::Duration,
};

pub struct ProjectManager {
    project: Project,

    tabs: RootedTree<TabHandler>,

    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    focused_tab_path: TreeNodePath,
    is_running: bool,
}
impl ProjectManager {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: AsRef<std::path::Path> + Clone + Send,
        std::path::PathBuf: From<P>,
    {
        let project = Project::new(project_directory.clone());

        Self {
            project,
            // running_subapps: RootedTree::from_root(Subapp::new(FileManager::new(
            //     project_directory,
            // ))),
            tabs: RootedTree::from_root(todo!()),
            app_focuser_index: None,
            focused_tab_path: TreeNodePath::new_root(),
            is_running: false,
        }
    }

    pub fn run_demo() -> io::Result<()> {
        // create demo manager

        // let manager = Self::new("examples/root-project");

        // manager.running_subapps.add_node(
        //     Subapp::new(TaskOrganizer::new(
        //         "examples/root-project/.project/tasks.json",
        //     )),
        //     &TreeNodePath::new_root(),
        // );

        // let manager = Self {
        //     project: Project::new("examples/root-project"),
        //     tabs: RootedTree::from_root(TabHandler::new(basic_tab_creator(
        //         "examples/root-project/file_to_edit.txt",
        //         Editor::new,
        //         Editor::render,
        //         Editor::handle_event,
        //     ))),
        //     app_focuser_index: None,
        //     focused_tab_path: TreeNodePath::new_root(),
        //     is_running: false,
        // };

        let mut manager = Self {
            project: Project::new("examples/root-project"),
            tabs: RootedTree::from_root(TabHandler::new(basic_tab_creator(
                "examples/root-project",
                FileManager::new,
                FileManager::render,
                FileManager::handle_event,
            ))),
            app_focuser_index: None,
            focused_tab_path: TreeNodePath::new_root(),
            is_running: false,
        };

        // let event_loop = EventLoop::new().unwrap();
        // event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        // dbg!("will run app");
        // event_loop.run_app(&mut manager).unwrap();

        // dbg!("ran app");

        manager.run()
    }

    pub fn run(mut self) -> io::Result<()> {
        self.is_running = true;

        // while self.is_running {
        //     // terminal.draw(|f| self.draw_app(f))?;
        //     self.handle_input();
        //     self.process_tab_requests();
        //     self.answer_tab_queries();
        // }

        Ok(())
    }

    // fn draw_app(&mut self, frame: &mut Frame) {
    //     frame.render_widget(Clear, frame.area());

    //     for (index, tab_path) in self.tabs.collect_paths_dfs().into_iter().enumerate() {
    //         let tab = &mut self.tabs[&tab_path];

    //         let total_tab_area =
    //             Rect::new(2 * tab_path.depth() as u16, (12 * index) as u16, 50, 12);
    //         let inner_tab_area = total_tab_area.inner(Margin::new(1, 1));

    //         // TODO: only send on actual resize
    //         tab.send_event(Event::Resize(inner_tab_area));

    //         frame.buffer_mut().merge(&ratatui::buffer::Buffer {
    //             area: inner_tab_area,
    //             // REVIEW: not sure about the cost of this
    //             // NOTE: the vec length needs to be at least the area and is allowed to be more
    //             content: tab.get_display_buffer(inner_tab_area.area() as usize),
    //         });

    //         ratatui::widgets::Block::bordered()
    //             .title(tab.tab_name.clone())
    //             .render(total_tab_area, frame.buffer_mut());
    //     }

    //     if let Some(focusing_index) = &self.app_focuser_index {
    //         let num_subapps = self.tabs.num_nodes();

    //         frame.render_widget(Clear, Rect::new(13, 5, 30, (2 + num_subapps) as u16));
    //         frame.render_widget(
    //             Paragraph::new("").block(ratatui::widgets::Block::bordered().title("Choose Tab")),
    //             Rect::new(13, 5, 30, (2 + num_subapps) as u16),
    //         );

    //         for (index, tab_path) in self.tabs.iter_paths_dfs().enumerate() {
    //             let tab = &self.tabs[&tab_path];

    //             let mut widget = Paragraph::new(tab.tab_name.clone());

    //             if tab_path == self.focused_tab_path {
    //                 widget = widget.light_yellow().bold();
    //             }

    //             if tab_path == focusing_index.clone() {
    //                 widget = widget.on_cyan();
    //             }

    //             frame.render_widget(
    //                 widget,
    //                 Rect::new(
    //                     (13 + 1 + 2 * tab_path.depth()) as u16,
    //                     (5 + 1 + index) as u16,
    //                     tab.tab_name.len() as u16,
    //                     1,
    //                 ),
    //             );
    //         }
    //     }
    // }

    // fn handle_input(&mut self) {
    //     if crossterm::event::poll(Duration::ZERO).unwrap() {
    //         self.process_input(crossterm::event::read().unwrap());
    //     }
    // }

    // fn process_input(&mut self, event: TUIEvent) {
    //     match event {
    //         TUIEvent::Key(KeyEvent {
    //             modifiers: KeyModifiers::CONTROL,
    //             code: KeyCode::Char('q'),
    //             kind: KeyEventKind::Press,
    //             ..
    //         }) => {
    //             println!("Sayonara!");
    //             thread::sleep(Duration::from_secs_f32(0.2));
    //             self.is_running = false;
    //         }

    //         TUIEvent::Key(KeyEvent {
    //             modifiers: KeyModifiers::ALT,
    //             code,
    //             kind: KeyEventKind::Press,
    //             ..
    //         }) if matches!(
    //             code,
    //             KeyCode::Enter
    //                 | KeyCode::Char('w')
    //                 | KeyCode::Char('a')
    //                 | KeyCode::Char('s')
    //                 | KeyCode::Char('d')
    //         ) =>
    //         {
    //             // Alt + arrows should be like alt tab for Windows and Linux but tree based
    //             // Alt + Enter either opens the tab chooser or closes it and chooses the tab

    //             if code == KeyCode::Enter && self.app_focuser_index.is_some() {
    //                 // save tree index and close window

    //                 let new_focus_index = self.app_focuser_index.take().unwrap();

    //                 self.focused_tab_path = new_focus_index;
    //             } else {
    //                 let mut new_focus_index = self
    //                     .app_focuser_index
    //                     .clone()
    //                     .unwrap_or(self.focused_tab_path.clone());

    //                 self.app_focuser_index = match code {
    //                     KeyCode::Enter => Some(new_focus_index),
    //                     KeyCode::Char(traverse_key)
    //                         if matches!(traverse_key, 'w' | 'a' | 's' | 'd') =>
    //                     {
    //                         new_focus_index = new_focus_index
    //                             .clamped_traverse_based_on_wasd(&self.tabs, traverse_key);
    //                         Some(new_focus_index)
    //                     }
    //                     _ => None,
    //                 };
    //             }
    //         }

    //         event => {
    //             // forward the event to focused tab
    //             let focused_tab = &mut self.tabs[&self.focused_tab_path];

    //             focused_tab.send_event(singularity_common::tab::packets::Event::UIEvent(event));
    //         }
    //     }
    // }

    /// Requests from tab to manager
    fn process_tab_requests(&mut self) {
        for requestor_path in self.tabs.collect_paths_dfs() {
            let requests = self.tabs[&requestor_path].collect_requests();

            for request in requests {
                match request {
                    Request::ChangeName(new_name) => {
                        self.tabs[&requestor_path].tab_name = new_name;
                    }
                    Request::SpawnChildTab(tab_creator) => {
                        self.focused_tab_path = self
                            .tabs
                            .add_node(TabHandler::new(tab_creator), &requestor_path)
                            .unwrap();
                    }
                }
            }
        }
    }

    fn answer_tab_queries(&self) {
        for tab_path in self.tabs.collect_paths_dfs() {
            let inquieror = &self.tabs[&tab_path];
            inquieror.answer_query(move |query| match query {
                Query::Path => Response::Path(tab_path.clone()),
                Query::Name => Response::Name(inquieror.tab_name.clone()),
            });
        }
    }
}
impl Drop for ProjectManager {
    fn drop(&mut self) {
        // revert the terminal to its original state
        // drop is called even on panic
    }
}
