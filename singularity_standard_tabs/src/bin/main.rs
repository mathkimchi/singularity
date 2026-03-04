use singularity_sar::runner::AppletRunner;
use singularity_sttk::{
    command_hub::CommandHubApplet, nodular_applet::recursive_node_applet::RecursiveNodeApplet,
};

fn main() {
    AppletRunner::run(
        singularity_sttk::nodular_applet::root_node_applet::RootNodeApplet::get_initializer(
            RecursiveNodeApplet::get_initializer(CommandHubApplet::get_boxed_initiator()),
        ),
    )
}
