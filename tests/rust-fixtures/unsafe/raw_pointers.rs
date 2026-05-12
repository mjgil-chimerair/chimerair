// Test fixture: raw_pointers.rs - unsafe code testing
// @verify chimera-rust-proof
// @expected ch_status SUCCESS

use std::ptr;

pub struct RawBuffer {
    ptr: *mut u8,
    len: usize,
    cap: usize,
}

unsafe impl Send for RawBuffer {}
unsafe impl Sync for RawBuffer {}

impl RawBuffer {
    pub unsafe fn from_ptr(ptr: *mut u8, len: usize, cap: usize) -> Self {
        RawBuffer { ptr, len, cap }
    }

    pub unsafe fn get(&self, index: usize) -> Option<u8> {
        if index < self.len {
            Some(*self.ptr.add(index))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

// Safety: caller must ensure ptr is valid for 'len' elements
pub unsafe fn sum_ptr(ptr: *const u8, len: usize) -> u64 {
    let slice = std::slice::from_raw_parts(ptr, len);
    slice.iter().map(|&b| b as u64).sum()
}
