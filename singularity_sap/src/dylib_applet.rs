use libc::c_void;
use std::{ffi::OsStr, path::Path};

pub const DYLIB_APPLET_SIGNATURE: u64 = 784532439874536;

/// Just a way of representing a byte slice like `&[u8]` for FFI's.
/// From [here](https://users.rust-lang.org/t/how-to-return-byte-array-from-rust-function-to-ffi-c/18136/4)
/// and the [rustonomicon](https://doc.rust-lang.org/nomicon/ffi.html).
/// REVIEW: Not sure if the naming makes sense.
///
/// The `Range<*const T>` returned by [`slice::as_ptr_range`] would be good,
/// but it isn't marked `#[repr(C)]` so I guess it doesn't work.
#[repr(C)]
pub struct CBytes {
    bytes: *const u8,
    len: usize,
}
impl From<&[u8]> for CBytes {
    fn from(value: &[u8]) -> Self {
        Self {
            bytes: value.as_ptr(),
            len: value.len(),
        }
    }
}
#[repr(C)]
pub struct CMutBytes {
    bytes: *mut u8,
    len: *mut usize,
}

/// When we call a global function of a dylib applet,
/// we give it the `GlobalAppletContext` so it can call things like `request`.
#[repr(C)]
pub struct GlobalAppletContext {
    /// Should contain all the information needed for request and query.
    ctxt: *const c_void,
    request: extern "C" fn(CBytes, *const c_void),
    /// The `CMutBytes` is the output buffer.
    query: extern "C" fn(CBytes, *const c_void, CMutBytes),
}
#[cfg(feature = "client")] // These impls shoud be used by the client
impl GlobalAppletContext {
    /// Passes on the bytes for a request.
    pub fn request_bytes(&self, request_bytes: &[u8]) {
        (self.request)(CBytes::from(request_bytes), self.ctxt);
    }
    /// Given the bytes for a query, returns response as bytes.
    pub fn query_bytes(&self, query_bytes: &[u8]) -> Vec<u8> {
        let mut response_buffer = Vec::new();
        // TODO: yeah idk how to do this rn
        (self.query)(
            CBytes::from(query_bytes),
            self.ctxt,
            CMutBytes {
                bytes: response_buffer.as_mut_ptr(),
                len: todo!(),
            },
        );
        response_buffer
    }
}

/// Represents client on the server side.
#[cfg(feature = "server")]
pub struct DylibClient {
    library: libloading::Library,
}
impl DylibClient {
    pub fn find_plugins<P>(path: P) -> Vec<Self>
    where
        P: AsRef<Path>,
    {
        let mut plugins = Vec::new();
        if let Ok(paths) = std::fs::read_dir(path) {
            for path in paths.flatten() {
                if let Some(plugin) = Self::load_plugin(path.path()) {
                    plugins.push(plugin);
                }
            }
        }
        plugins
    }
    // TODO: use result instead of option ; this kind of goes for the entire codebase. Ctrl+F `ok()?`
    pub fn load_plugin<P>(path: P) -> Option<Self>
    where
        P: AsRef<OsStr>,
    {
        unsafe {
            let library = libloading::Library::new(path).ok()?;
            let dylib_applet_signature: libloading::Symbol<unsafe extern "C" fn() -> u64> =
                library.get(b"_get_applet_signature").ok()?;

            let hash = dylib_applet_signature();
            if hash != DYLIB_APPLET_SIGNATURE {
                return None;
            }

            Some(Self { library })
        }
    }

    // pub fn do_a_thing(&self) -> Option<()> {
    //     unsafe {
    //         let func: libloading::Symbol<unsafe extern "C" fn() -> ()> =
    //             self.library.get(b"do_a_thing").ok()?;
    //         Some(func())
    //     }
    // }
    // pub fn say_hello(&self, to: *const c_char) -> Option<bool> {
    //     unsafe {
    //         let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> bool> =
    //             self.library.get(b"say_hello").ok()?;
    //         Some(func(to))
    //     }
    // }
    // pub fn trigger_function(&self, a_func: extern "C" fn(u32, u32)) -> Option<()> {
    //     unsafe {
    //         let func: libloading::Symbol<unsafe extern "C" fn(extern "C" fn(u32, u32)) -> ()> =
    //             self.library.get(b"trigger_function").ok()?;
    //         Some(func(a_func))
    //     }
    // }
}
