use libc::c_char;
use std::{ffi::OsStr, fs::read_dir, path::Path};

// plugin_interface! {
//     extern trait ExamplePlugin {
//         /// Ask the plugin to do a thing
//         fn do_a_thing();
//         /// Say hello to a person
//         fn say_hello(to: *const c_char) -> bool;
//         /// Here's a function
//         fn trigger_function(a_func: extern "C" fn(u32, u32));
//     }
// }

// Modified expansion of above
// ===============================================

pub struct ExamplePlugin {
    // library: ::dynamic_plugin::PluginDynamicLibrary,
    library: libloading::Library,
}
impl ExamplePlugin {
    pub const PLUGIN_SIGNATURE: u64 = 2869018329193855427u64;
}
impl ExamplePlugin {
    pub fn find_plugins<P>(path: P) -> Vec<Self>
    where
        P: AsRef<Path>,
    {
        let mut plugins = Vec::new();
        if let Ok(paths) = read_dir(path) {
            for path in paths.flatten() {
                if let Ok(plugin) = Self::load_plugin_and_check(path.path()) {
                    plugins.push(plugin);
                }
            }
        }
        plugins
    }
    pub fn load_plugin_and_check<P>(path: P) -> ::dynamic_plugin::Result<Self>
    where
        P: AsRef<OsStr>,
    {
        Self::load_plugin(path, true)
    }
    pub fn load_plugin<P>(path: P, check_signature: bool) -> ::dynamic_plugin::Result<Self>
    where
        P: AsRef<OsStr>,
    {
        unsafe {
            let library = ::dynamic_plugin::PluginDynamicLibrary::new(path)?;
            let func: ::dynamic_plugin::PluginLibrarySymbol<unsafe extern "C" fn() -> u64> =
                library
                    .get(b"_dynamic_plugin_signature")
                    .map_err(|_| ::dynamic_plugin::Error::NotAPlugin)?;
            if check_signature {
                let hash = func();
                if hash != 2869018329193855427u64 {
                    return ::dynamic_plugin::Result::Err(
                        ::dynamic_plugin::Error::InvalidPluginSignature,
                    );
                }
            }
            Ok(Self { library })
        }
    }
    // pub extern "C" fn do_a_thing(&self) -> ::dynamic_plugin::Result<()> {
    //     unsafe {
    //         let func: ::dynamic_plugin::PluginLibrarySymbol<unsafe extern "C" fn() -> ()> =
    //             self.library.get(b"do_a_thing")?;
    //         Ok(func())
    //     }
    // }
    // pub extern "C" fn say_hello(&self, to: *const c_char) -> ::dynamic_plugin::Result<bool> {
    //     unsafe {
    //         let func: ::dynamic_plugin::PluginLibrarySymbol<
    //             unsafe extern "C" fn(*const c_char) -> bool,
    //         > = self.library.get(b"say_hello")?;
    //         Ok(func(to))
    //     }
    // }
    // pub extern "C" fn trigger_function(
    //     &self,
    //     a_func: extern "C" fn(u32, u32),
    // ) -> ::dynamic_plugin::Result<()> {
    //     unsafe {
    //         let func: ::dynamic_plugin::PluginLibrarySymbol<
    //             unsafe extern "C" fn(extern "C" fn(u32, u32)) -> (),
    //         > = self.library.get(b"trigger_function")?;
    //         Ok(func(a_func))
    //     }
    // }
    pub extern "C" fn do_a_thing(&self) -> ::dynamic_plugin::Result<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> ()> =
                self.library.get(b"do_a_thing")?;
            Ok(func())
        }
    }
    pub extern "C" fn say_hello(&self, to: *const c_char) -> ::dynamic_plugin::Result<bool> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> bool> =
                self.library.get(b"say_hello")?;
            Ok(func(to))
        }
    }
    pub extern "C" fn trigger_function(
        &self,
        a_func: extern "C" fn(u32, u32),
    ) -> ::dynamic_plugin::Result<()> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(extern "C" fn(u32, u32)) -> ()> =
                self.library.get(b"trigger_function")?;
            Ok(func(a_func))
        }
    }
}
