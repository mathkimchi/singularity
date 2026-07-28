use singularity_sar::runner::AppletRunner;
use singularity_standard_tabs::{
    code_editor::CodeEditorApplet, image_viewer::ImageViewerApplet,
    one_char_render_test::OneCharRenderTestApplet,
};
use singularity_sttk::{
    command_hub::CommandHubApplet,
    nodular_applet::{
        recursive_node_applet::RecursiveNodeApplet, root_node_applet::RootNodeApplet,
    },
};
use singularity_wl_compositor::WaylandApplet;
use std::collections::BTreeMap;
use tracing_subscriber::EnvFilter;

fn main() {
    init_logging();

    AppletRunner::run(RootNodeApplet::get_initializer(
        RecursiveNodeApplet::get_initializer(CommandHubApplet::get_boxed_initiator()),
        BTreeMap::from_iter(vec![
            (
                "command_hub".to_string(),
                CommandHubApplet::get_applet_spawner(),
            ),
            (
                "code_editor".to_string(),
                CodeEditorApplet::get_applet_spawner(),
            ),
            (
                "image_viewer".to_string(),
                ImageViewerApplet::get_applet_spawner(),
            ),
            ("wl_app".to_string(), WaylandApplet::get_applet_spawner()),
            (
                "char_render_test".to_string(),
                OneCharRenderTestApplet::get_applet_spawner(),
            ),
        ]),
    ))
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("warn,singularity=trace"))
        .init();
}
