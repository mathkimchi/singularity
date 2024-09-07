use super::{Subapp, SubappInterface};
use libloading::{Library, Symbol};
use std::ffi::OsStr;

pub type CreateSubappInterface = unsafe fn() -> *mut dyn SubappInterface;

/// # Safety
///
/// Unsafe when Library::new is unsafe
pub fn load_subapp<P: AsRef<OsStr>>(path: P) -> Subapp {
    let subapp_interface = unsafe {
        let lib = Library::new(path).expect("Failed to load library");
        let create_subapp_interface: Symbol<CreateSubappInterface> = lib
            .get(b"create_subapp_interface")
            .expect("failed to get subapp interface creator");
        Box::from_raw(create_subapp_interface())
    };

    let mut subapp = Subapp::from_box(subapp_interface);

    // dbg!(subapp.subapp_interface.dump_requests());

    // subapp

    todo!()
}
