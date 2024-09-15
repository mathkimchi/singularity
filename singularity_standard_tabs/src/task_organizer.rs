use serde::{Deserialize, Serialize};
use singularity_common::{
    elements::text_box::TextBox,
    utils::tree::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
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
    tasks: Vec<RecursiveTreeNode<IndividualTask>>,

    /// (root index, task path, body editor)
    focused_task_path: Option<(usize, TreeNodePath, TextBox)>,
    /// If editing mode, there should be some focusd task
    mode: Mode,
}
impl TaskOrganizer {
    pub fn new<P>(task_file_path: P) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let tasks =
            serde_json::from_str(&std::fs::read_to_string(&task_file_path).unwrap()).unwrap();

        Self {
            task_file_path: PathBuf::from(task_file_path),
            tasks,
            focused_task_path: None,
            mode: Mode::Viewing,
        }
    }

    fn set_focused_task(&mut self, root_index: usize, task_path: TreeNodePath) {
        self.focused_task_path = Some((
            root_index,
            task_path.clone(),
            TextBox::from(self.tasks[root_index][&task_path].body.clone()),
        ));
    }

    /// Expects traverse key to be a traverse key (wasd); panics if it isn't
    fn handle_traversal(&mut self, traverse_key: char) {
        if let Some((root_index, task_path, _text_box)) = &self.focused_task_path {
            match traverse_key {
                'a' | 'd' => {
                    // parent-child traversal works as it would normally

                    self.set_focused_task(
                        *root_index,
                        task_path
                            .clamped_traverse_based_on_wasd(&self.tasks[*root_index], traverse_key),
                    );
                }
                'w' => {
                    // sibling traversal needs to take care of the edge case of root
                    if task_path.is_root() {
                        // already root, go to previous task's root
                        // if this is very first task, then change nothing
                        self.set_focused_task(
                            root_index.saturating_sub(1),
                            TreeNodePath::new_root(),
                        );
                    } else {
                        self.set_focused_task(
                            *root_index,
                            task_path
                                .traverse_to_previous_sibling()
                                .unwrap_or(task_path.clone()),
                        );
                    }
                }
                's' => {
                    // sibling traversal needs to take care of the edge case of root
                    if task_path.is_root() {
                        // already root, go to next task's root
                        // if there is no later root, then change nothing
                        self.set_focused_task(
                            root_index.saturating_add(1).clamp(0, self.tasks.len() - 1),
                            TreeNodePath::new_root(),
                        );
                    } else {
                        self.set_focused_task(
                            *root_index,
                            task_path
                                .traverse_to_next_sibling(&self.tasks[*root_index])
                                .unwrap_or(task_path.clone()),
                        );
                    }
                }
                _ => {
                    panic!()
                }
            }
        } else {
            if self.tasks.is_empty() {
                // if user tries traversing when there are no tasks, create a placeholder task
                self.tasks
                    .push(RecursiveTreeNode::from_value(IndividualTask::default()));
            }

            // user tried to traverse for the first time, select first task
            self.set_focused_task(0, TreeNodePath::new_root());
        }
    }
}
// impl SubappUI for TaskOrganizer {
//     fn get_title(&self) -> String {
//         "Task Organizer".to_string()
//     }

//     fn render(
//         &mut self,
//         total_area: Rect,
//         display_buffer: &mut ratatui::prelude::Buffer,
//         _manager_proxy: &mut ManagerProxy,
//         is_focused: bool,
//     ) {
//         ratatui::widgets::Block::bordered()
//             .title("Tasks")
//             .render(total_area, display_buffer);

//         let display_area = total_area.inner(Margin::new(1, 1));

//         let (layout, spacers) =
//             Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
//                 .spacing(1)
//                 .split_with_spacers(display_area);

//         let tasks_area = layout[0];
//         let selected_task_area = layout[1];

//         // Draw divider between the two areas
//         Block::bordered()
//             .borders(Borders::LEFT)
//             .render(spacers[1], display_buffer);

//         // list tasks
//         for (running_count, (root_index, path)) in self
//             .tasks
//             .iter()
//             .enumerate()
//             .flat_map(|(root_index, tree)| tree.iter_paths_dfs().map(move |x| (root_index, x)))
//             .enumerate()
//         {
//             // TODO: style complete vs todo
//             let mut line_style = Style::new();

