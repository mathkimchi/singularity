use crate::{
    applet_data::{AppletSpawnMethod, AppletTypeId},
    project_settings::Project,
    tile::Tiles,
};
use serde::{Deserialize, Serialize};
use singularity_common::utils::{
    id_map::{Id, IdMap},
    tree::id_tree::IdTree,
};
use sonamu_ui::display_units::DisplayArea;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct OpenTab {
    pub applet_type_id: Option<AppletTypeId>,

    /// is kind of dangerous to let user change the id of a tab, but if they screw this up, it is their fault
    pub tab_area: DisplayArea,
    /// NOTE: Read devlog ~2024/10/29 and 2025/02/19 for description; this is like SessionStorage for webdev
    /// REVIEW: rename?
    /// REVIEW: include Area and UIElement and TabType into this?
    /// This type is kind of a black sheep
    pub applet_session_storage: serde_json::Value,
    /// If this is None, then checks the applet type's default spawn data.
    /// In other words, if this applet was spawned from default spawn, then just leave this none.
    pub spawn_method: Option<AppletSpawnMethod>,
}

/// Data for the whole session, things like opened tabs and their sessions as well as focused tab.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionData {
    pub tabs: IdMap<OpenTab>,

    /// ORGanizational tree
    pub org_tree: IdTree<OpenTab>,
    pub focused_tab: Id<OpenTab>,

    // /// currently, last in vec is "top" in gui
    // pub display_order: Vec<Uuid>,
    pub display_tiles: Tiles<OpenTab>,
}
impl SessionData {
    pub fn try_parse_from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)
            .expect("session data should be formatted correctly"))
    }

    // NOTE: this type currently isn't suited for manual modification. It's really for the SDE to serialize to and from
    #[must_use]
    pub fn new(project: &Project) -> Self {
        let id = Id::generate();

        let root_tab = OpenTab {
            applet_type_id: Some(AppletTypeId::FileManager),
            tab_area: DisplayArea::new((0., 0.), (0.5, 1.)),
            applet_session_storage: serde_json::to_value(project.get_project_directory().clone())
                .unwrap(),
            spawn_method: None,
        };

        let org_tree = IdTree::new(id);
        let mut tabs = IdMap::new();
        tabs.insert(id, root_tab);
        let display_tiles = Tiles::new_from_root(id);

        Self {
            tabs,
            org_tree,
            focused_tab: id,
            display_tiles,
        }
    }
}

pub struct Session {
    pub project: Project,
    pub session_data: SessionData,
}
impl Session {
    /// Given the project directory, gets the previously closed session (and project settings) if they exist, otherwise starts a new session.
    pub fn get_or_make_session<P>(project_directory: P) -> Self
    where
        P: AsRef<Path> + Clone,
        PathBuf: From<P>,
    {
        let project = Project::open_or_make(project_directory.clone());
        let session_data = SessionData::try_parse_from_file(
            PathBuf::from(project_directory).join(".project/session_data.json"),
        )
        .ok()
        .unwrap_or_else(|| SessionData::new(&project));

        Self {
            project,
            session_data,
        }
    }

    pub fn save_to_file(&self) {
        self.project.save_to_file();

        let core_project_settings_path = self
            .project
            .get_project_directory()
            .join(".project/session_data.json");
        let serialized_project = serde_json::to_string_pretty(&self.session_data).unwrap();
        std::fs::write(core_project_settings_path, serialized_project)
            .expect("failed to write serialized project to `.project/session_data.json`");
    }
}
