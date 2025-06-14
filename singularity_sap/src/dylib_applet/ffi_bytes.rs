//! REVIEW: kinda wonky name, I also call the structs `CBytes`
//! but I actually assume both sides of FFI are going to be in Rust and this exact crate.
//! TODO: rename to FFIBytes

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