//             if let Some((focused_index, focused_path, _)) = &self.focused_task_path {
//                 if (focused_index == &root_index) && (focused_path == &path) {
//                     line_style = line_style.on_cyan();

//                     if is_focused {
//                         line_style = line_style.light_yellow().bold();
//                     }
//                 }
//             }

//             display_buffer.set_stringn(
//                 tasks_area.x + 2 * path.depth() as u16,
//                 tasks_area.y + running_count as u16,
//                 &self.tasks[root_index][&path].title,
//                 (tasks_area.width as usize) - 2 * path.depth(),
//                 line_style,
//             );
//         }

//         // draw focused task
//         if let Some((focused_index, focused_path, body_text_box)) = &mut self.focused_task_path {
//             let focused_task = &self.tasks[*focused_index][focused_path];

//             // draw title
//             display_buffer.set_stringn(
//                 selected_task_area.x,
//                 selected_task_area.y,
//                 &focused_task.title,
//                 selected_task_area.width as usize,
//                 Style::new().underlined(),
//             );

//             // draw body
//             let body_area = Rect::new(
//                 selected_task_area.x,
//                 selected_task_area.y + 1,
//                 selected_task_area.width,
//                 selected_task_area.height - 1,
//             );
//             body_text_box.render(
//                 body_area,
//                 display_buffer,
//                 is_focused && matches!(self.mode, Mode::Editing),
//             );
//         }
//     }

//     fn handle_input(&mut self, _manager_proxy: &mut ManagerProxy, event: Event) {
//         let standardized_event = if let Event::Key(KeyEvent {
//             code: key_code,
//             modifiers,
//             kind: KeyEventKind::Press,
//             ..
//         }) = event
//         {
//             (key_code, modifiers)
//         } else {
//             // right now, no use for any other event type
//             return;
//         };

//         match self.mode {
//             Mode::Viewing => match standardized_event {
//                 (KeyCode::Char('+'), KeyModifiers::NONE) => {
//                     // NOTE: Pressing: `SHIFT` and `=` is thought of as pressing `+`
//                     // on its own which makes sense, but i feel icky about it

//                     // add a placeholder root task & focus on it

//                     self.tasks
//                         .push(RecursiveTreeNode::from_value(IndividualTask::default()));

//                     self.set_focused_task(self.tasks.len() - 1, TreeNodePath::new_root());
//                 }
//                 (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
//                     // save body
//                     if let Some((focused_index, focused_path, body_text_box)) =
//                         &self.focused_task_path
//                     {
//                         let focused_task = &mut self.tasks[*focused_index][focused_path];

//                         focused_task.body = body_text_box.get_text_as_string();
//                     }

//                     // save to file
//                     std::fs::write(
//                         &self.task_file_path,
//                         serde_json::to_string_pretty(&self.tasks).unwrap(),
//                     )
//                     .unwrap();
//                 }
//                 (KeyCode::Enter, KeyModifiers::NONE) => {
//                     // NOTE: Enter+CONTROL doesn't work as an event for some reason
//                     // enter edit mode

//                     if self.focused_task_path.is_none() {
//                         // edit mode requires Some focused task so set one if n/a

//                         if self.tasks.is_empty() {
//                             // if user tries traversing when there are no tasks, create a placeholder task
//                             self.tasks
//                                 .push(RecursiveTreeNode::from_value(IndividualTask::default()));
//                         }

//                         // user tried to traverse for the first time, select first task
//                         self.set_focused_task(0, TreeNodePath::new_root());
//                     }

//                     self.mode = Mode::Editing;
//                 }
//                 (KeyCode::Char(traverse_key), KeyModifiers::NONE)
//                     if matches!(traverse_key, 'w' | 'a' | 's' | 'd') =>
//                 {
//                     self.handle_traversal(traverse_key);
//                 }
//                 _ => {}
//             },
//             Mode::Editing => match standardized_event {
//                 (KeyCode::Esc, KeyModifiers::NONE) => {
//                     self.mode = Mode::Viewing;
//                 }
//                 _ => {
//                     if let Some((_focused_index, _focused_path, body_text_box)) =
//                         &mut self.focused_task_path
//                     {
//                         body_text_box.handle_input(event);
//                     }
//                 }
//             },
//         }
//     }
// }
