use singularity_common::{
    project::Project,
    tab::{packets::Request, tile::Tile, TabHandler},
    utils::{
        id_map::Id,
        tree::{id_tree::IdTree, tree_node_path::{TraversableTree, TreeNodePath, TREE_TRAVERSE_KEYS}},
    },
};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplaySize},
    ui_element::{CharCell, CharGrid, UIElement},
    ui_event::{KeyModifiers, KeyTrait, UIEvent},
    UIDisplay,
};
use std::{
    io::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};
use tabs::Tabs;

mod tabs;

#[derive(Debug, Clone)]
enum Mode {
    /// Focused on some app
    Normal,
    /// If ChoosingFocus, there should be a special window app focuser
    ChoosingFocus {
        focusing_index: TreeNodePath,
        plucked: Option<IdTree<TabHandler>>,
    }
}
impl Mode {
    fn try_as_choosing_focus(&self) -> Option<(&TreeNodePath, &Option<IdTree<TabHandler>>)> {
        match self {
            Mode::Normal => None,
            Mode::ChoosingFocus { focusing_index, plucked } => Some((focusing_index, plucked)),
        }
    }

    fn try_as_choosing_focus_mut(&mut self) -> Option<(&mut TreeNodePath, &mut Option<IdTree<TabHandler>>)> {
        match self {
            Mode::Normal => None,
            Mode::ChoosingFocus { focusing_index, plucked } => Some((focusing_index, plucked)),
        }
    }

    fn try_get_focusing_index(&self) -> Option<&TreeNodePath> {
        match self {
            Mode::Normal => None,
            Mode::ChoosingFocus { focusing_index, plucked: _ } => Some(focusing_index),
        }
    }
}

pub struct ProjectManager {
    project: Project,

    tabs: Tabs,

    mode: Mode,
    is_running: Arc<AtomicBool>,

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
        let tabs = Tabs::parse_from_project(&project);

        Self {
            project,
            tabs,
            mode: Mode::Normal,
            is_running: Arc::new(AtomicBool::new(false)),
            ui_element: Arc::new(Mutex::new(UIElement::Container(Vec::new()))),
            ui_event_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn run(mut self) -> io::Result<()> {
        self.is_running.store(true, Ordering::Relaxed);

        let ui_element_clone = self.ui_element.clone();
        let ui_event_queue_clone = self.ui_event_queue.clone();
        let is_running_clone = self.is_running.clone();
        let ui_thread_handle = thread::spawn(move || {
            UIDisplay::run_display(ui_element_clone, ui_event_queue_clone, is_running_clone);
        });

        while self.is_running.load(Ordering::Relaxed) {
            self.draw_app();
            self.handle_input();
            self.process_tab_requests();
            self.answer_tab_queries();

            // FIXME: somehow prevent singularity from eating all of my CPU
            // const SLEEP_DURATION: std::time::Duration = std::time::Duration::from_millis(100);
            // thread::sleep(SLEEP_DURATION);
        }

        ui_thread_handle.join().unwrap();

        self.save_to_file();

        Ok(())
    }

    fn render_tile_recursive(
        &mut self,
        tile_id: Id<Tile>,
        container_area: DisplayArea,
    ) -> UIElement {
        let tile = *self.tabs.get_display_tiles().get_tile(tile_id).unwrap();

        match tile {
            Tile::Container {
                children,
                orientation,
                split,
            } => {
                let area_splits = match orientation {
                    singularity_common::tab::tile::Orientation::Horizontal => [
                        DisplayArea::new((0., 0.), (1., split)),
                        DisplayArea::new((0., split), (1., 1.)),
                    ],
                    singularity_common::tab::tile::Orientation::Vertical => [
                        DisplayArea::new((0., 0.), (split, 1.)),
                        DisplayArea::new((split, 0.), (1., 1.)),
                    ],
                };

                UIElement::Container(vec![
                    self.render_tile_recursive(
                        children[0],
                        area_splits[0].map_onto(container_area),
                    ),
                    self.render_tile_recursive(
                        children[1],
                        area_splits[1].map_onto(container_area),
                    ),
                ])
            }
            Tile::Tab { tab_id } => {
                let tab = self.tabs.get_mut_tab_handler(tab_id).unwrap();

                // NOTE: rn, this is how the tab area is updated, but there's gotta be a better way
                tab.set_area(container_area);

                tab.get_ui_element().contain(container_area)
            }
        }
    }

    fn draw_app(&mut self) {
        let mut tab_elements = Vec::new();

        // for tab_id in self.tabs.get_display_order().clone() {
        //     let tab = &mut self.tabs.get_mut_tab_handler(tab_id).unwrap();

        //     tab_elements.push(tab.get_ui_element().contain(tab.get_area()));
        // }
        tab_elements.push(self.render_tile_recursive(
            self.tabs.get_display_tiles().get_root_tile(),
            DisplayArea::FULL,
        ));

        // display the tab focuser/selector
        if let Mode::ChoosingFocus { focusing_index, plucked } = &self.mode {
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
                    .contain(DisplayArea::new((0.4, 0.4), (0.6, 0.6)))
            );

            if let Some(plucked) = plucked {
                let mut plucked_display = CharGrid::default();
    
                for tab_path in plucked.iter_paths_dfs() {
                    let tab_id = plucked.get_id_from_path(&tab_path).unwrap();
                    let tab = self.tabs.get_tab_handler(tab_id).unwrap();
    
                    let fg = Color::LIGHT_GREEN;
    
                    let bg = Color::TRANSPARENT;
    
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
    
                    plucked_display.content.push(subapp_title_display);
                }
    
                tab_elements.push(
                    UIElement::CharGrid(plucked_display)
                        .fill_bg(Color::DARK_GRAY)
                        .bordered(Color::LIGHT_GREEN)
                        .contain(DisplayArea::new((0.5, 0.4), (0.6, 0.6))),
                );
            }
        }

        *(self.ui_element.lock().unwrap()) =
            UIElement::Container(tab_elements).fill_bg(Color::BLACK);
    }

