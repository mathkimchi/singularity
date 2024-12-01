#![cfg(test)]

use singularity_common::{
    components::{button::Button, remap_event, text_box::TextBox, Component},
    tab::packets::Event,
    utils::tree::{
        rooted_tree::RootedTree,
        tree_node_path::{TraversableTree, TreeNodePath},
    },
};
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplaySize},
    ui_element::{CharGrid, UIElement},
};
use std::sync::Mutex;

enum Focus {
    Button1,
    Button2,
    /// Focused index, focused path
    Tree(usize, TreeNodePath),
}
pub struct Test {
    focus: Focus,
    button1: Button,
    button2: Button,

    tree: RootedTree<TextBox>,
}
impl Test {
    fn button1_area() -> DisplayArea {
        DisplayArea::new((0.0, 0.0), (0.25, 0.05))
    }
    fn button2_area() -> DisplayArea {
        DisplayArea::new((0.7, 0.0), (0.9, 0.05))
    }

    /// REVIEW: technically, index is redundant, but makes things easier
    fn generate_tree_area(index: usize, path: &TreeNodePath) -> DisplayArea {
        DisplayArea::from_corner_size(
            DisplayCoord::new(
                (path.depth() as f32 * 0.2).into(),
                (index as f32 * 0.2 + 0.1).into(),
            ),
            DisplaySize::new(0.5.into(), 0.2.into()),
        )
    }

    fn render_individual_tree_node(&mut self, path: &TreeNodePath) -> UIElement {
        self.tree[path].render()
    }

    fn handle_individual_tree_event(path: &TreeNodePath, event: Event) {
        println!("Event {:?} occured for demo path {:?}", event, path);
    }
}
impl Component for Test {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        singularity_ui::ui_element::UIElement::Container(vec![
            self.button1.render().contain(Self::button1_area()),
            self.button2.render().contain(Self::button2_area()),
            singularity_ui::ui_element::UIElement::Container(
                self.tree
                    .collect_paths_dfs()
                    .iter()
                    .enumerate()
                    .map(|(index, path)| {
                        self.render_individual_tree_node(path)
                            .contain(Self::generate_tree_area(index, path))
                    })
                    .collect(),
            ),
        ])
    }

    fn handle_event(&mut self, event: singularity_common::tab::packets::Event) {
        match self.focus {
            Focus::Button1 => {
                if let Some(remapped_event) = remap_event(Self::button1_area(), event.clone()) {
                    self.button1.handle_event(remapped_event);
                    return;
                }
            }
            Focus::Button2 => {
                if let Some(remapped_event) = remap_event(Self::button2_area(), event.clone()) {
                    self.button2.handle_event(remapped_event);
                    return;
                }
            }
            Focus::Tree(focused_index, ref focused_path) => {
                if let Some(remapped_event) = remap_event(
                    Self::generate_tree_area(focused_index, focused_path),
                    event.clone(),
                ) {
                    Self::handle_individual_tree_event(focused_path, remapped_event);
                    return;
                }
            }
        }

        if let Some(remapped_event) = remap_event(Self::button1_area(), event.clone()) {
            self.focus = Focus::Button1;
            self.button1.handle_event(remapped_event);
            dbg!("focus updated {self.focus}");
        } else if let Some(remapped_event) = remap_event(Self::button2_area(), event.clone()) {
            self.focus = Focus::Button2;
            self.button2.handle_event(remapped_event);
            dbg!("focus updated {self.focus}");
        } else if let Some((index, path, remapped_event)) = self
            .tree
            .collect_paths_dfs()
            .into_iter()
            .enumerate()
            .find_map(|(index, path)| {
                remap_event(Self::generate_tree_area(index, &path), event.clone())
                    .map(|remapped_event| (index, path, remapped_event))
            })
        {
            Self::handle_individual_tree_event(&path, remapped_event);
            self.focus = Focus::Tree(index, path);
            dbg!("focus updated {self.focus}");
        }
    }
}

#[test]
pub fn run_test() {
    use std::sync::{atomic::AtomicBool, Arc};

    let mut test_widget = Test {
        focus: Focus::Button1,
        button1: Button::new(
            singularity_ui::ui_element::UIElement::CharGrid(CharGrid::new_monostyled(
                "button1".to_string(),
                Color::WHITE,
                Color::BLACK,
            )), // .bordered(Color::LIGHT_GREEN),
        ),
        button2: Button::new(
            singularity_ui::ui_element::UIElement::CharGrid(CharGrid::new_monostyled(
                "button1".to_string(),
                Color::WHITE,
                Color::BLACK,
            ))
            .bordered(Color::ORANGE),
        ),
        tree: RootedTree::from_root(TextBox::new("tree button".to_string()))
            .builder_add_node(TextBox::new("text".to_string()), &TreeNodePath::new_root()),
    };

    let root_element = Arc::new(Mutex::new(test_widget.render()));
    let ui_event_queue = Arc::new(Mutex::new(Vec::new()));
    let is_running = Arc::new(AtomicBool::new(true));

    let ui_event_queue_clone = ui_event_queue.clone();
    let is_running_clone = is_running.clone();
    let ui_thread_handle = std::thread::spawn(move || {
        singularity_ui::UIDisplay::run_display(
            root_element,
            ui_event_queue_clone,
            is_running_clone,
        );
    });

    while is_running.load(std::sync::atomic::Ordering::Relaxed) {
        for ui_event in std::mem::take(&mut *(ui_event_queue.lock().unwrap())) {
            use singularity_ui::ui_event::{KeyModifiers, UIEvent};
            match ui_event {
                UIEvent::KeyPress(key, KeyModifiers::CTRL) if key.raw_code == 16 => {
                    // Ctrl+Q
                    dbg!("Ending demo");
                    is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
                UIEvent::KeyPress(_, _) => {
                    test_widget
                        .handle_event(singularity_common::tab::packets::Event::UIEvent(ui_event));
                }
                UIEvent::WindowResized(_) => {}
                UIEvent::MousePress([[click_x, click_y], [tot_width, tot_height]], container) => {
                    test_widget.handle_event(singularity_common::tab::packets::Event::UIEvent(
                        singularity_ui::ui_event::UIEvent::MousePress(
                            [[click_x, click_y], [tot_width, tot_height]],
                            container,
                        ),
                    ));
                }
            }
        }
    }

    ui_thread_handle.join().unwrap();
}
