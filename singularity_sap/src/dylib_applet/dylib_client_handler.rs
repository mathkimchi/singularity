#![cfg(feature = "server")]

use crate::{
    dylib_applet::{applet_context::AppletContext, ffi_bytes::CBytes, DYLIB_APPLET_SIGNATURE},
    packet::{EventPacketTrait, EventPacketUnion},
};
use std::{ffi::OsStr, path::Path};

/// Represents dylib applet client on the server side.
/// Like `ClientHandler` or `UniversalServerSide`
pub struct DylibClientHandler {
    library: libloading::Library,
}
impl DylibClientHandler {
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

    fn event_bytes(&self, event_bytes: &[u8], applet_context: &AppletContext) {
        unsafe {
            // REVIEW: check if the unsafe is necessary for the symbol
            let func: libloading::Symbol<unsafe extern "C" fn(CBytes, &AppletContext)> =
                self.library.get(b"_recieve_event_bytes").unwrap();
            func(CBytes::from(event_bytes), applet_context);
        }
    }
    pub fn send_event<Event: EventPacketTrait>(
        &self,
        event: Event,
        applet_context: &AppletContext,
    ) {
        self.event_bytes(&event.packet_to_typed_data(), applet_context);
    }
    pub fn send_event_union(&self, event: impl EventPacketUnion, applet_context: &AppletContext) {
        self.event_bytes(&event.packet_to_typed_data(), applet_context);
    }
}
