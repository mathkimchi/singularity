//! From https://github.com/lilopkins/dynamic-plugins-rs/tree/main/example-plugin-host/src

use server::ExamplePlugin;

extern "C" fn a_func(a: u32, b: u32) {
    println!("{} + {} = {}", a, b, a + b);
}

fn main() -> dynamic_plugin::Result<()> {
    let plugin = ExamplePlugin::load_plugin_and_check("./plugins")?;

    plugin.trigger_function(a_func)?;

    Ok(())
}
