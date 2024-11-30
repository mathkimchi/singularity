use serde::{Deserialize, Serialize};
use singularity_common::{
    ask_query,
    components::{button::Button, text_box::TextBox, Component},
    tab::packets::Event,
};
use singularity_macros::ComposeComponents;
use singularity_ui::{
    color::Color,
    display_units::DisplayArea,
    ui_element::{CharGrid, UIElement},
};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

/// NOTE: Immutable
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    /// NOTE: SystemTime should be more or less the same as Instant, just compatable with serde
    start_time: SystemTime,
    end_time: SystemTime,
    title: String,
    notes: String,
}
type Blocks = Vec<Block>;
impl Block {
    pub fn duration(&self) -> Duration {
        // SystemTime isn't perfect, so duration_since's type is result
        // should be negligible though
        self.end_time
            .duration_since(self.start_time)
            .unwrap_or_default()
    }
}

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

    #[component((DisplayArea::new((0.5, 0.0), (1.0, 0.3))), (0))]
    title_editor: TextBox,
    #[component((DisplayArea::new((0.5, 0.3), (1.0, 0.8))), (1))]
    body_editor: TextBox,
    #[component((DisplayArea::new((0.4, 0.7), (0.6, 0.8))), (2))]
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
        let num_blocks = blocks.len();

        manager_handler.send_request(singularity_common::tab::packets::Request::ChangeName(
            "Time Manager".to_string(),
        ));

        Self {
            blocks_file_path: blocks_file_path.into(),
            blocks,
            mode: Mode::Idle,

            focused_component: 0,
            title_editor: TextBox::new(format!("Block {}", num_blocks)),
            body_editor: TextBox::default(),
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

        // render the past blocks
        let blocks = self
            .blocks
            .iter()
            .map(|block| format!("{}: {:#?}", block.title, block.duration()))
            .collect::<Vec<String>>()
            .join("\n");

        Some(
            UIElement::Container(vec![
                UIElement::CharGrid(CharGrid::from(blocks)),
                self.render_components(),
            ])
            .bordered(Color::LIGHT_GREEN),
        )
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
        } else {
            if let Err(Some(focused_component)) = self.forward_events_to_focused(event.clone()) {
                self.focused_component = focused_component;
                self.forward_events_to_focused(event).unwrap();
            }
            if self.button.was_clicked() {
                // alternate mode
                match self.mode {
                    Mode::Timing { start_time } => {
                        // was timing, now can stop timing

                        // log the finished block
                        let new_block = Block {
                            start_time,
                            end_time: SystemTime::now(),
                            title: self.title_editor.get_text_as_string(),
                            notes: self.body_editor.get_text_as_string(),
                        };
                        self.blocks.push(new_block);

                        // restart the ui
                        self.title_editor = TextBox::new(format!("Block {}", self.blocks.len()));
                        self.body_editor = TextBox::default();

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
}
