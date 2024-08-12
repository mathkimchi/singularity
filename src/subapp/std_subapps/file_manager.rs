use ratatui::{text::ToLine, widgets::Widget};

use crate::subapp::SubappUI;
use std::path::PathBuf;

pub struct FileManager {
    directory_path: PathBuf,
}
impl FileManager {
    pub fn new<P>(file_path: P) -> Self
    where
        // P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        Self {
            // temp_text_lines: Self::get_content_from_file(&file_path),
            directory_path: PathBuf::from(file_path),
        }
    }
}
impl SubappUI for FileManager {
    fn get_title(&self) -> String {
        self.directory_path
            .file_name() // this function can return directory name
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn render(
        &mut self,
        area: ratatui::prelude::Rect,
        display_buffer: &mut ratatui::prelude::Buffer,
        is_focused: bool,
    ) {
        // NOTE depth 1 for now

        display_buffer.set_line(
            area.x + 1,
            area.y + 1,
            &self
                .directory_path
                .file_name() // this function can return directory name
                .unwrap()
                .to_str()
                .unwrap()
                .to_line(),
            area.width - 2,
        );

        for (index, child) in self.directory_path.read_dir().unwrap().enumerate() {
            display_buffer.set_line(
                area.x + 1 + 1,
                area.y + 1 + index as u16 + 1,
                &child
                    .unwrap()
                    .file_name() // this function can return directory name
                    .to_str()
                    .unwrap()
                    .to_line(),
                area.width - 2,
            );
        }

        ratatui::widgets::Block::bordered()
            .title(format!("{} - File Manager", self.get_title()))
            .render(area, display_buffer);
    }

    fn handle_input(&mut self, event: ratatui::crossterm::event::Event) {}
}
