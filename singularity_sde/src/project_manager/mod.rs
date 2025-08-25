use crate::{
    packets::{SDEEvent, SDERequest},
    tab::TabHandler,
};
use mode::{Mode, UserAction};
use singularity_common::utils::{
    id_map::Id,
    tree::{
        id_tree::IdTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use singularity_sap::{
    standard_packets::{
        display_packets::{
            CloseWarningEvent, DisplayEvent, DisplayRequest, NameQuery, NameResponse, PathQuery,
            PathResponse, RequestChangeName, RequestSpawnChildTab, RequestSpawnDefaultChildApplet,
            RequestUpdateWindow, SessionStorageQuery, SessionStorageResponse,
        },
        file_packets::{ReadFileQuery, ReadFileResponse, WriteFileRequest},
    },
    universal_stream::universal_server_stream::as_query_data_responder,
};
use singularity_sporg::{
    applet_data::{AppletType, AppletTypeId},
    session::Session,
    tile::{Orientation, Tile},
};
use singularity_ui::{
    UIDisplay,
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplaySize},
    ui_element::{CharCell, CharGrid, UIElement},
    ui_event::{KeyTrait, UIEvent},
};
use std::{
    fs::File,
    io::{self, Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};
use tabs::Tabs;

mod mode;
mod tabs;

pub struct ProjectManager {
    session: Session,

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
        let session = Session::get_or_make_session(project_directory.clone());
        let tabs = Tabs::parse_from_session(&session);

        Self {
            session,
            tabs,
            mode: Mode::TabFocus,
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
            self.draw_all();
            self.handle_inputs();
            self.handle_incoming();

            // FIXME: somehow prevent singularity from eating all of my CPU
            // const SLEEP_DURATION: std::time::Duration = std::time::Duration::from_millis(100);
            // thread::sleep(SLEEP_DURATION);
        }

        ui_thread_handle.join().unwrap();

        self.save_to_file();

        // close tab processes
        for mut tab in self.tabs.tabs.into_values() {
            tab.send_event(SDEEvent::DisplayEvent(DisplayEvent::Close(
                CloseWarningEvent,
            )));
            tab.kill();
        }

        Ok(())
    }

    fn render_tile_recursive(
        &mut self,
        tile_id: Id<Tile<TabHandler>>,
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
                    Orientation::Horizontal => [
                        DisplayArea::new((0., 0.), (1., split)),
                        DisplayArea::new((0., split), (1., 1.)),
                    ],
                    Orientation::Vertical => [
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

                tab.get_display().clone().contain(container_area)
            }
        }
    }

    /// display the tab focuser/selector
    fn draw_tab_selector(
        &self,
        focusing_index: &TreeNodePath,
        plucked: &Option<IdTree<TabHandler>>,
        ui_elements: &mut Vec<UIElement>,
    ) {
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

        ui_elements.push(
            UIElement::CharGrid(subapps_focuser_display)
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN)
                .contain(DisplayArea::new((0.4, 0.4), (0.6, 0.6))),
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

            ui_elements.push(
                UIElement::CharGrid(plucked_display)
                    .fill_bg(Color::DARK_GRAY)
                    .bordered(Color::LIGHT_GREEN)
                    .contain(DisplayArea::new((0.5, 0.4), (0.6, 0.6))),
            );
        }
    }

    fn draw_command_palette(&self, command_buffer: String, ui_elements: &mut Vec<UIElement>) {
        ui_elements.push(
            UIElement::CharGrid(CharGrid::from(command_buffer))
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN)
                .contain(DisplayArea::new((0.3, 0.0), (0.7, 0.02))),
        );
    }

    fn draw_all(&mut self) {
        let mut ui_elements = Vec::new();

        // for tab_id in self.tabs.get_display_order().clone() {
        //     let tab = &mut self.tabs.get_mut_tab_handler(tab_id).unwrap();

        //     tab_elements.push(tab.get_ui_element().contain(tab.get_area()));
        // }
        ui_elements.push(self.render_tile_recursive(
            self.tabs.get_display_tiles().get_root_tile(),
            DisplayArea::FULL,
        ));

        // display the tab focuser/selector
        match &self.mode {
            Mode::ChoosingFocus {
                focusing_index,
                plucked,
            } => {
                self.draw_tab_selector(focusing_index, plucked, &mut ui_elements);
            }
            Mode::CommandPalette { command_buffer } => {
                self.draw_command_palette(command_buffer.clone(), &mut ui_elements);
            }
            _ => {}
        }

        *(self.ui_element.lock().unwrap()) =
            UIElement::Container(ui_elements).fill_bg(Color::BLACK);
    }

    fn save_to_file(&mut self) {
        // save the tabs session
        let open_tabs = self.tabs.save_session();
        self.session.session_data = open_tabs;
        self.session.save_to_file();
    }

    /// Returns if it was a quit. Just for that specific case.
    fn handle_user_action(&mut self, user_action: UserAction) -> bool {
        match user_action {
            UserAction::Quit => {
                // Ctrl+Q
                dbg!("Goodbye!");
                self.is_running.store(false, Ordering::Relaxed);
                return true;
            }

            UserAction::ChooseFocus => {
                // REVIEW: Pass focus and pluck into UserAction::ChooseFocus?
                // That would make things more safe conceptually. Won't have to call `try_as_choosing_focus_mut` and unwrap.
                // TODO

                let (new_focus_index, pluck) = self.mode.try_as_choosing_focus_mut().unwrap();

                if let Some(pluck) = pluck.take() {
                    self.tabs.org_place(
                        pluck,
                        self.tabs.get_id_by_org_path(new_focus_index).unwrap(),
                    );
                }

                self.tabs.set_focused_tab_path(new_focus_index);

                self.mode = Mode::TabFocus;
            }
            UserAction::OpenFocusChooser => {
                let (new_focus_index, plucked) = match &self.mode {
                    Mode::TabFocus | Mode::CommandPalette { .. } => (
                        &self
                            .tabs
                            .get_tab_path(&self.tabs.get_focused_tab_id())
                            .unwrap(),
                        &None,
                    ),
                    Mode::ChoosingFocus {
                        focusing_index,
                        plucked,
                    } => (focusing_index, plucked),
                };

                self.mode = Mode::ChoosingFocus {
                    focusing_index: new_focus_index.clone(),
                    plucked: plucked.clone(),
                };
            }
            UserAction::TraverseTabTree(traverse_operation) => {
                let (new_focus_index, plucked) = match &self.mode {
                    Mode::TabFocus | Mode::CommandPalette { .. } => (
                        &self
                            .tabs
                            .get_tab_path(&self.tabs.get_focused_tab_id())
                            .unwrap(),
                        &None,
                    ),
                    Mode::ChoosingFocus {
                        focusing_index,
                        plucked,
                    } => (focusing_index, plucked),
                };

                self.mode = Mode::ChoosingFocus {
                    focusing_index: new_focus_index
                        .clamped_traverse_on_operation(&self.tabs, traverse_operation),
                    plucked: plucked.clone(),
                };

                dbg!(&self.mode);
                // dbg!(&self.focused_tab_path);
            }
            UserAction::TreeSwapTraverse(operation) => {
                // Alt + Windows + traversal key swaps position of focused and what would be the new focused

                let (prev_focus_index, plucked) = match &self.mode {
                    Mode::TabFocus | Mode::CommandPalette { .. } => (
                        &self
                            .tabs
                            .get_tab_path(&self.tabs.get_focused_tab_id())
                            .unwrap(),
                        &None,
                    ),
                    Mode::ChoosingFocus {
                        focusing_index,
                        plucked,
                    } => (focusing_index, plucked),
                };

                let new_focus_index =
                    prev_focus_index.clamped_traverse_on_operation(&self.tabs, operation);

                self.tabs.org_swap([
                    self.tabs.get_id_by_org_path(prev_focus_index).unwrap(),
                    self.tabs.get_id_by_org_path(&new_focus_index).unwrap(),
                ]);

                // self.tabs.set_focused_tab_path(&new_focus_index);
                self.mode = Mode::ChoosingFocus {
                    focusing_index: new_focus_index,
                    plucked: plucked.clone(),
                };

                dbg!(&self.mode);
            }
            UserAction::PluckPlace => {
                // Alt + Windows + P does pluck/place

                let (focusing_index, plucked) = match self.mode {
                    Mode::TabFocus | Mode::CommandPalette { .. } => (
                        &self
                            .tabs
                            .get_tab_path(&self.tabs.get_focused_tab_id())
                            .unwrap(),
                        None,
                    ),
                    Mode::ChoosingFocus {
                        ref focusing_index,
                        ref mut plucked,
                    } => (focusing_index, plucked.take()),
                };

                if let Some(plucked) = plucked {
                    // place
                    self.tabs.org_place(
                        plucked,
                        self.tabs.get_id_by_org_path(focusing_index).unwrap(),
                    );
                } else {
                    // pluck
                    if !focusing_index.is_root() {
                        self.mode = Mode::ChoosingFocus {
                            focusing_index: focusing_index.traverse_to_parent().unwrap(),
                            plucked: self
                                .tabs
                                .org_pluck(&self.tabs.get_id_by_org_path(focusing_index).unwrap()),
                        };
                    }
                }
            }
            UserAction::TreeSwap => {
                // Alt + Shift + Enter swaps actually focused and focusing

                if let Some(focuser_path) = self.mode.try_get_focusing_index().cloned() {
                    let focusing = self.tabs.get_id_by_org_path(&focuser_path).unwrap();
                    let actually_focused = self.tabs.get_focused_tab_id();

                    self.tabs.org_swap([focusing, actually_focused]);

                    // self.app_focuser_index = Some(todo!());
                }
            }
            UserAction::OpenCommandPalette => {
                // Alt + Shift + P opens command pallette (see: https://github.com/mathkimchi/singularity/issues/11)

                self.mode = Mode::CommandPalette {
                    command_buffer: String::new(),
                };
            }
            UserAction::QuitCommandPalette => {
                // ESC quits command pallette (see: https://github.com/mathkimchi/singularity/issues/11)

                self.mode = Mode::TabFocus;
            }
            UserAction::TransposeTileParent => {
                // "T"ranspose selected tile's container (change horizontal vs vertical)
                self.tabs.transpose_focused_tile_parent();
            }
            UserAction::SwapTileSiblings => {
                // "S"wap selected tile's siblings
                self.tabs.swap_focused_tile_siblings();
            }
            UserAction::RecursivelyCloseFocusedTab => {
                println!("Deletin");
                self.tabs.close_focused_tab_recursively();
            }

            UserAction::ForwardKeyPressTab(key, key_mod) => {
                // forward the event to focused tab
                let focused_tab = self.tabs.get_focused_tab_mut();

                // rebuild the keypress. redundant but feels safer
                focused_tab.send_event(SDEEvent::DisplayEvent(DisplayEvent::UIEvent(
                    UIEvent::KeyPress(key, key_mod),
                )));
            }
            UserAction::ForwardKeyPressCommandPalette(key, _key_mod) => {
                if let Some(command_buffer) = self.mode.try_get_command_palette_buffer_mut() {
                    if let Some(key_char) = key.to_char() {
                        if key_char.is_ascii_graphic() || key_char == ' ' {
                            command_buffer.push(key_char);
                        } else if key_char == '\u{8}' {
                            // this is DELETE
                            command_buffer.pop();
                        } else if key_char == '\n' {
                            // FIXME: hideous nesting
                            let command = command_buffer.trim();
                            let command = command.split_once(' ');
                            if let Some((prefix, args)) = command {
                                match prefix {
                                    "spawn" => {
                                        let applet = AppletTypeId::new(args);
                                        dbg!("Attempting to spawn:", &applet);
                                        if let Some(AppletType {
                                            default_spawn: Some(applet_spawn_data),
                                            ..
                                        }) = self
                                            .session
                                            .project
                                            .project_settings
                                            .applet_types
                                            .get(&applet)
                                        {
                                            dbg!("Spawning:", command);
                                            self.tabs.add(
                                                TabHandler::spawn(
                                                    applet_spawn_data,
                                                    Self::generate_tab_area(
                                                        self.tabs.num_tabs(),
                                                        1,
                                                    ),
                                                ),
                                                &self.tabs.get_root_id(),
                                            );
                                        }
                                    }
                                    "dbg_print" => {
                                        dbg!("Debug print command ran!", command);
                                    }
                                    _ => {
                                        dbg!("Couldn't parse command", command);
                                    }
                                }
                            }

                            // regardless of if the command was succesful or not, just return
                            self.mode = Mode::TabFocus;
                        }
                    }
                }
            }
            UserAction::NoAction => {}
            UserAction::WindowResized => {
                // currently just ignore
                // self.ui_window_px = ui_window_px;
            }
            UserAction::MousePress([[click_x, click_y], [tot_width, tot_height]]) => {
                let container = DisplayArea::FULL;

                // if pressed on focused tab, then forward the click
                {
                    let focused_tab = self
                        .tabs
                        .get_mut_tab_handler(self.tabs.get_focused_tab_id())
                        .unwrap();
                    if focused_tab.get_area().map_onto(container).contains(
                        DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                        [tot_width as i32, tot_height as i32],
                    ) {
                        focused_tab.send_event(SDEEvent::DisplayEvent(DisplayEvent::UIEvent(
                            singularity_ui::ui_event::UIEvent::MousePress(
                                [[click_x, click_y], [tot_width, tot_height]],
                                focused_tab.get_area().map_onto(container),
                            ),
                        )));
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

        false
    }

    fn handle_inputs(&mut self) {
        let ui_events = std::mem::take(&mut *(self.ui_event_queue.lock().unwrap()));
        for ui_event in ui_events {
            let to_quit = self.handle_user_action(UserAction::from_ui_event(&self.mode, ui_event));
            if to_quit {
                return;
            }
        }
    }

    fn handle_incoming(&mut self) {
        for tab_path in self.tabs.collect_paths_dfs() {
            let sender = self
                .tabs
                .get_mut_tab_handler(self.tabs.get_id_by_org_path(&tab_path).unwrap())
                .unwrap();
            let requests = {
                let sender_name = sender.tab_name.clone();
                let session_storage = sender.applet_session_storage.clone();
                sender.handle_incoming(&mut vec![
                    &mut as_query_data_responder(|PathQuery| Some(PathResponse(tab_path.clone()))),
                    &mut as_query_data_responder(move |NameQuery| {
                        Some(NameResponse(sender_name.clone()))
                    }),
                    &mut as_query_data_responder(move |SessionStorageQuery| {
                        Some(SessionStorageResponse(session_storage.clone()))
                    }),
                    &mut as_query_data_responder(move |ReadFileQuery(path)| {
                        let mut file = File::open(path).ok()?;
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).ok()?;
                        Some(ReadFileResponse(buf))
                    }),
                ])
            };

            for request in requests {
                match request {
                    SDERequest::DisplayRequest(DisplayRequest::RequestChangeName(
                        RequestChangeName { new_name },
                    )) => {
                        self.tabs
                            .get_mut_tab_handler(self.tabs.get_id_by_org_path(&tab_path).unwrap())
                            .unwrap()
                            .tab_name = new_name;
                    }
                    SDERequest::DisplayRequest(DisplayRequest::RequestUpdateWindow(
                        RequestUpdateWindow { contents: new_ui },
                    )) => {
                        self.tabs
                            .get_mut_tab_handler(self.tabs.get_id_by_org_path(&tab_path).unwrap())
                            .unwrap()
                            .tab_display = new_ui;
                    }
                    SDERequest::DisplayRequest(DisplayRequest::RequestSpawnChildTab(
                        RequestSpawnChildTab(tab_data),
                    )) => {
                        self.tabs.add(
                            TabHandler::spawn(
                                &tab_data,
                                // NOTE: the argument child index is technically incorrect,
                                // but the purpose of the generator is to generally prevent all
                                // tabs from being spawned all in one place.
                                Self::generate_tab_area(self.tabs.num_tabs(), tab_path.depth() + 1),
                            ),
                            &self.tabs.get_id_by_org_path(&tab_path).unwrap(),
                        );
                    }
                    SDERequest::DisplayRequest(DisplayRequest::RequestSpawnDefaultChildApplet(
                        RequestSpawnDefaultChildApplet(applet_type_id),
                    )) => {
                        if let Some(applet) = self
                            .session
                            .project
                            .project_settings
                            .applet_types
                            .get(&applet_type_id)
                        {
                            if let Some(default_spawn) = &applet.default_spawn {
                                self.tabs.add(
                                    TabHandler::spawn(
                                        default_spawn,
                                        // NOTE: the argument child index is technically incorrect,
                                        // but the purpose of the generator is to generally prevent all
                                        // tabs from being spawned all in one place.
                                        Self::generate_tab_area(
                                            self.tabs.num_tabs(),
                                            tab_path.depth() + 1,
                                        ),
                                    ),
                                    &self.tabs.get_id_by_org_path(&tab_path).unwrap(),
                                );
                            }
                        }
                    }
                    SDERequest::WriteFileRequest(WriteFileRequest(dest, conent_bytes)) => {
                        if let Ok(mut dest_file) = File::open(dest) {
                            dest_file.write_all(&conent_bytes).unwrap();
                        }
                    }
                }
            }
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
// impl Drop for ProjectManager {
//     fn drop(&mut self) {
//         // revert the terminal to its original state
//         // drop is called even on panic
//     }
// }
