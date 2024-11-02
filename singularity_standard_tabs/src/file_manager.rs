use singularity_common::{
    ask_query,
    project::project_settings::TabData,
    tab::{
        packets::{Event, Request},
        BasicTab, ManagerHandler,
    },
    utils::tree::{
        rooted_tree::RootedTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
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
}
impl BasicTab for FileManager {
    fn initialize_tab(manager_handler: &ManagerHandler) -> Self {
        Self::new(
            serde_json::from_value::<String>(
                ask_query!(manager_handler.get_query_channels(), TabData).session_data,
            )
            .unwrap(),
            manager_handler,
        )
    }

    fn render_tab(
        &mut self,
        _manager_handler: &ManagerHandler,
    ) -> Option<singularity_ui::ui_element::UIElement> {
        use singularity_ui::{
            color::Color,
            ui_element::{CharCell, CharGrid, UIElement},
        };

        let mut lines = Vec::new();

        for tree_node_path in self.directory_tree.iter_paths_dfs() {
            let bg_color = if tree_node_path == self.selected_path {
                Color::CYAN
            } else {
                Color::TRANSPARENT
            };

            let line = " ".repeat(2 * tree_node_path.depth())
                + self.directory_tree[&tree_node_path]
                    .file_name() // this function can return directory name
                    .unwrap()
                    .to_str()
                    .unwrap();

            lines.push(
                line.chars()
                    .map(|c| CharCell {
                        character: c,
                        fg: Color::LIGHT_YELLOW,
                        bg: bg_color,
                    })
                    .collect(),
            );
        }

        Some(
            UIElement::CharGrid(CharGrid { content: lines })
                .fill_bg(Color::DARK_GRAY)
                .bordered(Color::LIGHT_GREEN),
        )
    }

    fn handle_tab_event(&mut self, event: Event, manager_handler: &ManagerHandler) {
        use singularity_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};
        match event {
            Event::UIEvent(ui_event) => match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if matches!(key.to_char(), Some('\n' | 'w' | 'a' | 's' | 'd')) =>
                {
                    self.selected_path = self.selected_path.clamped_traverse_based_on_wasd(
                        &self.directory_tree,
                        key.to_char().unwrap(),
                    );
                }
                UIEvent::KeyPress(key, KeyModifiers::NONE)
                    if matches!(key.to_char(), Some('f')) =>
                {
                    // `f` stands for open selected *F*ile

                    let selected_element = &self.directory_tree[&self.selected_path];
                    if selected_element.is_file() {
                        use crate::editor::Editor;
                        manager_handler.send_request(Request::SpawnChildTab(
                            Box::new(Editor::new_tab_creator()),
                            TabData {
                                tab_type: "EDITOR".to_string(),
                                session_data: serde_json::to_value(selected_element.clone())
                                    .unwrap(),
                            },
                        ));
                    }
                    // if selected path isn't a file, then don't do anything
                }

                _ => {}
            },
            Event::Resize(_) => {}
            Event::Close => panic!("Event::Close should not have been forwarded"),
        }
    }
}
