use iced::{executor, Command};
use std::sync::{Arc, Mutex};

pub type DisplayArea = (usize, usize);
// pub type DisplayBuffer = Vec<u8>;
pub type UIEvent = ();

#[derive(Debug, Clone)]
pub enum UIElement {
    Div(Vec<UIElement>),
    Letter(char),
}
impl From<&UIElement> for iced::Element<'_, ()> {
    fn from(value: &UIElement) -> Self {
        match value {
            UIElement::Div(vec) => {
                iced::widget::column(vec.iter().map(|child| child.into())).into()
            }
            UIElement::Letter(c) => iced::widget::text(c).into(),
        }
    }
}
impl UIElement {
    pub fn get_as_iced_element(&self) -> iced::Element<'static, ()> {
        match self {
            UIElement::Div(vec) => {
                iced::widget::column(vec.iter().map(|child| child.into())).into()
            }
            UIElement::Letter(c) => iced::widget::button(iced::widget::text(c)).into(),
        }
    }
}

pub struct UIDisplay {
    root_element: Arc<Mutex<UIElement>>,
}
impl UIDisplay {
    pub fn create_display(root_element: Arc<Mutex<UIElement>>) -> UIDisplay {
        UIDisplay { root_element }
    }

    // pub fn render_display_buffer(&self, display_area: DisplayArea, display_buffer: DisplayBuffer) {}

    pub fn try_iter_events(&self) -> Vec<UIEvent> {
        todo!()
    }

    pub fn run_display(root_element: Arc<Mutex<UIElement>>) {
        use iced::Application;
        // winit::event_loop::EventLoop::builder()
        //     .with_any_thread(true)
        //     .build()
        //     .unwrap()
        //     .run();

        std::thread::spawn(|| {
            Self::run(iced::Settings {
                flags: root_element,

                id: None,
                window: iced::window::Settings::default(),
                fonts: Vec::new(),
                default_font: iced::Font::default(),
                default_text_size: iced::Pixels(16.0),
                antialiasing: false,
            })
            .unwrap();
        });
    }
}

#[cfg(feature = "iced_backend")]
impl iced::Application for UIDisplay {
    type Message = ();
    type Executor = executor::Default;
    type Theme = iced::Theme;
    type Flags = Arc<Mutex<UIElement>>;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            UIDisplay {
                root_element: flags,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Window")
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<()> {
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        dbg!(&self.root_element.lock());
        self.root_element.lock().unwrap().get_as_iced_element()
    }

    fn theme(&self) -> Self::Theme {
        iced::Theme::Dark
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch(vec![iced::keyboard::on_key_press(|_, _| {
            dbg!("Hi");
            Some(())
        })])
    }
}
#[cfg(not(feature = "iced_backend"))]
compile_error!("");
