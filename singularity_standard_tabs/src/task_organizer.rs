use serde::{Deserialize, Serialize};
use singularity_common::{
    ask_query,
    components::{button::ToggleButton, text_box::TextBox, timer_widget::TimerWidget, Component},
    tab::packets::Event,
    utils::{
        timer::Timer,
        tree::{
            recursive_tree::RecursiveTreeNode,
            tree_node_path::{TreeNodePath, TREE_TRAVERSE_KEYS},
        },
    },
};
use singularity_macros::ComposeComponents;
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord},
    ui_element::{CharGrid, UIElement},
    ui_event::UIEvent,
};
use std::{path::PathBuf, time::Duration};

#[derive(Serialize, Deserialize)]
pub struct IndividualTask {
    title: String,
    body: String,

    is_complete: bool,
    timer: Option<Timer>,
}
impl Default for IndividualTask {
    fn default() -> Self {
        Self {
            title: "Placeholder Title".to_string(),
            body: "Placeholder body.".to_string(),
            is_complete: false,
            // timer: None,
            timer: Some(Timer::new_clean(Duration::from_secs(30))),
        }
    }
}

#[derive(ComposeComponents)]
struct IndividualTaskWidget {
    task_path: TreeNodePath,

    #[component((DisplayArea::new((0.0, 0.0), (0.9, 0.05))), (0))]
    title: TextBox,
    #[component((DisplayArea::new((0.9, 0.0), (1.0, 0.05))), (1))]
    checkbox: ToggleButton,
    #[component((DisplayArea::new((0.0, 0.05), (1.0, 0.5))), (2))]
    body_editor: TextBox,
    #[component((DisplayArea::new((0.0, 0.5), (1.0, 1.0))), (3))]
    timer_widget: Option<TimerWidget>,

    /// this name is a keyword for ComposeComponents
    focused_component: usize,
}
impl IndividualTaskWidget {
    fn new(task: &IndividualTask, task_path: TreeNodePath) -> Self {
        Self {
            task_path,
            title: TextBox::from(task.title.clone()),
            checkbox: ToggleButton::new(
                UIElement::CharGrid(CharGrid::from("DONE".to_string()))
                    .bordered(Color::LIGHT_GREEN),
                UIElement::CharGrid(CharGrid::from("TODO".to_string()))
                    .bordered(Color::LIGHT_GREEN),
                task.is_complete,
            ),
            body_editor: TextBox::from(task.body.clone()),
            timer_widget: task
                .timer
                .as_ref()
                .map(|timer| TimerWidget::new(*timer, false)),
            focused_component: 1,
        }
    }

    fn save_into(&self, tasks: &mut RecursiveTreeNode<IndividualTask>) {
        tasks[&self.task_path].title = self.title.get_text_as_string();
        tasks[&self.task_path].body = self.body_editor.get_text_as_string();
        tasks[&self.task_path].is_complete = self.checkbox.toggle;

        if let Some(timer_widget) = &self.timer_widget {
            tasks[&self.task_path].timer = Some(*timer_widget.get_timer());
        }
    }
}
impl Component for IndividualTaskWidget {
    /// TODO: add `focused` argument
    fn render(&mut self) -> UIElement {
        self.render_components()
            .fill_bg(Color::DARK_GRAY)
            .bordered(Color::LIGHT_GREEN)
    }

    fn handle_event(&mut self, event: singularity_common::tab::packets::Event) {
        use singularity_common::tab::packets::Event;
        use singularity_ui::ui_event::{KeyModifiers, UIEvent};
        match event {
            Event::UIEvent(ref ui_event) => match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::NONE) if key.raw_code == 15 => {
                    // TAB pressed, shift focus
                    self.focused_component += 1;
                    self.focused_component %= 3;
                }
                UIEvent::KeyPress(key, KeyModifiers::SHIFT) if key.raw_code == 15 => {}
                // UIEvent::MousePress(..) => {
                //     if let Some(timer_widget) = &mut self.timer_widget {
                //         timer_widget.handle_event(event);
                //         dbg!("mouse pressed on individual task");
                //     }
                // }
                _ => {
                    // forward to focused component
                    if let Err(Some(clicked_component_index)) =
                        self.forward_events_to_focused(event.clone())
                    {
                        // if mousclicked on another component, then change focus and re-forward
                        self.focused_component = clicked_component_index;
                        self.forward_events_to_focused(event).unwrap();
                    }
                }
            },
            Event::Focused => {}
            Event::Unfocused => {}
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}

#[derive(Debug)]
enum Mode {
    /// also traversing
    Viewing,
    /// more accurately, this mode just means the "focus" should be on an individual task
    Editing,
}

