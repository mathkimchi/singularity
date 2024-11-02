use super::TabCreator;
use crate::{project::project_settings::TabData, utils::tree::tree_node_path::TreeNodePath};
use singularity_ui::{display_units::DisplayArea, ui_event::UIEvent};

#[derive(Debug, Clone)]
pub enum Event {
    UIEvent(UIEvent),
    Resize(DisplayArea),
    /// TODO: close forcibly
    Close,
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

        pub struct QueryAskChannels {
            pub query_tx: std::sync::mpsc::Sender<QueryTypes>,
            $(
                pub [<$query_name:snake _rx>] : std::sync::mpsc::Receiver<$response_type>,
            )*
        }
        #[macro_export]
        macro_rules! ask_query {
            $(($ask_channel:tt, $query_name) => {
                $ask_channel.query_tx
                    .send(QueryTypes::$query_name)
                    .expect("failed to send query");

                $ask_channel.[<$query_name:snake _rx>]
                    .recv()
                    .expect("failed to get response")
            };)*
        }

        pub struct QueryAnswerChannels {
            pub query_rx: std::sync::mpsc::Receiver<QueryTypes>,
            $(pub [<$query_name:snake _tx>]: std::sync::mpsc::Sender<$response_type>,)*
        }
        #[automatically_derived]
        impl QueryAnswerChannels {
            pub fn answer_query<$([<$query_name Responder>]: FnOnce() -> $response_type,)*>(&self, $([<$query_name:snake _responder>]: [<$query_name Responder>],)*) {
                if let Ok(query) = self.query_rx.try_recv() {
                    match query {
                        $(
                            QueryTypes::$query_name => {
                                self.[<$query_name:snake _tx>]
                                    .send([<$query_name:snake _responder>]())
                                    .expect("failed to send response");
                            },
                        )*
                    }
                }
            }
        }

        pub fn create_query_channels() -> (QueryAskChannels, QueryAnswerChannels) {
            let (query_tx, query_rx) = std::sync::mpsc::channel();

            $(
                let ([<$query_name:snake _tx>], [<$query_name:snake _rx>]) = std::sync::mpsc::channel();
            )*

            (
                QueryAskChannels {
                    query_tx,
                    $(
                        [<$query_name:snake _rx>],
                    )*
                },
                QueryAnswerChannels {
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
fn a() {
    let (ask, answer) = create_query_channels();

    ask_query!(ask, OrgPath);
}

/// TODO: auto generate this with macro
pub enum Query {
    Path,
    Name,
    TabData,
}

#[derive(Debug)]
pub enum Response {
    Path(TreeNodePath),
    Name(String),
    TabData(TabData),
}
/// TODO: macro this
impl Response {
    pub fn try_as_path(self) -> Option<TreeNodePath> {
        if let Self::Path(path) = self {
            Some(path)
        } else {
            None
        }
    }
    pub fn try_as_name(self) -> Option<String> {
        if let Self::Name(name) = self {
            Some(name)
        } else {
            None
        }
    }
    pub fn try_as_tab_data(self) -> Option<TabData> {
        if let Self::TabData(tab_data) = self {
            Some(tab_data)
        } else {
            None
        }
    }
}
