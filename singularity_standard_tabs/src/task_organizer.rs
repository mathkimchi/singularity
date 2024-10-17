use serde::{Deserialize, Serialize};
use singularity_common::{
    components::{text_box::TextBox, timer_widget::TimerWidget},
    utils::{
        timer::Timer,
        tree::{
            recursive_tree::RecursiveTreeNode,
            tree_node_path::{TraversableTree, TreeNodePath},
        },
    },
};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplayUnits},
    ui_element::{CharGrid, UIElement},
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

struct IndividualTaskWidget {
    task_path: TreeNodePath,

    title: String,
    body_editor: TextBox,
    timer_widget: Option<TimerWidget>,
}
impl IndividualTaskWidget {
    fn new(task: &IndividualTask, task_path: TreeNodePath) -> Self {
        Self {
            task_path,
            title: task.title.clone(),
            body_editor: TextBox::from(task.body.clone()),
            timer_widget: task
                .timer
                .as_ref()
                .map(|timer| TimerWidget::new(*timer, false)),
        }
    }

    /// TODO: add `focused` argument
    fn render(&mut self) -> UIElement {
        let mut elements = Vec::new();

        // title
        elements.push(
            UIElement::CharGrid(CharGrid::from(self.title.clone())).contain(DisplayArea(
                DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
                DisplayCoord::new(DisplayUnits::FULL, 0.05.into()),
            )),
        );

        // task body text
        elements.push(
            UIElement::CharGrid(self.body_editor.render())
                .bordered(Color::LIGHT_GREEN)
                .contain(DisplayArea(
                    DisplayCoord::new(DisplayUnits::ZERO, 0.05.into()),
                    DisplayCoord::new(DisplayUnits::FULL, 0.5.into()),
                )),
        );

        if let Some(timer_widget) = &mut self.timer_widget {
            elements.push(timer_widget.render().contain(DisplayArea(
                DisplayCoord::new(DisplayUnits::ZERO, 0.5.into()),
                DisplayCoord::new(DisplayUnits::FULL, 1.0.into()),
            )));
        }

        UIElement::Container(elements)
            .fill_bg(Color::DARK_GRAY)
            .bordered(Color::LIGHT_GREEN)
    }

    fn handle_event(&mut self, event: singularity_common::tab::packets::Event) {
        use singularity_common::tab::packets::Event;
        use singularity_ui::ui_event::UIEvent;
        match event {
            Event::UIEvent(ref ui_event) => match ui_event {
                UIEvent::MousePress([[click_x, click_y], [tot_width, tot_height]], container) => {
                    if let Some(timer_widget) = &mut self.timer_widget {
                        let timer_area = DisplayArea(
                            DisplayCoord::new(DisplayUnits::ZERO, 0.5.into()),
                            DisplayCoord::new(DisplayUnits::FULL, 1.0.into()),
                        );

                        if timer_area.map_onto(*container).contains(
                            DisplayCoord::new((*click_x as i32).into(), (*click_y as i32).into()),
                            [*tot_width as i32, *tot_height as i32],
                        ) {
                            timer_widget.handle_event(Event::UIEvent(UIEvent::MousePress(
                                [[*click_x, *click_y], [*tot_width, *tot_height]],
                                timer_area.map_onto(*container),
                            )));
                        }
                    }
                }
                _ => {
                    // forward to body
                    self.body_editor.handle_event(event);
                }
            },
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }

    fn save_into(&self, tasks: &mut RecursiveTreeNode<IndividualTask>) {
        tasks[&self.task_path].body = self.body_editor.get_text_as_string();

        if let Some(timer_widget) = &self.timer_widget {
            tasks[&self.task_path].timer = Some(*timer_widget.get_timer());
        }
    }
}

enum Mode {
    /// also traversing
    Viewing,
    /// more accurately, this mode just means the "focus" should be on an individual task
    Editing,
}

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
    tasks: RecursiveTreeNode<IndividualTask>,

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

