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
use singularity_standard_tabs::editor::Editor;
use singularity_ui::{
    display_units::{DisplayArea, DisplayCoord, DisplaySize},
    ui_event::{Key, KeyModifiers, KeyTrait, UIEvent},
    CharCell, CharGrid, Color32, UIDisplay, UIElement,
};
use std::{
    io::{self},
    sync::{Arc, Mutex},
    thread,
};

pub struct ProjectManager {
    project: Project,

    tabs: RootedTree<TabHandler>,

    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    focused_tab_path: TreeNodePath,
    is_running: bool,

    /// gui
    ui_element: Arc<Mutex<UIElement>>,
    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
}
impl ProjectManager {
    pub fn new<P>(_project_directory: P) -> Self
    where
        P: AsRef<std::path::Path> + Clone + Send,
        std::path::PathBuf: From<P>,
    {
        // let project = Project::new(project_directory.clone());

        // Self {
        //     project,
        //     // running_subapps: RootedTree::from_root(Subapp::new(FileManager::new(
        //     //     project_directory,
        //     // ))),
        //     tabs: RootedTree::from_root(todo!()),
        //     app_focuser_index: None,
        //     focused_tab_path: TreeNodePath::new_root(),
        //     is_running: false,
        //     ui_element: todo!(),
        // }
        todo!()
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
                "examples/root-project/file_to_edit.txt",
                Editor::new,
                Editor::render,
                Editor::handle_event,
            ))),
            app_focuser_index: None,
            focused_tab_path: TreeNodePath::new_root(),
            is_running: false,
            ui_element: Arc::new(Mutex::new(UIElement::Container(Vec::new()))),
            ui_event_queue: Arc::new(Mutex::new(Vec::new())),
        };

        manager.tabs.add_node(
            TabHandler::new(basic_tab_creator(
                "examples/root-project/lorem_ipsum.txt",
                Editor::new,
                Editor::render,
                Editor::handle_event,
            )),
            &TreeNodePath::new_root(),
        );

        let ui_element_clone = manager.ui_element.clone();
        let ui_event_queue_clone = manager.ui_event_queue.clone();
        let ui_thread_handle = thread::spawn(move || {
            UIDisplay::run_display(ui_element_clone, ui_event_queue_clone);
        });

        manager.run().unwrap();
        ui_thread_handle.join().unwrap();

        Ok(())
    }

    pub fn run(mut self) -> io::Result<()> {
        self.is_running = true;

        while self.is_running {
            self.draw_app();
            self.handle_input();
            self.process_tab_requests();
            self.answer_tab_queries();
        }

        Ok(())
    }

    fn draw_app(&mut self) {
        let mut tab_elements = Vec::new();

        for (index, tab_path) in self.tabs.collect_paths_dfs().into_iter().enumerate() {
            let tab = &mut self.tabs[&tab_path];

            const TAB_DELTA_Y: f32 = 200.0;

            let display_area = DisplayArea(
                DisplayCoord::new(
                    20.0 * (tab_path.depth() as f32),
                    TAB_DELTA_Y * (index as f32),
                ),
                DisplayCoord::new(
                    20.0 * (tab_path.depth() as f32) + 50.0,
                    TAB_DELTA_Y * (index as f32) + (TAB_DELTA_Y - 10.0),
                ),
            );
            let tab_inner_area = display_area;

            // TODO: only send on actual resize
            tab.send_event(Event::Resize(tab_inner_area));

            tab_elements.push((tab.get_ui_element().bordered(), tab_inner_area));
        }

        if let Some(focusing_index) = &self.app_focuser_index {
            let mut subapps_focuser_display = CharGrid::default();

            for tab_path in self.tabs.iter_paths_dfs() {
                let tab = &self.tabs[&tab_path];

                let fg = if tab_path == self.focused_tab_path {
                    Color32::LIGHT_YELLOW
                } else {
                    Color32::LIGHT_GREEN
                };

                let bg = if tab_path == focusing_index.clone() {
                    Color32::LIGHT_BLUE
                } else {
                    Color32::TRANSPARENT
                };

                let mut subapp_title_display = vec![
                    CharCell {
                        character: ' ',
                        fg: Color32::TRANSPARENT,
                        bg: Color32::TRANSPARENT
                    };
                    2 * tab_path.depth()
                ];

                for character in tab.tab_name.chars() {
                    subapp_title_display.push(CharCell { character, fg, bg });
                }

                subapps_focuser_display.content.push(subapp_title_display);
            }

            tab_elements.push((
                UIElement::CharGrid(subapps_focuser_display).bordered(),
                DisplayArea::from_coord_size(
                    DisplayCoord::new(400.0, 400.0),
                    DisplaySize::new(100.0, 100.0),
                ),
            ));
        }

        *(self.ui_element.lock().unwrap()) = UIElement::Container(tab_elements);
    }

    fn handle_input(&mut self) {
        for ui_event in std::mem::take(&mut *(self.ui_event_queue.lock().unwrap())) {
            match ui_event {
                UIEvent::Key {
                    key: Key::Q,
                    modifiers,
                    pressed: true,
                    ..
                } if modifiers.command_only() => {
                    dbg!("Goodbye!");
                    self.is_running = false;
                }
                UIEvent::Key {
                    key,
                    modifiers: KeyModifiers::ALT,
                    pressed: true,
                    ..
                } if matches!(key, Key::Enter | Key::W | Key::A | Key::S | Key::D) => {
                    // Alt + arrows should be like alt tab for Windows and Linux but tree based
                    // Alt + Enter either opens the tab chooser or closes it and chooses the tab

                    if key == Key::Enter && self.app_focuser_index.is_some() {
                        // save tree index and close window

                        let new_focus_index = self.app_focuser_index.take().unwrap();

                        self.focused_tab_path = new_focus_index;
                    } else {
                        let mut new_focus_index = self
                            .app_focuser_index
                            .clone()
                            .unwrap_or(self.focused_tab_path.clone());

                        self.app_focuser_index = match key.to_char() {
                            Some('\n') => Some(new_focus_index),
                            Some(traverse_key) if matches!(traverse_key, 'w' | 'a' | 's' | 'd') => {
                                new_focus_index = new_focus_index
                                    .clamped_traverse_based_on_wasd(&self.tabs, traverse_key);
                                Some(new_focus_index)
                            }
                            _ => panic!(),
                        };
                    }
                    dbg!(&self.app_focuser_index);
                    dbg!(&self.focused_tab_path);
                }
                ui_event => {
                    // forward the event to focused tab
                    let focused_tab = &mut self.tabs[&self.focused_tab_path];

                    focused_tab
                        .send_event(singularity_common::tab::packets::Event::UIEvent(ui_event));
                }
            }
        }
    }

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
