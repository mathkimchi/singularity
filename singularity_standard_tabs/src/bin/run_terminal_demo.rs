use singularity_sar::runner::AppletRunner;
use singularity_standard_tabs::terminal::TerminalApplet;
use singularity_sttk::nodular_applet::recursive_node_applet::RecursiveNodeApplet;

fn main() {
    AppletRunner::run(
        singularity_sttk::nodular_applet::root_node_applet::RootNodeApplet::get_initializer(
            RecursiveNodeApplet::get_initializer(TerminalApplet::get_boxed_initiator()),
        ),
    )
}
