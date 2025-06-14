#![cfg(feature = "dylib_applet")]

pub mod applet_context;
pub mod dylib_client_handler;
pub mod ffi_bytes;

// use crate::dylib_applet::{applet_context::AppletContext, ffi_bytes::CBytes};

/// "Generated" by mashing keyboard.
pub const DYLIB_APPLET_SIGNATURE: u64 = 784532439874536;
