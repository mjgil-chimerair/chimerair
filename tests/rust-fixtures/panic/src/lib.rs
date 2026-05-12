//! Panic fixture demonstrating panic policy behavior at FFI boundaries.
//!
//! This fixture tests the chimera-rust-effects and compiler-core panic
//! boundary handling: abort, catch_unwind, and extern "C-unwind".

use std::panic::{catch_unwind, AssertUnwindSafe};

/// Status code for panic results.
#[repr(C)]
pub struct PanicStatus {
    pub did_not_panic: bool,
    pub panic_payload: u64,
}

impl PanicStatus {
    pub fn ok() -> Self {
        PanicStatus {
            did_not_panic: true,
            panic_payload: 0,
        }
    }

    pub fn panicked(payload: u64) -> Self {
        PanicStatus {
            did_not_panic: false,
            panic_payload: payload,
        }
    }
}

/// A function that does not panic (safe to call across FFI).
#[no_mangle]
pub extern "C" fn no_panic_function(value: i32) -> i32 {
    value * 2
}

/// A function that may panic - demonstrates may_panic effect.
///
/// Note: This function is marked `extern "C"` but SHOULD NOT be called
/// across an FFI boundary without a catch_unwind wrapper.
#[no_mangle]
pub extern "C" fn may_panic_function(value: i32) -> i32 {
    if value < 0 {
        panic!("negative value not allowed: {}", value);
    }
    value * 3
}

/// Catch panics from a function pointer call.
///
/// This is the recommended pattern for calling potentially-panicking
/// Rust functions from FFI: wrap in catch_unwind and convert to status.
#[no_mangle]
pub extern "C" fn catch_panic_wrapper(
    f: unsafe extern "C" fn(i32) -> i32,
    arg: i32,
) -> PanicStatus {
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: The caller guarantees f is a valid function pointer
        // and arg is a valid argument. The function may panic, which
        // is why we wrap in catch_unwind.
        unsafe { f(arg) }
    }));
    match result {
        Ok(_) => PanicStatus::ok(),
        Err(_) => PanicStatus::panicked(1),
    }
}

/// Wrapper for no_panic_function that can be called safely from FFI.
#[no_mangle]
pub extern "C" fn safe_call_no_panic(value: i32) -> PanicStatus {
    let result = catch_unwind(AssertUnwindSafe(|| no_panic_function(value)));
    match result {
        Ok(_) => PanicStatus::ok(),
        Err(_) => PanicStatus::panicked(1),
    }
}

/// Wrapper for may_panic_function that catches panics.
///
/// This is the safe FFI pattern for calling functions that may panic.
#[no_mangle]
pub extern "C" fn safe_call_may_panic(value: i32) -> PanicStatus {
    let result = catch_unwind(AssertUnwindSafe(|| may_panic_function(value)));
    match result {
        Ok(_) => PanicStatus::ok(),
        Err(_) => PanicStatus::panicked(1),
    }
}

/// A function that catches its own panic and returns a status.
///
/// This demonstrates self-contained panic handling.
#[no_mangle]
pub extern "C" fn self_catching_function(value: i32) -> PanicStatus {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if value < 0 {
            panic!("negative not allowed");
        }
        value
    }));

    match result {
        Ok(v) if v >= 0 => PanicStatus::ok(),
        _ => PanicStatus::panicked(2),
    }
}

/// Check if a panic status indicates no panic occurred.
#[no_mangle]
pub extern "C" fn panic_status_is_ok(status: PanicStatus) -> bool {
    status.did_not_panic
}

/// Get the panic payload from a status.
#[no_mangle]
pub extern "C" fn panic_status_payload(status: PanicStatus) -> u64 {
    status.panic_payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_panic_function_positive() {
        assert_eq!(no_panic_function(5), 10);
    }

    #[test]
    fn test_no_panic_function_zero() {
        assert_eq!(no_panic_function(0), 0);
    }

    #[test]
    fn test_no_panic_function_negative() {
        assert_eq!(no_panic_function(-1), -2);
    }

    #[test]
    fn test_safe_call_no_panic_check_ok() {
        let status = safe_call_no_panic(21);
        assert!(panic_status_is_ok(status));
    }

    #[test]
    fn test_safe_call_no_panic_check_payload() {
        let status = safe_call_no_panic(21);
        assert_eq!(panic_status_payload(status), 0);
    }

    #[test]
    fn test_self_catching_ok() {
        let status = self_catching_function(42);
        assert!(panic_status_is_ok(status));
    }

    #[test]
    fn test_self_catching_negative() {
        let status = self_catching_function(-1);
        assert!(!panic_status_is_ok(status));
    }

    #[test]
    fn test_self_catching_negative_payload() {
        let status = self_catching_function(-1);
        assert_eq!(panic_status_payload(status), 2);
    }
}