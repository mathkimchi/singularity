use super::SubappUI;
use crate::manager::ManagerProxy;
use ratatui::{crossterm::event::Event, layout::Rect, widgets::Widget};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    title: String,
    body: String,

    is_complete: bool,

    subtasks: Vec<Task>,
}

pub struct TaskOrganizer {
    task_file_path: PathBuf,
    tasks: Vec<Task>,
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
        manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    ) {
        ratatui::widgets::Block::bordered()
            .title("Tasks")
            .render(area, display_buffer);

        dbg!(&self.tasks);
    }

    fn handle_input(&mut self, manager_proxy: &mut ManagerProxy, event: Event) {}
}