#[derive(ComposeComponents)]
#[focused_component((self.mode), (Mode))]
pub struct TaskOrganizer {
    task_file_path: PathBuf,
    /// REVIEW: rooted tree or recursive tree?
    /// NOTE: making this a vec of trees allows it to be empty.
    /// You can almost think of the task organizer as being the root
    /// and the tasks are the children of the larger tree.
    /// However, the fact that this isn't the standard tree means
    /// I need to implement special cases, which is very annoying
    /// and also means that for any change I make to trees,
    /// I might need to make it here as well.
    /// So, I wonder if I should simply make this a single task
    /// and I can maybe just ignore the root task or pretend like
    /// mandating a root task is a feature not a bug.
    /// REVIEW: what I said above ^
    #[tree_component((Self::generate_tree_area(__index, __path)), (self.render_task_list_item(__path)), (self.handle_item_event(__path, __event)), (Mode::Viewing))]
    tasks: RecursiveTreeNode<IndividualTask>,

    #[component((DisplayArea::new((0.5, 0.0), (1.0, 1.0))), (Mode::Editing))]
    focused_task_widget: Option<IndividualTaskWidget>,

    /// If editing mode, there should be Some focused task
    mode: Mode,
}
impl TaskOrganizer {
    pub fn new_from_project<P>(
        project_path: P,
        manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let mut task_file_path: PathBuf = project_path.into();
        task_file_path.push(".project");
        task_file_path.push("tasks.json");

        Self::new::<PathBuf>(task_file_path, manager_handler)
    }

    pub fn new<P>(
        task_file_path: P,
        manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let tasks =
            serde_json::from_str(&std::fs::read_to_string(&task_file_path).unwrap()).unwrap();

        manager_handler.send_request(singularity_common::tab::packets::Request::ChangeName(
            "Task Organizer".to_string(),
        ));

        Self {
            task_file_path: PathBuf::from(task_file_path),
            tasks,
            focused_task_widget: None,
            mode: Mode::Viewing,
        }
    }

    fn set_focused_task(&mut self, task_path: &TreeNodePath) {
        self.focused_task_widget = Some(IndividualTaskWidget::new(
            &self.tasks[task_path],
            task_path.clone(),
        ));
    }

    fn generate_tree_area(index: usize, path: &TreeNodePath) -> DisplayArea {
        DisplayArea::from_corner_size(
            DisplayCoord::new(
                (path.depth() as i32 * 6 * 4).into(),
                (index as i32 * 12).into(),
            ),
            singularity_ui::display_units::DisplaySize::new((12 * 40).into(), 12.into()),
        )
    }

    fn set_mode(&mut self, new_mode: Mode) {
        match new_mode {
            Mode::Viewing => {
                if let Some(focused_task_widget) = &self.focused_task_widget {
                    // save before switching mode
                    focused_task_widget.save_into(&mut self.tasks);
                }

                self.mode = Mode::Viewing;
            }
            Mode::Editing => {
                if self.focused_task_widget.is_none() {
                    // edit mode requires Some focused task so set one if n/a:
                    // user tried to traverse for the first time, select root task
                    self.set_focused_task(&TreeNodePath::new_root());
                }

                self.mode = Mode::Editing;
            }
        }
    }

    fn render_task_list_item(&mut self, path: &TreeNodePath) -> UIElement {
        let fg = if self.tasks[path].is_complete {
            Color::LIGHT_GREEN
        } else {
            Color::RED
        };
        let bg = if let Some(IndividualTaskWidget { task_path, .. }) = &self.focused_task_widget {
            if task_path == path {
                Color::CYAN
            } else {
                Color::TRANSPARENT
            }
        } else {
            Color::TRANSPARENT
        };

        UIElement::CharGrid(CharGrid {
            content: vec![self.tasks[path]
                .title
                .chars()
                .map(|c| singularity_ui::ui_element::CharCell {
                    character: c,
                    fg,
                    bg,
                })
                .collect()],
        })
    }

    fn handle_item_event(&mut self, path: &TreeNodePath, event: Event) {
        if let Event::UIEvent(UIEvent::MousePress(_, _)) = event {
            self.set_focused_task(path);
        }
    }
}
impl singularity_common::tab::BasicTab for TaskOrganizer {
    fn initialize_tab(manager_handler: &singularity_common::tab::ManagerHandler) -> Self {
        Self::new_from_project(
            serde_json::from_value::<String>(
                ask_query!(manager_handler.get_query_channels(), TabData).session_data,
            )
            .unwrap(),
            manager_handler,
        )
    }

