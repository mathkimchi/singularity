use super::TabCreator;
use crate::{project::project_settings::TabData, utils::tree::tree_node_path::TreeNodePath};
use singularity_ui::{display_units::DisplayArea, ui_event::UIEvent};

#[derive(Debug, Clone)]
pub enum Event {
    UIEvent(UIEvent),
    Resize(DisplayArea),
    Focused,
    Unfocused,
    /// TODO: close forcibly
    Close,
}
impl Event {
    /// if mouseclick, return Some(remap area) if clicked within, else return None.
    /// For other events, just return normally
    pub fn remap(
        &self,
        area: singularity_ui::display_units::DisplayArea,
    ) -> Option<crate::tab::packets::Event> {
        use crate::tab::packets::Event;
        use singularity_ui::{display_units::DisplayCoord, ui_event::UIEvent};

        let event = self.clone();

        if let Event::UIEvent(singularity_ui::ui_event::UIEvent::MousePress(
            [[click_x, click_y], [tot_width, tot_height]],
            container,
        )) = event
        {
            if area.map_onto(container).contains(
                DisplayCoord::new((click_x as i32).into(), (click_y as i32).into()),
                [tot_width as i32, tot_height as i32],
            ) {
                Some(Event::UIEvent(UIEvent::MousePress(
                    [[click_x, click_y], [tot_width, tot_height]],
                    area.map_onto(container),
                )))
            } else {
                None
            }
        } else {
            Some(event)
        }
    }
}

pub enum Request {
    ChangeName(String),
    SpawnChildTab(Box<dyn TabCreator>, TabData),
}

macro_rules! query_macro {
    ($($query_name:ident => $response_type:ty),*) => {paste::paste!{
        pub enum QueryTypes {
            $($query_name),*
        }

        pub struct QueryChannels {
            pub query_tx: std::sync::mpsc::Sender<QueryTypes>,
            $(
                pub [<$query_name:snake _rx>] : std::sync::mpsc::Receiver<$response_type>,
            )*
        }
        #[macro_export]
        macro_rules! ask_query {
            $(($ask_channel:expr, $query_name) => {
                {
                    $ask_channel.query_tx
                        .send($crate::tab::packets::QueryTypes::$query_name)
                        .expect("failed to send query");

                    $ask_channel.[<$query_name:snake _rx>]
                        .recv()
                        .expect("failed to get response")
                }
            };)*
        }

        pub struct RespondChannels {
            pub query_rx: std::sync::mpsc::Receiver<QueryTypes>,
            $(pub [<$query_name:snake _tx>]: std::sync::mpsc::Sender<$response_type>,)*
        }
        #[automatically_derived]
        impl RespondChannels {
            pub fn answer_query<$([<$query_name Responder>]: FnOnce() -> $response_type,)*>(&self, $([<$query_name:snake _responder>]: [<$query_name Responder>],)*) {
                if let Ok(query) = self.query_rx.try_recv() {
                    match query {
                        $(
                            $crate::tab::packets::QueryTypes::$query_name => {
                                self.[<$query_name:snake _tx>]
                                    .send([<$query_name:snake _responder>]())
                                    .expect("failed to send response");
                            },
                        )*
                    }
                }
            }
        }

        pub fn create_query_channels() -> (QueryChannels, RespondChannels) {
            let (query_tx, query_rx) = std::sync::mpsc::channel();

            $(
                let ([<$query_name:snake _tx>], [<$query_name:snake _rx>]) = std::sync::mpsc::channel();
            )*

            (
                QueryChannels {
                    query_tx,
                    $(
                        [<$query_name:snake _rx>],
                    )*
                },
                RespondChannels {
                    query_rx,
                    $(
                        [<$query_name:snake _tx>],
                    )*
                }
            )
        }
    }};
}
query_macro!(OrgPath => TreeNodePath, TabName => String, TabData => TabData);
// TODO: add something to get the project directory
