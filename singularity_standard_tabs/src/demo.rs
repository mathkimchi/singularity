#![cfg(test)]

use singularity_common::{
    components::{button::Button, text_box::TextBox, Component},
    utils::tree::{rooted_tree::RootedTree, tree_node_path::TreeNodePath},
};
use singularity_macros::ComposeComponents;
use singularity_ui::{
    color::Color,
    display_units::{DisplayArea, DisplayCoord, DisplaySize},
    ui_element::{CharGrid, UIElement},
};
use std::sync::Mutex;

#[derive(ComposeComponents)]
pub struct Test {
    /// this name is a keyword for ComposeComponents
    focused_component: usize,
    #[component((DisplayArea::new((0.0, 0.0), (0.25, 0.05))), (0))]
    button: Button,
    #[component((DisplayArea::new((0.7, 0.0), (0.9, 0.05))), (1))]
    button2: Button,

    /// REVIEW: this doesn't directly have anything to do with compose components (might need to decouple)
    /// TODO: Ideal syntax: `...($index, $path)...` instead of `...(__index, __path)...`
    #[tree_component((Self::generate_tree_area(__index, __path)), (self.render_individual_tree_node(__path)), (), (2))]
    tree: RootedTree<TextBox>,
}
impl Test {
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
}
impl Component for Test {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        // singularity_ui::ui_element::UIElement::Container(vec![self
        //     .button
        //     .render()
        //     .contain(DisplayArea::FULL)])
        self.render_components()
    }

    fn handle_event(&mut self, event: singularity_common::tab::packets::Event) {
        if let Err(Some(clicked_component_index)) = self.forward_events_to_focused(event.clone()) {
            self.focused_component = clicked_component_index;
            self.forward_events_to_focused(event).unwrap();
        }
    }
}

#[test]
pub fn run_test() {
    use std::sync::{atomic::AtomicBool, Arc};

    let mut test_widget = Test {
        focused_component: 0,
        button: Button::new(
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
