use std::ffi::CStr;

use dynamic_plugin::libc;

// plugin_impl! {
//     server::ExamplePlugin,

//     fn do_a_thing() {
//         println!("A thing has been done!");
//     }

//     unsafe fn say_hello(name: *const libc::c_char) -> bool {
//         let name = CStr::from_ptr(name);
//         println!("Hello, {}!", name.to_string_lossy());
//         true
//     }

//     fn trigger_function(a_func: extern "C" fn(u32, u32)) {
//         a_func(5, 3);
//     }
// }

// Slightly modified expansion of above macro
// ==========================================

// // #[deny(const_err)]
// #[allow(unused_must_use)]
// const _: () = {
//     if !(server::ExamplePlugin::PLUGIN_SIGNATURE == 2869018329193855427u64) {
//         {
//             core::panicking::panic_fmt(core::const_format_args!(core::concat!(
//                 "Static assertion '",
//                 core::stringify!((server::ExamplePlugin::PLUGIN_SIGNATURE =  = 2869018329193855427u64)),
//                 "' failed: ",
//                 "The implementation signature does not match the definition. Check that all functions are implemented with the correct types."
//             )));
//         };
//     }
//     ()
// };
#[unsafe(no_mangle)]
pub extern "C" fn _dynamic_plugin_signature() -> u64 {
    2869018329193855427u64
}
#[unsafe(no_mangle)]
pub extern "C" fn do_a_thing() {
    println!("A thing has been done");
}
/// # Safety
/// To satisfy clippy
#[unsafe(no_mangle)]
pub unsafe extern "C" fn say_hello(name: *const libc::c_char) -> bool {
    let name = unsafe { CStr::from_ptr(name) };
    println!("Hello, {}!", name.to_string_lossy());
    true
}
#[unsafe(no_mangle)]
pub extern "C" fn trigger_function(a_func: extern "C" fn(u32, u32)) {
    a_func(5, 3);
}
