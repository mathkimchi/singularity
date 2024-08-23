use super::SubappUI;
use crate::{
    backend::utils::{
        recursive_tree::RecursiveTreeNode,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
    elements::text_box::TextBox,
    manager::ManagerProxy,
};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Widget},
};
use serde::{Deserialize, Serialize};
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
}
impl SubappUI for TaskOrganizer {
    fn get_title(&self) -> String {
        "Task Organizer".to_string()
    }

    fn render(
        &mut self,
        total_area: Rect,
        display_buffer: &mut ratatui::prelude::Buffer,
        _manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    ) {
        ratatui::widgets::Block::bordered()
            .title("Tasks")
            .render(total_area, display_buffer);

        let display_area = total_area.inner(Margin::new(1, 1));

        let (layout, spacers) =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .spacing(1)
                .split_with_spacers(display_area);

        let tasks_area = layout[0];
        let selected_task_area = layout[1];

        // Draw divider between the two areas
        Block::bordered()
            .borders(Borders::LEFT)
            .render(spacers[1], display_buffer);

        // list tasks
        for (running_count, (root_index, path)) in self
            .tasks
            .iter()
            .enumerate()
            .flat_map(|(root_index, tree)| tree.iter_paths_dfs().map(move |x| (root_index, x)))
            .enumerate()
        {
            // TODO: style complete vs todo
            let mut line_style = Style::new();

            if let Some((focused_index, focused_path, _)) = &self.focused_task_path {
                if (focused_index == &root_index) && (focused_path == &path) {
                    line_style = line_style.on_cyan();

                    if is_focused {
                        line_style = line_style.light_yellow().bold();
                    }
                }
            }

            display_buffer.set_stringn(
                tasks_area.x + 2 * path.depth() as u16,
                tasks_area.y + running_count as u16,
                &self.tasks[root_index][&path].title,
                (tasks_area.width as usize) - 2 * path.depth(),
                line_style,
            );
        }

        // draw focused task
        if let Some((focused_index, focused_path, body_text_box)) = &mut self.focused_task_path {
            let focused_task = &self.tasks[*focused_index][focused_path];

            // draw title
            display_buffer.set_stringn(
                selected_task_area.x,
                selected_task_area.y,
                &focused_task.title,
                selected_task_area.width as usize,
                Style::new().underlined(),
            );

            // draw body
            let body_area = Rect::new(
                selected_task_area.x,
                selected_task_area.y + 1,
                selected_task_area.width,
                selected_task_area.height - 1,
            );
            body_text_box.render(body_area, display_buffer, is_focused);
        }
    }

    fn handle_input(&mut self, _manager_proxy: &mut ManagerProxy, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {}
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {}
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code: KeyCode::Char('+'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // NOTE: Pressing: `SHIFT` and `=` is thought of as pressing `+`
                // on its own which makes sense, but i feel icky about it

                // add a placeholder root task & focus on it

                let placeholder_task = IndividualTask::default();
                let body_editor = TextBox::from(placeholder_task.body.clone());
                self.tasks
                    .push(RecursiveTreeNode::from_value(placeholder_task));

                self.focused_task_path =
                    Some((self.tasks.len() - 1, TreeNodePath::new_root(), body_editor));
                self.mode = Mode::Editing;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // save body
                if let Some((focused_index, focused_path, body_text_box)) = &self.focused_task_path
                {
                    let focused_task = &mut self.tasks[*focused_index][focused_path];

                    focused_task.body = body_text_box.get_text_as_string();
                }

                // save to file
                std::fs::write(
                    &self.task_file_path,
                    serde_json::to_string_pretty(&self.tasks).unwrap(),
                )
                .unwrap();
            }
            event => {
                if matches!(self.mode, Mode::Editing) {
                    if let Some((_focused_index, _focused_path, body_text_box)) =
                        &mut self.focused_task_path
                    {
                        body_text_box.handle_input(event);
                    }
                }
            }
        }
    }
}
