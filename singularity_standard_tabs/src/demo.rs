use singularity_common::components::{button::Button, Component};
use singularity_macros::ComposeComponents;
use singularity_ui::display_units::DisplayArea;

#[derive(ComposeComponents)]
pub struct Test {
    focused_component: usize,
    #[component(DisplayArea::FULL)]
    button: Button,
    #[component(DisplayArea::FULL)]
    button2: Button,
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
        self.forward_events_to_focused(event)
    }
}
