//! Negative test fixture: Vec crossing FFI boundary.
//!
//! This file should cause a validation error when processed by
//! chimera-adapter-rust, as Vec<T> is a forbidden native Rust type
//! across FFI boundaries.

#[no_mangle]
pub extern "C" fn take_vec(v: Vec<u8>) -> usize {
    // ERROR: Vec is a forbidden native type at FFI boundary
    v.len()
}

#[no_mangle]
pub extern "C" fn return_vec() -> Vec<u8> {
    // ERROR: Vec is a forbidden native type at FFI boundary
    vec![1, 2, 3, 4, 5]
}