    fn render_tab(
        &mut self,
        _manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Option<UIElement> {
        // let mut elements = Vec::new();

        // // draw task list
        // {
        //     let mut task_list_vec = Vec::new();
        //     for path in self.tasks.iter_paths_dfs() {
        //         // TODO: style complete vs todo
        //         let bg_color = if let Some(EnclosedComponent {
        //             inner_component: IndividualTaskWidget { task_path, .. },
        //             ..
        //         }) = &self.focused_task_widget
        //         {
        //             if task_path == &path {
        //                 Color::CYAN
        //             } else {
        //                 Color::TRANSPARENT
        //             }
        //         } else {
        //             Color::TRANSPARENT
        //         };

        //         let line = " ".repeat(2 * path.depth()) + &self.tasks[&path].title;

        //         task_list_vec.push(
        //             line.chars()
        //                 .map(|c| singularity_ui::ui_element::CharCell {
        //                     character: c,
        //                     fg: Color::LIGHT_YELLOW,
        //                     bg: bg_color,
        //                 })
        //                 .collect(),
        //         );
        //     }
        //     elements.push(
        //         UIElement::CharGrid(CharGrid {
        //             content: (task_list_vec),
        //         })
        //         .contain(DisplayArea(
        //             DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
        //             DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::FULL),
        //         )),
        //     );
        // }

        // // draw focused task
        // if let Some(focused_task_widget) = &mut self.focused_task_widget {
        //     // task body text
        //     elements.push(focused_task_widget.render().bordered(Color::LIGHT_GREEN));
        // }

        // Some(
        //     UIElement::Container(elements)
        //         .fill_bg(Color::DARK_GRAY)
        //         .bordered(Color::LIGHT_GREEN),
        // )
        Some(self.render_components().bordered(Color::LIGHT_GREEN))
    }

    fn handle_tab_event(
        &mut self,
        event: singularity_common::tab::packets::Event,
        _manager_handler: &singularity_common::tab::ManagerHandler,
    ) {
        use singularity_common::tab::packets::Event;
        use singularity_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};

        match self.mode {
            Mode::Viewing => match event {
                Event::UIEvent(ui_event) => match ui_event {
                    UIEvent::KeyPress(key, KeyModifiers::SHIFT) if key.to_char() == Some('+') => {
                        // add a placeholder root task & focus on it

                        let prev_focused_path = self.focused_task_widget.as_ref().map_or(
                            TreeNodePath::new_root(),
                            |focused_task_widget| focused_task_widget.task_path.clone());

                        self.tasks.safe_get_mut(&prev_focused_path).unwrap().push_child_node(
                                RecursiveTreeNode::from_value(IndividualTask::default()),
                        );

                        self.set_focused_task(&prev_focused_path.traverse_to_last_child(&self.tasks).unwrap());
                    }
                    UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.to_char() == Some('s') => {
                        // save body
                        if let Some(focused_task) = &self.focused_task_widget {
                            focused_task.save_into(&mut self.tasks);
                        }

                        // save to file
                        std::fs::write(
                            &self.task_file_path,
                            serde_json::to_string_pretty(&self.tasks).unwrap(),
                        )
                        .unwrap();
                    }
                    UIEvent::KeyPress(key, KeyModifiers::NONE) if key.to_char() == Some('\n') => {
                        // NOTE: Enter+CONTROL doesn't work as an event for some reason
                        // enter edit mode

                        self.set_mode(Mode::Editing);
                    }
                    UIEvent::KeyPress(traverse_key, KeyModifiers::NONE)
                    // `' '` is a placeholder for some key that isn't in tree traverse
                    if TREE_TRAVERSE_KEYS.contains(&traverse_key.to_char().unwrap_or(' ')) =>
                    {
                        if let Some(IndividualTaskWidget { task_path, .. },) = &self.focused_task_widget
                        {
                            self.set_focused_task(&task_path.clamped_traverse_based_on_wasd(
                                &self.tasks,
                                traverse_key.to_char().unwrap(),
                            ));
                        } else {
                            self.set_focused_task(&TreeNodePath::new_root());
                        }
                    }
                    UIEvent::MousePress(..) => {
                        let forward_result = self.forward_events_to_focused(Event::UIEvent(ui_event.clone()));

                        // even if focused_task_widget is none, forward events just checks if mouseclick is within area
                        // FIXME fix ^
                        if self.focused_task_widget.is_some() {
                            if let Err(Some(Mode::Editing)) = forward_result {
                                // index 1 should be the focused task widget
                                self.set_mode(Mode::Editing);

                                // re-forward the event
                                self.forward_events_to_focused(Event::UIEvent(ui_event)).unwrap();
                            }
                        }
                    }
                    _ => {}
                },
                Event::Focused => {}
                Event::Unfocused => {}
                Event::Resize(_) => {}
                Event::Close => panic!("Event::Close should not have been forwarded"),
            },
            Mode::Editing => match &event {
                Event::UIEvent(UIEvent::KeyPress(key, KeyModifiers::CTRL))
                    if key.to_char() == Some('s') =>
                {
                    // save body
                    if let Some(focused_task) = &self.focused_task_widget {
                        focused_task.save_into(&mut self.tasks);
                    }

                    // save to file
                    std::fs::write(
                        &self.task_file_path,
                        serde_json::to_string_pretty(&self.tasks).unwrap(),
                    )
                    .unwrap();
                }
                Event::UIEvent(UIEvent::KeyPress(key, KeyModifiers::NONE)) if key.raw_code == 1 => {
                    // ESCAPE KEY, switch to viewing mode

                    self.set_mode(Mode::Viewing);
                }
                Event::Resize(_) => {}
                Event::Close => panic!("Event::Close should not have been forwarded"),
                _ => {
                    if self.forward_events_to_focused(event.clone()).is_err() {
                        // clicked off of focus, either on tree or just on nothing
                        self.set_mode(Mode::Viewing);

                        self.forward_events_to_focused(event).unwrap();
                    }
                    // if let Some(task_widget) = &mut self.focused_task_widget {
                    //     task_widget.handle_event(event);
                    // }
                }
            },
        }
    }
}
