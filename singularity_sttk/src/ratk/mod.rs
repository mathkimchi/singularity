//! Reactive Applet ToolKit

use singularity_sap::{
    dylib_applet::{applet_context::AppletContext, ffi_bytes::CBytes},
    packet::EventPacketUnion,
};
use std::ffi::c_void;

/// TODO: different instance event and global event
/// TODO: method for initiating applets?
pub trait ReactiveApplet<Event: EventPacketUnion>: Sized {
    fn handle_event(&mut self, event: Event, applet_context: &AppletContext);

    /// REVIEW: should I just put this entire function and its body inside the `register_applet` macro?
    fn handle_event_bytes(
        applet_instance_bytes: *mut c_void,
        event_bytes: CBytes,
        applet_context: &AppletContext,
    ) {
        if let Some(event) = Event::packet_try_from_typed_data(event_bytes.as_ref()) {
            let applet_instance: &mut Self = unsafe { &mut *(applet_instance_bytes as *mut Self) };
            Self::handle_event(applet_instance, event, applet_context);
        } else {
            println!("Log: Unknown event {:?}!", event_bytes.as_ref());
        }
    }

    /// TODO: rename to handle event
    fn handle_global_event(event: Event, applet_context: &AppletContext);

    /// REVIEW: should I just put this entire function and its body inside the `register_applet` macro?
    fn handle_global_event_bytes(event_bytes: CBytes, applet_context: &AppletContext) {
        if let Some(event) = Event::packet_try_from_typed_data(event_bytes.as_ref()) {
            Self::handle_global_event(event, applet_context);
        } else {
            println!("Log: Unknown event {:?}!", event_bytes.as_ref());
        }
    }

    /// This function will be called to drop the applet.
    fn close_applet(self) {}

    /// REVIEW: should I just put this entire function and its body inside the `register_applet` macro?
    fn close_applet_bytes(applet_instance_bytes: *mut c_void) {
        // get owned self from applet_instance
        let applet_instance: Self = *unsafe { Box::from_raw(applet_instance_bytes as *mut Self) };
        Self::close_applet(applet_instance);
    }
}

#[macro_export]
macro_rules! register_applet {
    ($ApplicationName:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn _dynamic_plugin_signature() -> u64 {
            // NOTE: I manually mashed and copied this
            784532439874536
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _handle_event_bytes(
            applet_instance_bytes: *mut std::ffi::c_void,
            event_bytes: singularity_sap::dylib_applet::ffi_bytes::CBytes,
            applet_context: &AppletContext,
        ) {
            <$ApplicationName>::handle_event_bytes(
                applet_instance_bytes,
                event_bytes,
                applet_context,
            )
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _handle_global_event_bytes(
            event_bytes: singularity_sap::dylib_applet::ffi_bytes::CBytes,
            applet_context: &AppletContext,
        ) {
            <$ApplicationName>::handle_global_event_bytes(event_bytes, applet_context)
        }

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn _close_applet_bytes(applet_instance_bytes: *mut std::ffi::c_void) {
            <$ApplicationName>::close_applet_bytes(applet_instance_bytes)
        }
    };
}
