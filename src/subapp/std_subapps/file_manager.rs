use super::editor::Editor;
use crate::{
    backend::utils::{RootedTree, TreeNodePath},
    manager::ManagerProxy,
    subapp::SubappUI,
};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    text::ToLine,
    widgets::Widget,
};
use std::path::PathBuf;

pub struct FileManager {
    directory_tree: RootedTree<PathBuf>,
}
impl FileManager {
    pub fn new<P>(root_directory_path: P) -> Self
    where
        PathBuf: std::convert::From<P>,
    {
        Self {
            directory_tree: Self::generate_directory_tree(root_directory_path),
        }
    }

    fn generate_directory_tree<P>(root_directory_path: P) -> RootedTree<PathBuf>
    where
        PathBuf: std::convert::From<P>,
    {
        let mut directory_tree = RootedTree::from_root(PathBuf::from(root_directory_path));

        // TODO: max depth 1 rn
        for child in directory_tree[&TreeNodePath::new_root()]
            .read_dir()
            .unwrap()
        {
            directory_tree.add_node(child.unwrap().path(), &TreeNodePath::new_root());
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
        _is_focused: bool,
    ) {
        for (index, tree_node_path) in self.directory_tree.iter_paths_dfs().enumerate() {
            display_buffer.set_line(
                area.x + 1 + 2 * tree_node_path.depth() as u16,
                area.y + 1 + index as u16,
                &self.directory_tree[&tree_node_path]
                    .file_name() // this function can return directory name
                    .unwrap()
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
