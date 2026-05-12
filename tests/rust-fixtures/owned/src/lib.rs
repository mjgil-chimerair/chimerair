//! Ownership fixture demonstrating Box<T> and owned bytes with drop trampolines.
//!
//! This fixture tests the chimera-rust-ownership crate's handling of owned
//! handles, bytes, and allocator metadata at FFI boundaries.

use std::alloc::{alloc, dealloc, Layout};

/// A handle descriptor for FFI boundary passing with drop trampoline.
///
/// This represents the chimera-rust runtime ChHandle type:
/// - ptr: pointer to allocated memory
/// - size: size of allocation
/// - drop_trampoline: function to call for cleanup
#[repr(C)]
pub struct ChHandle {
    pub ptr: *mut u8,
    pub size: usize,
    pub drop_trampoline: Option<unsafe fn(*mut u8, usize)>,
}

/// A owned bytes descriptor for FFI boundary passing.
///
/// This represents the chimera-rust runtime ChOwnedBytes type:
/// - ptr: pointer to data
/// - len: number of bytes
/// - capacity: total capacity
#[repr(C)]
pub struct ChOwnedBytes {
    pub ptr: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl ChOwnedBytes {
    pub fn new(data: &[u8]) -> Self {
        let layout = Layout::array::<u8>(data.len()).unwrap();
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return ChOwnedBytes {
                ptr: std::ptr::null_mut(),
                len: 0,
                capacity: 0,
            };
        }
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        }
        ChOwnedBytes {
            ptr,
            len: data.len(),
            capacity: data.len(),
        }
    }

    pub fn empty() -> Self {
        ChOwnedBytes {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub unsafe fn free(&mut self) {
        if !self.ptr.is_null() {
            let layout = Layout::array::<u8>(self.capacity).unwrap();
            dealloc(self.ptr, layout);
            self.ptr = std::ptr::null_mut();
            self.len = 0;
            self.capacity = 0;
        }
    }
}

/// Drop trampoline for ChHandle - called when the handle is no longer needed.
unsafe fn handle_drop_trampoline(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 8).unwrap();
    dealloc(ptr, layout);
}

/// Create a new handle with the given size.
#[no_mangle]
pub extern "C" fn handle_create(size: usize) -> ChHandle {
    let layout = Layout::from_size_align(size, 8).unwrap();
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        return ChHandle {
            ptr: std::ptr::null_mut(),
            size: 0,
            drop_trampoline: None,
        };
    }
    ChHandle {
        ptr,
        size,
        drop_trampoline: Some(handle_drop_trampoline),
    }
}

/// Drop a handle using its trampoline.
#[no_mangle]
pub extern "C" fn handle_drop(handle: ChHandle) {
    let ptr = handle.ptr;
    let size = handle.size;
    if let Some(trampoline) = handle.drop_trampoline {
        unsafe { trampoline(ptr, size) };
    }
}

/// Get the size of a handle.
#[no_mangle]
pub extern "C" fn handle_size(handle: ChHandle) -> usize {
    handle.size
}

/// Check if a handle is valid (non-null).
#[no_mangle]
pub extern "C" fn handle_is_valid(handle: ChHandle) -> bool {
    !handle.ptr.is_null()
}

/// Create owned bytes from a pointer and length.
///
/// Note: This takes ownership - the caller should not use the original pointer.
#[no_mangle]
pub extern "C" fn owned_bytes_from_ptr(ptr: *mut u8, len: usize, capacity: usize) -> ChOwnedBytes {
    ChOwnedBytes { ptr, len, capacity }
}

/// Get the length of owned bytes.
#[no_mangle]
pub extern "C" fn owned_bytes_len(bytes: ChOwnedBytes) -> usize {
    bytes.len
}

/// Get the capacity of owned bytes.
#[no_mangle]
pub extern "C" fn owned_bytes_capacity(bytes: ChOwnedBytes) -> usize {
    bytes.capacity
}

