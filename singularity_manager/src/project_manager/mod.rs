use singularity_common::{
    project::Project,
    tab::{
        packets::{Query, Request, Response},
        BasicTab, TabHandler,
    },
    utils::tree::tree_node_path::{TraversableTree, TreeNodePath},
};
use singularity_standard_tabs::{file_manager::FileManager, task_organizer::TaskOrganizer};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplaySize, DisplayUnits},
    ui_element::{CharCell, CharGrid, UIElement},
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
    UIDisplay,
};
use std::{
    io::{self},
    sync::{Arc, Mutex, RwLock},
    thread,
};
use tabs::Tabs;

mod tabs;

pub struct ProjectManager {
    _project: Project,

    tabs: Tabs,

    /// App focuser is a special window
    /// This value is None if the app focuser isn't being used, if Some then it represents the index that the user wants
    app_focuser_index: Option<TreeNodePath>,
    is_running: Arc<RwLock<bool>>,

    /// gui
    ui_element: Arc<Mutex<UIElement>>,
    ui_event_queue: Arc<Mutex<Vec<UIEvent>>>,
    // ui_window_px: [u32; 2],
}
impl ProjectManager {
    pub fn new<P>(project_directory: P) -> Self
    where
        P: 'static + AsRef<std::path::Path> + Clone + Send,
        std::path::PathBuf: From<P>,
    {
        let project = Project::new(project_directory.clone());

        Self {
            _project: project,
            tabs: {
                let mut tabs = Tabs::new(TabHandler::new(
                    FileManager::new_tab_creator(project_directory.clone()),
                    Self::generate_tab_area(0, 0),
                ));

                tabs.add(
                    TabHandler::new(
                        TaskOrganizer::new_tab_creator(project_directory),
                        Self::generate_tab_area(0, 1),
                    ),
                    &tabs.get_id_by_org_path(&TreeNodePath::new_root()).unwrap(),
                );

                tabs
            },
            app_focuser_index: None,
            is_running: Arc::new(RwLock::new(false)),
            ui_element: Arc::new(Mutex::new(UIElement::Container(Vec::new()))),
            ui_event_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run_demo() -> io::Result<()> {
        // create demo manager

        let mut manager = Self::new("examples/root-project");

        manager.tabs.add(
            TabHandler::new(
                singularity_common::components::timer::Timer::new_tab_creator((
                    std::time::Duration::from_secs(10),
                    false,
                )),
                Self::generate_tab_area(1, 1),
            ),
            &manager
                .tabs
                .get_id_by_org_path(&TreeNodePath::new_root())
                .unwrap(),
        );

        manager.run().unwrap();

        Ok(())
    }

    pub fn run(mut self) -> io::Result<()> {
        *self.is_running.write().unwrap() = true;

        let ui_element_clone = self.ui_element.clone();
        let ui_event_queue_clone = self.ui_event_queue.clone();
        let is_running_clone = self.is_running.clone();
        let ui_thread_handle = thread::spawn(move || {
            UIDisplay::run_display(ui_element_clone, ui_event_queue_clone, is_running_clone);
        });

        while *self.is_running.read().unwrap() {
            self.draw_app();
            self.handle_input();
            self.process_tab_requests();
            self.answer_tab_queries();
        }

        ui_thread_handle.join().unwrap();

        Ok(())
    }

    fn draw_app(&mut self) {
        let mut tab_elements = Vec::new();

        for tab_id in self.tabs.get_display_order().clone() {
            let tab = &mut self.tabs.get_mut_tab_handler(tab_id).unwrap();

            tab_elements.push(tab.get_ui_element().contain(tab.get_area()));
        }

        // display the tab focuser/selector
        if let Some(focusing_index) = &self.app_focuser_index {
            let mut subapps_focuser_display = CharGrid::default();

            for tab_path in self.tabs.iter_paths_dfs() {
                let tab_id = self.tabs.get_id_by_org_path(&tab_path).unwrap();
                let tab = self.tabs.get_tab_handler(tab_id).unwrap();

                let fg = if tab_id == self.tabs.get_focused_tab_id() {
                    Color::LIGHT_YELLOW
                } else {
                    Color::LIGHT_GREEN
                };

                let bg = if tab_path == focusing_index.clone() {
                    Color::CYAN
                } else {
                    Color::TRANSPARENT
                };

                let mut subapp_title_display = vec![
                    CharCell {
                        character: ' ',
                        fg: Color::TRANSPARENT,
                        bg: Color::TRANSPARENT
                    };
                    2 * tab_path.depth()
                ];

                for character in tab.tab_name.chars() {
                    subapp_title_display.push(CharCell { character, fg, bg });
                }

                subapps_focuser_display.content.push(subapp_title_display);
            }

            tab_elements.push(
                UIElement::CharGrid(subapps_focuser_display)
                    .fill_bg(Color::DARK_GRAY)
                    .bordered(Color::LIGHT_GREEN)
                    .contain(DisplayArea::from_center_half_size(
                        DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::HALF),
                        DisplaySize::new(0.2.into(), 0.2.into()),
                    )),
            );
        }

        *(self.ui_element.lock().unwrap()) =
            UIElement::Container(tab_elements).fill_bg(Color::BLACK);
    }

    fn handle_input(&mut self) {
        for ui_event in std::mem::take(&mut *(self.ui_event_queue.lock().unwrap())) {
            use singularity_ui::ui_event::UIEvent;
            match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.raw_code == 16 => {
                    // Ctrl+Q
                    dbg!("Goodbye!");
                    *self.is_running.write().unwrap() = false;
                }
                UIEvent::KeyPress(key, KeyModifiers::ALT)
                    if matches!(key.to_char(), Some('\n' | 'w' | 'a' | 's' | 'd')) =>
                {
                    // Alt + arrows should be like alt tab for Windows and Linux but tree based
                    // Alt + Enter either opens the tab chooser or closes it and chooses the tab

                    if key.to_char() == Some('\n') && self.app_focuser_index.is_some() {
                        // save tree index and close window

                        let new_focus_index = self.app_focuser_index.take().unwrap();

                        self.tabs.set_focused_tab_path(&new_focus_index);
                    } else {
                        let mut new_focus_index = self.app_focuser_index.clone().unwrap_or(
                            self.tabs
                                .get_tab_path(&self.tabs.get_focused_tab_id())
                                .unwrap()
                                .clone(),
                        );

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
                    // dbg!(&self.focused_tab_path);
                }
                UIEvent::KeyPress(key, KeyModifiers::ALT) if key.raw_code == 103 => {
                    // Alt+ArrowUp
                    // TODO: figure out why Ctrl+Shift+ArrowUp specifically doesn't work...

                    // maximize focused tab
                    let focused_tab = self.tabs.get_focused_tab_mut();

                    focused_tab.set_area(DisplayArea::from_corner_size(
                        DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
                        DisplaySize::new(DisplayUnits::FULL, DisplayUnits::FULL),
                    ));
                }
                UIEvent::KeyPress(key, KeyModifiers::ALT) if key.raw_code == 108 => {
                    self.tabs.minimize_focused_tab();
                }
                UIEvent::KeyPress(key, KeyModifiers::CTRL | KeyModifiers::SHIFT)
                    if key.to_char() == Some('w') =>
                {
                    // self.tabs.close_focused_tab_recursively();
                }
                UIEvent::KeyPress(_, _) => {
                    // forward the event to focused tab
                    let focused_tab = self.tabs.get_focused_tab_mut();

                    focused_tab
                        .send_event(singularity_common::tab::packets::Event::UIEvent(ui_event));
                }
                UIEvent::WindowResized(_ui_window_px) => {
                    // self.ui_window_px = ui_window_px;
                }
                UIEvent::MousePress([[click_x, click_y], [tot_width, tot_height]], container) => {
                    assert_eq!(container, DisplayArea::FULL);

                    // if pressed on focused tab, then forward the click
                    {
                        let focused_tab = self
                            .tabs
                            .get_tab_handler(self.tabs.get_focused_tab_id())
                            .unwrap();
                        if focused_tab.get_area().contains(
                            DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                            [tot_width as i32, tot_height as i32],
                        ) {
                            focused_tab.send_event(
                                singularity_common::tab::packets::Event::UIEvent(
                                    singularity_ui::ui_event::UIEvent::MousePress(
                                        [[click_x, click_y], [tot_width, tot_height]],
                                        focused_tab.get_area().map_onto(container),
                                    ),
                                ),
                            );
                        }
                    }

                    // if pressed on unfocused tab, make that focused
                    for tab_id in self.tabs.get_display_order().iter().rev() {
                        let tab = self.tabs.get_tab_handler(*tab_id).unwrap();
                        let tab_area = tab.get_area();

                        if tab_area.contains(
                            DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                            [tot_width as i32, tot_height as i32],
                        ) {
                            self.tabs.set_focused_tab_id(*tab_id);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Requests from tab to manager
    fn process_tab_requests(&mut self) {
        for requestor_path in self.tabs.collect_paths_dfs() {
            let requests = self
                .tabs
                .get_tab_handler(self.tabs.get_id_by_org_path(&requestor_path).unwrap())
                .unwrap()
                .collect_requests();

            for request in requests {
                match request {
                    Request::ChangeName(new_name) => {
                        self.tabs
                            .get_mut_tab_handler(
                                self.tabs.get_id_by_org_path(&requestor_path).unwrap(),
                            )
                            .unwrap()
                            .tab_name = new_name;
                    }
                    Request::SpawnChildTab(tab_creator) => {
                        self.tabs.add(
                            TabHandler::new(
                                tab_creator,
                                // NOTE: the argument child index is technically incorrect,
                                // but the purpose of the generator is to generally prevent all
                                // tabs from being spawned all in one place.
                                Self::generate_tab_area(
                                    self.tabs.num_tabs(),
                                    requestor_path.depth() + 1,
                                ),
                            ),
                            &self.tabs.get_id_by_org_path(&requestor_path).unwrap(),
                        );
                    }
                }
            }
        }
    }

    fn answer_tab_queries(&self) {
        for tab_path in self.tabs.collect_paths_dfs() {
            let inquieror = self
                .tabs
                .get_tab_handler(self.tabs.get_id_by_org_path(&tab_path).unwrap())
                .unwrap();
            inquieror.answer_query(move |query| match query {
                Query::Path => Response::Path(tab_path.clone()),
                Query::Name => Response::Name(inquieror.tab_name.clone()),
            });
        }
    }

    fn generate_tab_area(child_index: usize, depth: usize) -> DisplayArea {
        const WIDTH: f32 = 0.5;
        const HEIGHT: f32 = 0.5;

        let child_index = child_index as f32;
        let depth = depth as f32;
        DisplayArea::from_corner_size(
            DisplayCoord::new(
                ((0.1 * depth + 0.01 * child_index) % WIDTH).into(),
                ((0.2 * child_index) % HEIGHT).into(),
            ),
            DisplaySize::new(WIDTH.into(), HEIGHT.into()),
        )
    }
}
impl Drop for ProjectManager {
    fn drop(&mut self) {
        // revert the terminal to its original state
        // drop is called even on panic
    }
}
