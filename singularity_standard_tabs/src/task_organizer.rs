use serde::{Deserialize, Serialize};
use singularity_common::{
    components::text_box::TextBox,
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplayUnits},
    ui_element::{CharGrid, UIElement},
};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct IndividualTask {
    title: String,
    body: String,

    is_complete: bool,
}
impl Default for IndividualTask {
    fn default() -> Self {
        Self {
            title: "Placeholder Title".to_string(),
            body: "Placeholder body.".to_string(),
            is_complete: false,
        }
    }
}

enum Mode {
    /// also traversing
    Viewing,
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

    /// (task path, body editor)
    focused_task: Option<(TreeNodePath, TextBox)>,
    /// If editing mode, there should be some focused task
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
            focused_task: None,
            mode: Mode::Viewing,
        }
    }

    fn set_focused_task(&mut self, task_path: TreeNodePath) {
        self.focused_task = Some((
            task_path.clone(),
            TextBox::from(self.tasks[&task_path].body.clone()),
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
                let bg_color = if let Some((focused_path, _)) = &self.focused_task {
                    if focused_path == &path {
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
        if let Some((focused_path, body_text_box)) = &mut self.focused_task {
            let focused_task = &self.tasks[focused_path];

            // title
            elements.push(
                UIElement::CharGrid(CharGrid::from(focused_task.title.clone())).contain(
                    DisplayArea(
                        DisplayCoord::new(DisplayUnits::HALF, DisplayUnits::ZERO),
                        DisplayCoord::new(DisplayUnits::FULL, 0.05.into()),
                    ),
                ),
            );

            // task body text
            elements.push(
                UIElement::CharGrid(body_text_box.render())
                    .bordered(Color::LIGHT_GREEN)
                    .contain(DisplayArea(
                        DisplayCoord::new(DisplayUnits::HALF, 0.05.into()),
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
        let (key, modifiers) =
            if let Event::UIEvent(UIEvent::KeyPress(key, modifiers)) = event.clone() {
                (key, modifiers)
            } else {
                // right now, no use for any other event type
                return;
            };

        match self.mode {
            Mode::Viewing => match (key.to_char(), modifiers) {
                (Some('+'), KeyModifiers::SHIFT) => {
                    // add a placeholder root task & focus on it

                    self.tasks
                        .push_child_node(RecursiveTreeNode::from_value(IndividualTask::default()));

                    self.set_focused_task(TreeNodePath::new_root());
                }
                (Some('s'), KeyModifiers::CTRL) => {
                    // save body
                    if let Some((focused_path, body_text_box)) = &self.focused_task {
                        let focused_task = &mut self.tasks[focused_path];

                        focused_task.body = body_text_box.get_text_as_string();
                    }

                    // save to file
                    std::fs::write(
                        &self.task_file_path,
                        serde_json::to_string_pretty(&self.tasks).unwrap(),
                    )
                    .unwrap();
                }
                (Some('\n'), KeyModifiers::NONE) => {
                    // NOTE: Enter+CONTROL doesn't work as an event for some reason
                    // enter edit mode

                    if self.focused_task.is_none() {
                        // edit mode requires Some focused task so set one if n/a

                        // user tried to traverse for the first time, select first task
                        self.set_focused_task(TreeNodePath::new_root());
                    }

                    self.mode = Mode::Editing;
                }
                (Some(traverse_key), KeyModifiers::NONE)
                    if matches!(traverse_key, 'w' | 'a' | 's' | 'd') =>
                {
                    if let Some((prev_focused, ..)) = &self.focused_task {
                        self.set_focused_task(
                            prev_focused.clamped_traverse_based_on_wasd(&self.tasks, traverse_key),
                        );
                    } else {
                        self.set_focused_task(TreeNodePath::new_root());
                    }
                }
                _ => {}
            },
            Mode::Editing => match (key, modifiers) {
                (key, KeyModifiers::NONE) if key.raw_code == 1 => {
                    // ESCAPE KEY, switch to viewing mode
                    let (task_path, text_box) = self.focused_task.as_ref().unwrap();
                    self.tasks[task_path].body = text_box.get_text_as_string();

                    self.mode = Mode::Viewing;
                }
                _ => {
                    if let Some((_focused_path, body_text_box)) = &mut self.focused_task {
                        body_text_box.handle_event(event);
                    }
                }
            },
        }
    }
}
