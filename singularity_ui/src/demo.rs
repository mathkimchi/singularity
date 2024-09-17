fn main() {
    use iced::{widget::text, Sandbox, Settings};

    struct IcedApp {}
    impl Sandbox for IcedApp {
        type Message = ();

        fn new() -> Self {
            IcedApp {}
        }

        fn title(&self) -> String {
            "Hi".to_string()
        }

        fn update(&mut self, message: Self::Message) {}

        fn view(&self) -> iced::Element<Self::Message> {
            iced::widget::column![text("Hello!")].into()
        }
    }

    IcedApp::run(Settings::default()).unwrap();
}