    fn set_focused_task(&mut self, task_path: TreeNodePath) {
        self.focused_task_widget = Some(IndividualTaskWidget::new(
            &self.tasks[&task_path],
            task_path.clone(),
        ));
    }
}
impl<P> singularity_common::tab::BasicTab<P> for TaskOrganizer
where
    P: 'static + Clone + AsRef<std::path::Path> + Send,
    PathBuf: std::convert::From<P>,
{
    fn initialize(
        init_args: &mut P,
        manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Self {
        Self::new_from_project(init_args.clone(), manager_handler)
    }

    fn render(
        &mut self,
        _manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Option<UIElement> {
        let mut elements = Vec::new();

        // draw task list
        {
            let mut task_list_vec = Vec::new();
            for path in self.tasks.iter_paths_dfs() {
                // TODO: style complete vs todo
                let bg_color = if let Some(IndividualTaskWidget { task_path, .. }) =
                    &self.focused_task_widget
                {
                    if task_path == &path {
                        Color::CYAN
                    } else {
                        Color::TRANSPARENT
                    }
                } else {
                    Color::TRANSPARENT
                };

                let line = " ".repeat(2 * path.depth()) + &self.tasks[&path].title;

                task_list_vec.push(
                    line.chars()
                        .map(|c| singularity_ui::ui_element::CharCell {
                            character: c,
                            fg: Color::LIGHT_YELLOW,
                            bg: bg_color,
                        })
                        .collect(),
                );
            }
            elements.push(
                UIElement::CharGrid(CharGrid {
                    content: (task_list_vec),
                })
                .contain(DisplayArea(
                    DisplayCoord::new(DisplayUnits::ZERO, DisplayUnits::ZERO),
                    DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::FULL),
                )),
            );
        }

        // draw focused task
        if let Some(focused_task_widget) = &mut self.focused_task_widget {
            // task body text
            elements.push(
                focused_task_widget
                    .render()
                    .bordered(Color::LIGHT_GREEN)
                    .contain(DisplayArea(
                        DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::ZERO),
                        DisplayCoord::new(DisplayUnits::FULL, DisplayUnits::FULL),
                    )),
            );
        }

        Some(
            UIElement::Container(elements)
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN),
        )
    }

    fn handle_event(
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

                        self.tasks.push_child_node(RecursiveTreeNode::from_value(
                            IndividualTask::default(),
                        ));

                        self.set_focused_task(TreeNodePath::new_root());
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

                        if self.focused_task_widget.is_none() {
                            // edit mode requires Some focused task so set one if n/a

                            // user tried to traverse for the first time, select first task
                            self.set_focused_task(TreeNodePath::new_root());
                        }

                        self.mode = Mode::Editing;
                    }
                    UIEvent::KeyPress(traverse_key, KeyModifiers::NONE)
                        if matches!(traverse_key.to_char(), Some('w' | 'a' | 's' | 'd')) =>
                    {
                        if let Some(IndividualTaskWidget { task_path, .. }) =
                            &self.focused_task_widget
                        {
                            self.set_focused_task(task_path.clamped_traverse_based_on_wasd(
                                &self.tasks,
                                traverse_key.to_char().unwrap(),
                            ));
                        } else {
                            self.set_focused_task(TreeNodePath::new_root());
                        }
                    }
                    _ => {}
                },
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

                    // save before switching mode
                    self.focused_task_widget
                        .as_ref()
                        .unwrap()
                        .save_into(&mut self.tasks);

                    self.mode = Mode::Viewing;
                }
                Event::UIEvent(UIEvent::MousePress(
                    [[click_x, click_y], [tot_width, tot_height]],
                    container,
                )) => {
                    if let Some(focused_task_widget) = &mut self.focused_task_widget {
                        let focused_task_area = DisplayArea(
                            DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::ZERO),
                            DisplayCoord::new(DisplayUnits::FULL, DisplayUnits::FULL),
                        );

                        if focused_task_area.map_onto(*container).contains(
                            DisplayCoord::new((*click_x as i32).into(), (*click_y as i32).into()),
                            [*tot_width as i32, *tot_height as i32],
                        ) {
                            focused_task_widget.handle_event(Event::UIEvent(UIEvent::MousePress(
                                [[*click_x, *click_y], [*tot_width, *tot_height]],
                                focused_task_area.map_onto(*container),
                            )));
                        }
                    }
                }
                Event::Resize(_) => {}
                Event::Close => panic!("Event::Close should not have been forwarded"),
                _ => {
                    if let Some(task_widget) = &mut self.focused_task_widget {
                        task_widget.handle_event(event);
                    }
                }
            },
        }
    }
}
