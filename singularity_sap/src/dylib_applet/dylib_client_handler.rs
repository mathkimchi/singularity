#![cfg(feature = "server")]

use crate::{
    dylib_applet::{DYLIB_APPLET_SIGNATURE, applet_context::AppletContext, ffi_bytes::CBytes},
    packet::{EventPacketTrait, EventPacketUnion},
};
use std::{ffi::OsStr, path::Path};

/// Represents dylib applet client on the server side.
/// Like `ClientHandler` or `UniversalServerSide`
// pub struct DylibClientHandler {
pub struct DylibAppletLibrary {
    library: libloading::Library,
    // handle_event_bytes_fn: libloading::Symbol<unsafe extern "C" fn(CBytes, &AppletContext)>,
}
impl DylibAppletLibrary {
    /// Credit: [`dynamic-plugin`](https://github.com/lilopkins/dynamic-plugins-rs)
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
    /// Credit: [`dynamic-plugin`](https://github.com/lilopkins/dynamic-plugins-rs)
    // TODO: use result instead of option ; this kind of goes for the entire codebase. Ctrl+F `ok()?`
    pub fn load_plugin<P>(path: P) -> Option<Self>
    where
        P: AsRef<OsStr>,
    {
        unsafe {
            let library = libloading::Library::new(path).ok()?;
            // REVIEW: check if the unsafe is necessary for the symbol
            let dylib_applet_signature: libloading::Symbol<unsafe extern "C" fn() -> u64> =
                library.get(b"_get_applet_signature").ok()?;

            let hash = dylib_applet_signature();
            if hash != DYLIB_APPLET_SIGNATURE {
                return None;
            }

            Some(Self { library })
        }
    }

    fn send_event_bytes(
        &self,
        applet_instance_bytes: *mut std::ffi::c_void,
        event_bytes: &[u8],
        applet_context: &AppletContext,
    ) {
        unsafe {
            // REVIEW: check if the unsafe is necessary for the symbol
            let func: libloading::Symbol<
                unsafe extern "C" fn(*mut std::ffi::c_void, CBytes, &AppletContext),
            > = self.library.get(b"_handle_event_bytes").unwrap();
            func(
                applet_instance_bytes,
                CBytes::from(event_bytes),
                applet_context,
            );
        }
    }
    pub fn send_event<Event: EventPacketTrait>(
        &self,
        applet_instance_bytes: *mut std::ffi::c_void,
        event: Event,
        applet_context: &AppletContext,
    ) {
        self.send_event_bytes(
            applet_instance_bytes,
            &event.packet_to_typed_data(),
            applet_context,
        );
    }
    pub fn send_event_union(
        &self,
        applet_instance_bytes: *mut std::ffi::c_void,
        event: impl EventPacketUnion,
        applet_context: &AppletContext,
    ) {
        self.send_event_bytes(
            applet_instance_bytes,
            &event.packet_to_typed_data(),
            applet_context,
        );
    }

    pub fn drop_applet_context(&self, applet_instance_bytes: *mut std::ffi::c_void) {
        unsafe {
            // REVIEW: check if the unsafe is necessary for the symbol
            let func: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> =
                self.library.get(b"_handle_event_bytes").unwrap();
            func(applet_instance_bytes);
        }
    }
}