/// Check if owned bytes is empty.
#[no_mangle]
pub extern "C" fn owned_bytes_is_empty(bytes: ChOwnedBytes) -> bool {
    bytes.len == 0
}

/// Read a byte from owned bytes at the given index.
#[no_mangle]
pub extern "C" fn owned_bytes_get(bytes: ChOwnedBytes, index: usize) -> u8 {
    if index >= bytes.len {
        return 0;
    }
    unsafe { *bytes.ptr.add(index) }
}

/// Create a new owned bytes from C string data.
#[no_mangle]
pub extern "C" fn owned_bytes_from_c_string(ptr: *const u8, len: usize) -> ChOwnedBytes {
    if ptr.is_null() {
        return ChOwnedBytes::empty();
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    ChOwnedBytes::new(slice)
}

/// Free owned bytes (call when done).
#[no_mangle]
pub extern "C" fn owned_bytes_free(mut bytes: ChOwnedBytes) {
    unsafe { bytes.free() };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_create_valid() {
        let handle = handle_create(64);
        assert!(handle_is_valid(handle));
    }

    #[test]
    fn test_handle_size() {
        let handle = handle_create(128);
        assert_eq!(handle_size(handle), 128);
    }

    #[test]
    fn test_handle_drop() {
        let handle = handle_create(64);
        handle_drop(handle);
    }

    #[test]
    fn test_handle_invalid() {
        let handle = ChHandle {
            ptr: std::ptr::null_mut(),
            size: 0,
            drop_trampoline: None,
        };
        assert!(!handle_is_valid(handle));
    }

    #[test]
    fn test_owned_bytes_len() {
        let mut data = [1u8, 2, 3, 4, 5];
        let bytes = owned_bytes_from_ptr(data.as_mut_ptr(), 5, 5);
        assert_eq!(owned_bytes_len(bytes), 5);
    }

    #[test]
    fn test_owned_bytes_capacity() {
        let mut data = [1u8, 2, 3, 4, 5];
        let bytes = owned_bytes_from_ptr(data.as_mut_ptr(), 5, 5);
        assert_eq!(owned_bytes_capacity(bytes), 5);
    }

    #[test]
    fn test_owned_bytes_not_empty() {
        let mut data = [1u8, 2, 3, 4, 5];
        let bytes = owned_bytes_from_ptr(data.as_mut_ptr(), 5, 5);
        assert!(!owned_bytes_is_empty(bytes));
    }

    #[test]
    fn test_owned_bytes_get_0() {
        let data = [10u8, 20, 30, 40, 50];
        let bytes = owned_bytes_from_ptr(data.as_ptr() as *mut u8, 5, 5);
        assert_eq!(owned_bytes_get(bytes, 0), 10);
    }

    #[test]
    fn test_owned_bytes_get_2() {
        let data = [10u8, 20, 30, 40, 50];
        let bytes = owned_bytes_from_ptr(data.as_ptr() as *mut u8, 5, 5);
        assert_eq!(owned_bytes_get(bytes, 2), 30);
    }

    #[test]
    fn test_owned_bytes_get_4() {
        let data = [10u8, 20, 30, 40, 50];
        let bytes = owned_bytes_from_ptr(data.as_ptr() as *mut u8, 5, 5);
        assert_eq!(owned_bytes_get(bytes, 4), 50);
    }

    #[test]
    fn test_owned_bytes_get_oob() {
        let data = [10u8, 20, 30, 40, 50];
        let bytes = owned_bytes_from_ptr(data.as_ptr() as *mut u8, 5, 5);
        assert_eq!(owned_bytes_get(bytes, 5), 0);
    }

    #[test]
    fn test_owned_bytes_empty() {
        let bytes = owned_bytes_from_c_string(std::ptr::null(), 0);
        assert!(owned_bytes_is_empty(bytes));
    }

    #[test]
    fn test_owned_bytes_new() {
        let data = [1u8, 2, 3, 4, 5];
        let mut bytes = ChOwnedBytes::new(&data);
        // Test that we can call free on it
        unsafe { bytes.free(); }
    }
}