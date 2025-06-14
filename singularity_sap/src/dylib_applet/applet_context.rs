use std::ffi::c_void;

use crate::dylib_applet::ffi_bytes::{CBytes, CVec};

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
        dylib_applet::{applet_context::AppletContext, ffi_bytes::CBytes},
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
