use crate::editor::Editor;
use singularity_common::{
    tab::{
        basic_tab_creator,
        packets::{Event, Request},
        ManagerHandler,
    },
    utils::tree::{
        rooted_tree::RootedTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use singularity_ui::DisplayBuffer;
use std::path::PathBuf;

pub struct FileManager {
    directory_tree: RootedTree<PathBuf>,
    selected_path: TreeNodePath,
}
impl FileManager {
    pub fn new<P>(root_directory_path: P, manager_handler: &ManagerHandler) -> Self
    where
        PathBuf: std::convert::From<P>,
    {
        let file_manager = Self {
            directory_tree: Self::generate_directory_tree(PathBuf::from(root_directory_path)),
            selected_path: TreeNodePath::new_root(),
        };

        manager_handler.send_request(Request::ChangeName(file_manager.get_directory_name()));

        file_manager
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

    fn get_directory_name(&self) -> String {
        self.directory_tree[&TreeNodePath::new_root()]
            .file_name() // this function can return directory name
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    pub fn render(&mut self, manager_handler: &ManagerHandler) -> Option<DisplayBuffer> {
        // use ratatui::{
        //     buffer::Buffer,
        //     style::{Style, Stylize},
        // };

        // let mut ratatui_buffer = Buffer::empty(manager_handler.inner_area);
        // // TODO
        // let is_focused = true;

        // for (index, tree_node_path) in self.directory_tree.iter_paths_dfs().enumerate() {
        //     let mut line_style = Style::new();

        //     if tree_node_path == self.selected_path {
        //         line_style = line_style.on_cyan();

        //         if is_focused {
        //             line_style = line_style.light_yellow().bold();
        //         }
        //     }

        //     ratatui_buffer.set_stringn(
        //         manager_handler.inner_area.x + 2 * tree_node_path.depth() as u16,
        //         manager_handler.inner_area.y + index as u16,
        //         self.directory_tree[&tree_node_path]
        //             .file_name() // this function can return directory name
        //             .unwrap()
        //             .to_str()
        //             .unwrap(),
        //         (manager_handler.inner_area.width - 2) as usize,
        //         line_style,
        //     );
        // }

        // Some(ratatui_buffer.content)

        todo!()
    }

    pub fn handle_event(&mut self, event: Event, manager_handler: &ManagerHandler) {
        // use ratatui::crossterm::event::{
        //     Event as TUIEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
        // };

        // match event {
        //     Event::TUIEvent(tui_event) => match tui_event {
        //         TUIEvent::Key(KeyEvent {
        //             modifiers: KeyModifiers::CONTROL,
        //             code: KeyCode::Char(traverse_key),
        //             kind: KeyEventKind::Press,
        //             ..
        //         }) if matches!(traverse_key, 'w' | 'a' | 's' | 'd') => {
        //             self.selected_path = self
        //                 .selected_path
        //                 .clamped_traverse_based_on_wasd(&self.directory_tree, traverse_key);
        //         }
        //         TUIEvent::Key(KeyEvent {
        //             modifiers: KeyModifiers::CONTROL,
        //             code: KeyCode::Char('f'),
        //             kind: KeyEventKind::Press,
        //             ..
        //         }) => {
        //             // `f` stands for open selected *F*ile

        //             let selected_element = &self.directory_tree[&self.selected_path];
        //             if selected_element.is_file() {
        //                 manager_handler.send_request(Request::SpawnChildTab(Box::new(
        //                     // Editor::new(selected_element),
        //                     basic_tab_creator(
        //                         selected_element.clone(),
        //                         Editor::new,
        //                         Editor::render,
        //                         Editor::handle_event,
        //                     ),
        //                 )));
        //             }
        //             // if selected path isn't a file, then don't do anything
        //         }
        //         _ => {}
        //     },
        //     Event::Resize(_) => {}
        //     Event::Close => panic!("Event::Close should not have been forwarded"),
        // }
    }
}
// impl SubappUI for FileManager {
//     fn get_title(&self) -> String {
//         self.directory_tree[&TreeNodePath::new_root()]
//             .file_name() // this function can return directory name
//             .unwrap()
//             .to_str()
//             .unwrap()
//             .to_string()
//     }

//     fn render(
//         &mut self,
//         area: ratatui::prelude::Rect,
//         display_buffer: &mut ratatui::prelude::Buffer,
//         is_focused: bool,
//     ) {
//         for (index, tree_node_path) in self.directory_tree.iter_paths_dfs().enumerate() {
//             let mut line_style = Style::new();

//             if tree_node_path == self.selected_path {
//                 line_style = line_style.on_cyan();

//                 if is_focused {
//                     line_style = line_style.light_yellow().bold();
//                 }
//             }

//             display_buffer.set_stringn(
//                 area.x + 1 + 2 * tree_node_path.depth() as u16,
//                 area.y + 1 + index as u16,
//                 self.directory_tree[&tree_node_path]
//                     .file_name() // this function can return directory name
//                     .unwrap()
//                     .to_str()
//                     .unwrap(),
//                 (area.width - 2) as usize,
//                 line_style,
//             );
//         }

//         ratatui::widgets::Block::bordered()
//             .title(format!("{} - File Manager", self.get_title()))
//             .render(area, display_buffer);
//     }

//     fn handle_input(&mut self, event: Event) {
//         match event {
//             Event::Key(KeyEvent {
//                 modifiers: KeyModifiers::CONTROL,
//                 code: KeyCode::Char(traverse_key),
//                 kind: KeyEventKind::Press,
//                 ..
//             }) if matches!(traverse_key, 'w' | 'a' | 's' | 'd') => {
//                 self.selected_path = self
//                     .selected_path
//                     .clamped_traverse_based_on_wasd(&self.directory_tree, traverse_key);
//             }
//             Event::Key(KeyEvent {
//                 modifiers: KeyModifiers::CONTROL,
//                 code: KeyCode::Char('f'),
//                 kind: KeyEventKind::Press,
//                 ..
//             }) => {
//                 // // `f` stands for open selected *F*ile

//                 // let selected_element = &self.directory_tree[&self.selected_path];
//                 // if selected_element.is_file() {
//                 //     manager_proxy.request_spawn_child(Box::new(Editor::new(selected_element)));
//                 // }
//                 // // if selected path isn't a file, then don't do anything
//             }
//             _ => {}
//         }
//     }
// }
