use super::editor::Editor;
use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    manager::ManagerProxy,
    subapp::SubappUI,
};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::{Style, Stylize},
    text::ToLine,
    widgets::Widget,
};
use std::path::PathBuf;

pub struct FileManager {
    directory_tree: RootedTree<PathBuf>,
    focused_path: TreeNodePath,
}
impl FileManager {
    pub fn new<P>(root_directory_path: P) -> Self
    where
        PathBuf: std::convert::From<P>,
    {
        Self {
            directory_tree: Self::generate_directory_tree(PathBuf::from(root_directory_path)),
            focused_path: TreeNodePath::new_root(),
        }
    }

    fn generate_directory_tree(root_directory_path: PathBuf) -> RootedTree<PathBuf> {
        let mut directory_tree = RootedTree::from_root(root_directory_path);

        // means the directory is added but its children arent
        // only directories
        let mut unvisited_directories = vec![TreeNodePath::new_root()];

        while !unvisited_directories.is_empty() {
            let mut new_unvisited_directories = Vec::new();
            for directory_path in unvisited_directories {
                for child in directory_tree[&directory_path].read_dir().unwrap() {
                    let child_path = child.unwrap().path();

                    let child_tree_path = directory_tree
                        .add_node(child_path.clone(), &directory_path)
                        .unwrap();

                    if child_path.is_dir() {
                        new_unvisited_directories.push(child_tree_path);
                    }
                }
            }

            unvisited_directories = new_unvisited_directories;
        }

        directory_tree
    }
}
impl SubappUI for FileManager {
    fn get_title(&self) -> String {
        self.directory_tree[&TreeNodePath::new_root()]
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
        _manager_proxy: &mut ManagerProxy,
        is_focused: bool,
    ) {
        for (index, tree_node_path) in self.directory_tree.iter_paths_dfs().enumerate() {
            let mut line_style = Style::new();

            if tree_node_path == self.focused_path {
                line_style = line_style.on_cyan();

                if is_focused {
                    line_style = line_style.light_yellow().bold();
                }
            }

            display_buffer.set_stringn(
                area.x + 1 + 2 * tree_node_path.depth() as u16,
                area.y + 1 + index as u16,
                self.directory_tree[&tree_node_path]
                    .file_name() // this function can return directory name
                    .unwrap()
                    .to_str()
                    .unwrap(),
                (area.width - 2) as usize,
                line_style,
            );
        }

        ratatui::widgets::Block::bordered()
            .title(format!("{} - File Manager", self.get_title()))
            .render(area, display_buffer);
    }

    fn handle_input(&mut self, manager_proxy: &mut ManagerProxy, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('t'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                // TODO: actually take care of heirarchy and stuff
                manager_proxy.request_spawn_child(Box::new(Editor::new(
                    "examples/project/file_to_edit.txt",
                )));
            }
            _ => {}
        }
    }
}
