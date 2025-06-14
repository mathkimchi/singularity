use libc::c_void;
use std::{ffi::OsStr, path::Path};

use crate::packet::{EventPacketTrait, EventPacketUnion};

/// "Generated" by mashing keyboard.
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
/// Whoever owns this object is in charge of freeing the slice this points to.
///
/// Assumes both sides of FFI are written in rust and are using this library.
///
/// From: https://users.rust-lang.org/t/how-to-return-byte-array-from-rust-function-to-ffi-c/18136/4.
#[repr(C)]
pub struct CVec {
    bytes_ptr: *mut u8,
    len: usize,
    /// REVIEW: check if this is actually needed; the rustlang thread doesn't use it.
    capacity: usize,
}
impl From<Vec<u8>> for CVec {
    fn from(mut value: Vec<u8>) -> Self {
        let bytes_ptr = value.as_mut_ptr();
        let len = value.len();
        let capacity = value.capacity();

        // https://stackoverflow.com/questions/74824779/why-is-it-considered-safe-to-memforget-boxes
        // this memory is leaked here but will be freed in `drop` or when converted to vec.
        std::mem::forget(value);

        Self {
            bytes_ptr,
            len,
            capacity,
        }
    }
}
impl Drop for CVec {
    fn drop(&mut self) {
        // REVIEW: is this going to double free?
        let vec: Vec<u8> = unsafe { Vec::from_raw_parts(self.bytes_ptr, self.len, self.capacity) };
        // unnecessary but highlights that the vec is dropped
        std::mem::drop(vec);
    }
}
impl From<CVec> for Vec<u8> {
    fn from(value: CVec) -> Self {
        let vec = unsafe { Vec::from_raw_parts(value.bytes_ptr, value.len, value.capacity) };

        // prevent double freeing the vec in CVec's drop
        std::mem::forget(value);

        vec
    }
}

/// When we call a function of a dylib applet,
/// we give it the `AppletContext` so it can call things like `request`.
///
/// This is pretty much just the server handler on the client side.
///
/// This is passed on both global and tab function calls,
/// though the implementation provided by the SDE might be different for the two.
///
/// REVIEW: rename to `ServerHandler`?
#[repr(C)]
pub struct AppletContext {
    /// Should contain all the information needed for request and query.
    ctxt: *const c_void,
    request_bytes_fn: extern "C" fn(CBytes, *const c_void),
    /// The `CMutBytes` is the output buffer.
    query_bytes_fn: extern "C" fn(CBytes, *const c_void) -> CVec,
}
#[cfg(feature = "client")] // These impls shoud be used by the client
mod global_applet_context_client_impls {
    use crate::{
        datable::TryFromData,
        dylib_applet::{AppletContext, CBytes},
        packet::{RequestPacketTrait, UniversalQueryTrait},
    };

    impl AppletContext {
        /// Passes on the bytes for a request.
        fn request_bytes(&self, request_bytes: &[u8]) {
            (self.request_bytes_fn)(CBytes::from(request_bytes), self.ctxt);
        }
        pub fn send_request<R: RequestPacketTrait>(&self, request: R) {
            // NOTE: as stated in the 2025-06-14 devlog, the byte structure for reactive applets are different than for active applets' universal stream
            let request_bytes = {
                let request_data = &request.to_data();

                [R::PACKET_TYPE_ID.to_be_bytes().as_slice(), request_data].concat()
            };

            self.request_bytes(&request_bytes);
        }

        /// Given the bytes for a query, returns response as bytes.
        fn query_bytes(&self, query_bytes: &[u8]) -> Vec<u8> {
            // REVIEW
            (self.query_bytes_fn)(CBytes::from(query_bytes), self.ctxt).into()
        }
        pub fn query<Q: UniversalQueryTrait>(&mut self, query: Q) -> Option<Q::ResponseType> {
            // send query

            // NOTE: as stated in the 2025-06-14 devlog, the byte structure for reactive applets are different than for active applets' universal stream
            let query_bytes = {
                let query_type_id = Q::PACKET_TYPE_ID.to_be_bytes();
                let query_inner_data = query.to_data();

                [query_type_id.as_slice(), &query_inner_data].concat()
            };

            let response_bytes = self.query_bytes(&query_bytes);

            // recieve response
            Q::ResponseType::try_from_data(&response_bytes)
        }
    }
}

/// Represents dylib applet client on the server side.
/// Like `ClientHandler` or `UniversalServerSide`
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

    fn event_bytes(&self, event_bytes: &[u8], applet_context: &AppletContext) {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(CBytes, &AppletContext)> =
                self.library.get(b"_event_bytes").unwrap();
            func(CBytes::from(event_bytes), applet_context);
        }
    }
    pub fn send_event<Event: EventPacketTrait>(
        &self,
        event: Event,
        applet_context: &AppletContext,
    ) {
        self.event_bytes(
            &[
                Event::PACKET_TYPE_ID.to_be_bytes().as_slice(),
                &event.to_data(),
            ]
            .concat(),
            applet_context,
        );
    }
    pub fn send_event_union(&self, event: impl EventPacketUnion, applet_context: &AppletContext) {
        let (packet_type_id, packet_inner_data) = event.packet_to_data();

        self.event_bytes(
            &[packet_type_id.to_be_bytes().as_slice(), &packet_inner_data].concat(),
            applet_context,
        );
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
