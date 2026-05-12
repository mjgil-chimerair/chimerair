//! Negative test fixture: Result crossing FFI boundary.
//!
//! This file should cause a validation error when processed by
//! chimera-adapter-rust, as native Result<T, E> should not cross
//! the FFI boundary directly.

#[no_mangle]
pub extern "C" fn might_fail() -> Result<u32, &'static str> {
    // ERROR: Result is a forbidden native type at FFI boundary
    // unless lowered to ch_status convention
    Ok(42)
}

#[no_mangle]
pub extern "C" fn use_result(r: Result<i64, ()>) -> i64 {
    // ERROR: Result parameter at FFI boundary is not allowed
    r.unwrap_or(-1)
}