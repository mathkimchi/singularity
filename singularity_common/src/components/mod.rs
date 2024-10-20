//! NOTE: Widget kind of just means it is more complicated than a bare bones component

pub mod button;
pub mod text_box;
pub mod timer_widget;
// pub mod tree_viewer;

pub trait Component: Send {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement;

    fn handle_event(&mut self, event: crate::tab::packets::Event);
}

/// REVIEW: naming
/// REVIEW: is this a good idea? (feels kind of bulky to have everything like `EnclosedComponent<InnerComponent>`)
pub struct EnclosedComponent<InnerComponent: Component + ?Sized> {
    pub area: singularity_ui::display_units::DisplayArea,
    pub inner_component: InnerComponent,
}
impl<InnerComponent: Component> EnclosedComponent<InnerComponent> {
    pub fn new(
        inner_component: InnerComponent,
        area: singularity_ui::display_units::DisplayArea,
    ) -> Self {
        Self {
            area,
            inner_component,
        }
    }
}
impl<InnerComponent: Component> Component for EnclosedComponent<InnerComponent> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        self.inner_component.render().contain(self.area)
    }

    /// currently, only special behavior is mouseclick
    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        use crate::tab::packets::Event;
        use singularity_ui::{display_units::DisplayCoord, ui_event::UIEvent};

        if let Event::UIEvent(singularity_ui::ui_event::UIEvent::MousePress(
            [[click_x, click_y], [tot_width, tot_height]],
            container,
        )) = event
        {
            if self.area.map_onto(container).contains(
                DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                [tot_width as i32, tot_height as i32],
            ) {
                self.inner_component
                    .handle_event(Event::UIEvent(UIEvent::MousePress(
                        [[click_x, click_y], [tot_width, tot_height]],
                        self.area.map_onto(container),
                    )));
            }
        } else {
            self.inner_component.handle_event(event);
        }
    }
}

impl<T: Component> Component for Option<T> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        self.as_mut().map(|inner| inner.render()).into()
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        if let Some(inner) = self.as_mut() {
            inner.handle_event(event)
        }
    }
}
impl<T: Component> Component for Box<T> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        T::render(self)
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        T::handle_event(self, event)
    }
}
impl<T: Component> Component for std::sync::Arc<std::sync::Mutex<T>> {
    fn render(&mut self) -> singularity_ui::ui_element::UIElement {
        T::render(self.lock().as_mut().unwrap())
    }

    fn handle_event(&mut self, event: crate::tab::packets::Event) {
        T::handle_event(self.lock().as_mut().unwrap(), event)
    }
}

/// REVIEW: is this a bad idea? I might be tightly coupling all other logic with the ui logic
pub struct ComponentContainer<T: Send> {
    /// REVIEW: storing mutex to containers might reduce bulk
    // pub children: Vec<std::sync::Arc<std::sync::Mutex<EnclosedComponent<Box<dyn Component>>>>>,
    // pub children: Vec<EnclosedComponent<Box<dyn Component>>>,
    // pub children: Vec<Box<dyn Component>>,
    pub children: T,
    /// the only current use of this is for event forwarding
    pub focused_child: usize,
}
// TODO: this is reverse order
macro_rules! component_container_tuple_impls {
    (($current:ident, $index:tt),) => {
        impl<$current> Component for ComponentContainer<(EnclosedComponent<$current>,)>
        where
            $current: Component + Send,
        {
            fn render(&mut self) -> singularity_ui::ui_element::UIElement {
                self.children.$index.render()
            }

            fn handle_event(&mut self, event: crate::tab::packets::Event) {
                self.children.$index.handle_event(event);
            }
        }
    };
    (($head:ident, $index:tt), $(($tail:ident, $tail_index:tt),)*) => {
        impl<$head, $( $tail ),*> Component for ComponentContainer<(EnclosedComponent<$head>, $(EnclosedComponent<$tail>),*)>
        where
            $head: Component + Send,
            $($tail: Component + Send,)*
        {
            fn render(&mut self) -> singularity_ui::ui_element::UIElement {
                singularity_ui::ui_element::UIElement::Container(
                    vec![
                        self.children.$index.render(),
                        $(
                            self.children.$tail_index.render(),
                        )*
                    ]
                )
            }

            fn handle_event(&mut self, event: crate::tab::packets::Event) {
                match self.focused_child {
                    $index => self.children.$index.handle_event(event),
                    $(
                        $tail_index => self.children.$tail_index.handle_event(event),
                    )*
                    _ => panic!()
                }
            }
        }

        component_container_tuple_impls!($(($tail,$tail_index),)*);
    };
}

// TODO: abstract this
component_container_tuple_impls!((C, 2), (B, 1), (A, 0),);
