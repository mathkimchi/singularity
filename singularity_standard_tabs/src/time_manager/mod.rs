use serde::{Deserialize, Serialize};
use singularity_common::{
    ask_query,
    components::{button::Button, Component},
    tab::packets::Event,
};
use singularity_macros::ComposeComponents;
use singularity_ui::{
    color::Color,
    display_units::DisplayArea,
    ui_element::{CharGrid, UIElement},
};
use std::{path::PathBuf, time::SystemTime};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Block {
    /// I think SystemTime should be more or less the same as Instant, just compatable with serde
    start_time: SystemTime,
    end_time: SystemTime,
}
type Blocks = Vec<Block>;

enum Mode {
    Timing { start_time: SystemTime },
    Idle,
}

#[derive(ComposeComponents)]
pub struct TimeManager {
    blocks_file_path: PathBuf,

    blocks: Blocks,
    mode: Mode,

    focused_component: usize,
    #[component((DisplayArea::new((0.0, 0.0), (1.0, 0.05))), (0))]
    button: Button,
}
impl TimeManager {
    pub fn new_from_project<P>(
        project_path: P,
        manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let mut blocks_file_path: PathBuf = project_path.into();
        blocks_file_path.push(".project");
        blocks_file_path.push("blocks.json");

        Self::new::<PathBuf>(blocks_file_path, manager_handler)
    }

    pub fn new<P>(
        blocks_file_path: P,
        manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: std::convert::From<P>,
    {
        let blocks = Self::parse_blocks(&blocks_file_path).unwrap_or_default();

        manager_handler.send_request(singularity_common::tab::packets::Request::ChangeName(
            "Time Manager".to_string(),
        ));

        Self {
            blocks_file_path: blocks_file_path.into(),
            blocks,
            mode: Mode::Idle,

            focused_component: 0,
            button: Button::new(
                singularity_ui::ui_element::UIElement::CharGrid(CharGrid::from(
                    "Idle - Click to Start".to_string(),
                ))
                .bordered(Color::LIGHT_GREEN),
            ),
        }
    }

    fn parse_blocks<P>(blocks_file_path: &P) -> Option<Blocks>
    where
        P: AsRef<std::path::Path>,
    {
        serde_json::from_str(&std::fs::read_to_string(blocks_file_path).ok()?).ok()
    }

    /// TODO: make button's `was_clicked` feature a macro so it is more flexible
    fn update_button_ui(&mut self) {
        self.button.inner_element = UIElement::CharGrid(CharGrid::from(match self.mode {
            Mode::Timing { start_time } => {
                format!(
                    "Timing - {:#?} elapsed",
                    start_time.elapsed().unwrap_or_default()
                )
            }
            Mode::Idle => "Idle - Click to Start".to_string(),
        }))
        .bordered(Color::LIGHT_GREEN);
    }

    fn save_to_file(&self) {
        std::fs::write(
            &self.blocks_file_path,
            serde_json::to_string_pretty(&self.blocks).unwrap(),
        )
        .unwrap();
    }
}
impl singularity_common::tab::BasicTab for TimeManager {
    fn initialize_tab(manager_handler: &singularity_common::tab::ManagerHandler) -> Self {
        Self::new_from_project(
            serde_json::from_value::<String>(
                ask_query!(manager_handler.get_query_channels(), TabData).session_data,
            )
            .unwrap(),
            manager_handler,
        )
    }

    fn render_tab(
        &mut self,
        _manager_handler: &singularity_common::tab::ManagerHandler,
    ) -> Option<singularity_ui::ui_element::UIElement> {
        self.update_button_ui();
        Some(self.render_components())
    }

    fn handle_tab_event(
        &mut self,
        event: singularity_common::tab::packets::Event,
        _manager_handler: &singularity_common::tab::ManagerHandler,
    ) {
        use singularity_ui::ui_event::{KeyModifiers, KeyTrait, UIEvent};
        if let Event::UIEvent(UIEvent::KeyPress(key, KeyModifiers::CTRL)) = event {
            if key.to_char() == Some('s') {
                self.save_to_file();
            }
        } else if self.forward_events_to_focused(event).is_ok() && self.button.was_clicked() {
            // alternate mode
            match self.mode {
                Mode::Timing { start_time } => {
                    // was timing, now can stop timing

                    let new_block = Block {
                        start_time,
                        end_time: SystemTime::now(),
                    };
                    self.blocks.push(new_block);

                    self.mode = Mode::Idle;
                }
                Mode::Idle => {
                    // was idle, now start timing
                    self.mode = Mode::Timing {
                        start_time: SystemTime::now(),
                    }
                }
            }
        }
    }
}
