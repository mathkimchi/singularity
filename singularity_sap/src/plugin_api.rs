pub trait SingularityPlugin {
    fn start_plugin(plugin_helper: PluginHelper);
}

pub struct PluginHelper {}

pub mod wasm_bindings {
    use wit_bindgen::generate;

    generate!({path: "./wit/world.wit", pub_export_macro: true, });
}