    fn save_to_file(mut self) {
        // save the tabs session
        let open_tabs = self.tabs.save_session();
        self.project.project_settings.open_tabs = Some(open_tabs);
        self.project.save_to_file();
    }

    fn handle_input(&mut self) {
        for ui_event in std::mem::take(&mut *(self.ui_event_queue.lock().unwrap())) {
            use singularity_ui::ui_event::UIEvent;
            match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.raw_code == 16 => {
                    // Ctrl+Q
                    dbg!("Goodbye!");
                    self.is_running.store(false, Ordering::Relaxed);
                    return;
                }
                UIEvent::KeyPress(key, KeyModifiers::ALT)
                    if key.to_char() == Some('\n')
                        // `' '` is a placeholder for some key that isn't in tree traverse
                        || TREE_TRAVERSE_KEYS.contains(&key.to_char().unwrap_or(' ')) =>
                {
                    // Alt + arrows should be like alt tab for Windows and Linux but tree based
                    // Alt + Enter either opens the tab chooser or closes it and chooses the tab

                    if key.to_char() == Some('\n') && self.mode.try_as_choosing_focus().is_some() {
                        // place if needed, save tree index, and close window

                        let (new_focus_index, pluck) = self.mode.try_as_choosing_focus_mut().unwrap();

                        if let Some(pluck) = pluck.take() {
                            self.tabs.org_place(pluck, self.tabs.get_id_by_org_path(new_focus_index).unwrap());
                        }

                        self.tabs.set_focused_tab_path(new_focus_index);

                        self.mode = Mode::Normal;
                    } else {
                        let (new_focus_index, plucked) = match &self.mode {
                            Mode::Normal => {
                                (&self.tabs
                                    .get_tab_path(&self.tabs.get_focused_tab_id())
                                    .unwrap(), &None)
                            },
                            Mode::ChoosingFocus { focusing_index, plucked } => (focusing_index, plucked),
                        };

                        self.mode = Mode::ChoosingFocus { 
                            focusing_index: match key.to_char() {
                                Some('\n') => new_focus_index.clone(),
                                Some(traverse_key) if TREE_TRAVERSE_KEYS.contains(&traverse_key) => {
                                    new_focus_index
                                        .clamped_traverse_based_on_wasd(&self.tabs, traverse_key)
                                }
                                _ => panic!(),
                            },
                            plucked: plucked.clone(),
                        };
                    }
                    dbg!(&self.mode);
                    // dbg!(&self.focused_tab_path);
                }
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: false,
                        caps_lock: false,
                        logo: true,
                        num_lock: false,
                    },
                ) if // `' '` is a placeholder for some key that isn't in tree traverse
                TREE_TRAVERSE_KEYS.contains(&key.to_char().unwrap_or(' ')) =>
                {
                    // Alt + Windows + traversal key swaps position of focused and what would be the new focused
                    
                    let (prev_focus_index, plucked) = match &self.mode {
                        Mode::Normal => (&self.tabs
                            .get_tab_path(&self.tabs.get_focused_tab_id())
                            .unwrap(), &None),
                        Mode::ChoosingFocus { focusing_index, plucked } => (focusing_index, plucked),
                    };
                    
                    let new_focus_index = prev_focus_index.clamped_traverse_based_on_wasd(&self.tabs, key.to_char().unwrap());
                    
                    self.tabs.org_swap([self.tabs.get_id_by_org_path(prev_focus_index).unwrap(), self.tabs.get_id_by_org_path(&new_focus_index).unwrap()]);
                    
                    // self.tabs.set_focused_tab_path(&new_focus_index);
                    self.mode = Mode::ChoosingFocus{ focusing_index: new_focus_index, plucked: plucked.clone() };
                    
                    dbg!(&self.mode);
                }
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: true,
                        caps_lock: false,
                        logo: false,
                        num_lock: false,
                    },
                ) if // `' '` is a placeholder for some key that isn't in tree traverse
                    key.to_char()==Some('P') =>
                {
                    // Alt + Windows + P does pluck/place

                    let (focusing_index, plucked) = match self.mode {
                        Mode::Normal => {
                            (&self.tabs
                                .get_tab_path(&self.tabs.get_focused_tab_id())
                                .unwrap(), None)
                        },
                        Mode::ChoosingFocus { ref focusing_index, ref mut plucked } => (focusing_index, plucked.take()),
                    };

                    if let Some(plucked) = plucked {
                        // place
                        self.tabs.org_place(plucked, self.tabs.get_id_by_org_path(focusing_index).unwrap());
                    } else {
                        // pluck
                        if !focusing_index.is_root() {
                            self.mode = Mode::ChoosingFocus {
                                focusing_index: focusing_index.traverse_to_parent().unwrap(),
                                plucked: self.tabs.org_pluck(&self.tabs.get_id_by_org_path(focusing_index).unwrap())
                            };
                        }
                    }
                }
                UIEvent::KeyPress(
                    key,
                    KeyModifiers {
                        ctrl: false,
                        alt: true,
                        shift: true,
                        caps_lock: false,
                        logo: false,
                        num_lock: false,
                    },
                ) if // `' '` is a placeholder for some key that isn't in tree traverse
                key.to_char()==Some('S') =>
                {
                    // Alt + Shift + S swaps actually focused and focusing
                    
                    if let Some(focuser_path) = self.mode.try_get_focusing_index().cloned() {
                        let focusing = self.tabs.get_id_by_org_path(&focuser_path).unwrap();
                        let actually_focused = self.tabs.get_focused_tab_id();

                        self.tabs.org_swap([focusing, actually_focused]);

                        // self.app_focuser_index = Some(todo!());
                    }
                }
                // UIEvent::KeyPress(key, KeyModifiers::ALT) if key.raw_code == 103 => {
                //     // Alt+ArrowUp
                //     // TODO: figure out why Ctrl+Shift+ArrowUp specifically doesn't work...

                //     // maximize focused tab
                //     let focused_tab = self.tabs.get_focused_tab_mut();

                //     focused_tab.set_area(DisplayArea::from_corner_size(
                //         DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
                //         DisplaySize::new(DisplayUnits::FULL, DisplayUnits::FULL),
                //     ));
                // }
                // UIEvent::KeyPress(key, KeyModifiers::ALT) if key.raw_code == 108 => {
                //     // Alt+ArrowDown
                //     self.tabs.minimize_focused_tab();
                // }
                UIEvent::KeyPress(key, KeyModifiers::LOGO) if key.raw_code == 103 => {
                    // LOGO+ArrowUp
                }
                UIEvent::KeyPress(key, KeyModifiers::LOGO) if key.to_char() == Some('=') => {
                    // LOGO+"=" (but it represents "+")
                    // TODO: increment tile split
                }
                UIEvent::KeyPress(key, KeyModifiers::LOGO) if key.to_char() == Some('t') => {
                    // "T"ranspose selected tile's container (change horizontal vs vertical)
                    self.tabs.transpose_focused_tile_parent();
                }
                UIEvent::KeyPress(key, KeyModifiers::LOGO) if key.to_char() == Some('s') => {
                    // "S"wap selected tile's siblings
                    self.tabs.swap_focused_tile_siblings();
                }
                UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.to_char() == Some('w') => {
                    println!("Deletin");
                    self.tabs.close_focused_tab_recursively();
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
                        if focused_tab.get_area().map_onto(container).contains(
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
                    for tab_id in self.tabs.collect_tab_ids().iter().rev() {
                        let tab = self.tabs.get_tab_handler(*tab_id).unwrap();
                        let tab_area = tab.get_area();

                        if tab_area.map_onto(container).contains(
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
                    Request::SpawnChildTab(tab_creator, tab_data) => {
                        self.tabs.add(
                            TabHandler::new(
                                tab_creator,
                                tab_data,
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
            inquieror.get_respond_channels().answer_query(
                move || tab_path.clone(),
                move || inquieror.tab_name.clone(),
                move || inquieror.get_tab_data().clone(),
            );
        }
    }

    /// TODO: now, with tiling, I don't need this
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
