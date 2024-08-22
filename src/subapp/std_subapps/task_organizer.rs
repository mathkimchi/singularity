use super::SubappUI;
use crate::{
    backend::utils::{
        recursive_tree::RecursiveTreeNode, rooted_tree::RootedTree, tree_node_path::TreeNodePath,
    },
    manager::ManagerProxy,
};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::Rect,
    widgets::Widget,
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

    focused_task_path: Option<(usize, TreeNodePath)>,
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
        area: Rect,
        display_buffer: &mut ratatui::prelude::Buffer,
        _manager_proxy: &mut ManagerProxy,
        _is_focused: bool,
    ) {
        ratatui::widgets::Block::bordered()
            .title("Tasks")
            .render(area, display_buffer);

        // for path in self.tasks.iter_pat
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

                self.tasks
                    .push(RecursiveTreeNode::from_value(IndividualTask::default()));

                self.focused_task_path = Some((self.tasks.len() - 1, TreeNodePath::new_root()));
                self.mode = Mode::Editing;
            }
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('s'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // save to file

                std::fs::write(
                    &self.task_file_path,
                    serde_json::to_string_pretty(&self.tasks).unwrap(),
                )
                .unwrap();
            }
            _ => {}
        }
    }
}
