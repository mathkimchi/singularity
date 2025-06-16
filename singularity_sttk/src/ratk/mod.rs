//! Reactive Applet ToolKit

use singularity_sap::{
    dylib_applet::{applet_context::AppletContext, ffi_bytes::CBytes},
    packet::EventPacketUnion,
};

pub trait ReactiveApplet<Event: EventPacketUnion> {
    fn recieve_event(event: Event, applet_context: &AppletContext);

    /// REVIEW: should I just put this entire function and its body inside the `register_applet` macro?
    fn recieve_event_bytes(event_bytes: CBytes, applet_context: &AppletContext) {
        if let Some(event) = Event::packet_try_from_typed_data(event_bytes.as_ref()) {
            Self::recieve_event(event, applet_context);
        } else {
            println!("Log: Unknown event {:?}!", event_bytes.as_ref());
        }
    }
}

#[macro_export]
macro_rules! register_applet {
    ($ApplicationName:ty) => {
        #[no_mangle]
        pub extern "C" fn _dynamic_plugin_signature() -> u64 {
            // NOTE: I manually mashed and copied this
            784532439874536
        }

        #[no_mangle]
        pub unsafe extern "C" fn _recieve_event_bytes(
            event_bytes: CBytes,
            applet_context: &AppletContext,
        ) {
            $ApplicationName::recieve_event_bytes(event_bytes, applet_context)
        }
    };
}
