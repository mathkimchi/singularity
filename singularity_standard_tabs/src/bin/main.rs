use singularity_sar::runner::AppletRunner;
use singularity_standard_tabs::{editor::TextEditorApplet, image_viewer::ImageViewerApplet};
use singularity_sttk::{
    command_hub::CommandHubApplet,
    nodular_applet::{
        recursive_node_applet::RecursiveNodeApplet, root_node_applet::RootNodeApplet,
    },
};
use std::collections::BTreeMap;

fn main() {
    AppletRunner::run(RootNodeApplet::get_initializer(
        RecursiveNodeApplet::get_initializer(CommandHubApplet::get_boxed_initiator()),
        BTreeMap::from_iter(vec![
            (
                "command_hub".to_string(),
                CommandHubApplet::get_applet_spawner(),
            ),
            (
                "text_editor".to_string(),
                TextEditorApplet::get_applet_spawner(),
            ),
            (
                "image_viewer".to_string(),
                ImageViewerApplet::get_applet_spawner(),
            ),
        ]),
    ))
}
