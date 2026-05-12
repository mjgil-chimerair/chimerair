//! FFI fixture demonstrating Result wrapping for error handling.
//!
//! This fixture tests the chimera-rust-abi and chimera-rust-to-chimera
//! crates' handling of Result<T, E> at FFI boundaries.

use std::os::raw::c_int;

/// Custom error type for FFI operations.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FfiError {
    pub code: c_int,
    pub message: [u8; 64],
}

impl FfiError {
    pub fn new(code: c_int, msg: &str) -> Self {
        let mut message = [0u8; 64];
        let bytes = msg.as_bytes();
        let len = bytes.len().min(63);
        message[..len].copy_from_slice(&bytes[..len]);
        FfiError { code, message }
    }

    pub fn code(&self) -> c_int {
        self.code
    }
}

/// Result type used across the FFI boundary.
///
/// When returning this through FFI, Ok(0) means success and
/// Err(error) contains the error information.
#[repr(C)]
pub struct FfiResult {
    pub is_ok: bool,
    pub value: u64,
    pub error: FfiError,
}

/// A safe wrapper that converts FfiResult into a Rust Result.
pub fn interpret_result(result: FfiResult) -> Result<u64, FfiError> {
    if result.is_ok {
        Ok(result.value)
    } else {
        Err(result.error)
    }
}

/// Perform a division operation that can fail.
///
/// Returns FfiResult with Ok(value) on success or Err on failure.
#[no_mangle]
pub extern "C" fn safe_divide(a: u64, b: u64) -> FfiResult {
    if b == 0 {
        FfiResult {
            is_ok: false,
            value: 0,
            error: FfiError::new(1, "division by zero"),
        }
    } else {
        FfiResult {
            is_ok: true,
            value: a / b,
            error: FfiError::new(0, ""),
        }
    }
}

/// Parse a string into a number.
#[no_mangle]
pub extern "C" fn parse_number(s: *const u8, len: usize) -> FfiResult {
    if s.is_null() {
        return FfiResult {
            is_ok: false,
            value: 0,
            error: FfiError::new(2, "null pointer"),
        };
    }

    let slice = unsafe { std::slice::from_raw_parts(s, len) };
    let Ok(s) = std::str::from_utf8(slice) else {
        return FfiResult {
            is_ok: false,
            value: 0,
            error: FfiError::new(3, "invalid utf8"),
        };
    };

    match s.trim().parse::<u64>() {
        Ok(n) => FfiResult {
            is_ok: true,
            value: n,
            error: FfiError::new(0, ""),
        },
        Err(_) => FfiResult {
            is_ok: false,
            value: 0,
            error: FfiError::new(4, "parse error"),
        },
    }
}

/// A nullable pointer wrapper using Option<NonNull<T>> idiom.
#[repr(C)]
pub struct NullablePointer {
    pub pointer: *mut u8,
    pub is_some: bool,
}

impl NullablePointer {
    pub fn new(ptr: *mut u8) -> Self {
        NullablePointer {
            pointer: ptr,
            is_some: !ptr.is_null(),
        }
    }

    pub fn none() -> Self {
        NullablePointer {
            pointer: std::ptr::null_mut(),
            is_some: false,
        }
    }

    pub fn some(ptr: *mut u8) -> Self {
        Self::new(ptr)
    }

    pub fn as_option(&self) -> Option<*mut u8> {
        if self.is_some {
            Some(self.pointer)
        } else {
            None
        }
    }
}

/// Create a nullable pointer from an optional value.
#[no_mangle]
pub extern "C" fn make_nullable(value: *mut u8) -> NullablePointer {
    NullablePointer::new(value)
}

/// Get the value from a nullable pointer or return a default.
#[no_mangle]
pub extern "C" fn get_or_default(ptr: NullablePointer, default: u8) -> u8 {
    if ptr.is_some {
        unsafe { *ptr.pointer }
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_divide_success() {
        let result = safe_divide(10, 2);
        assert!(result.is_ok);
        assert_eq!(result.value, 5);
    }

    #[test]
    fn test_safe_divide_by_zero() {
        let result = safe_divide(10, 0);
        assert!(!result.is_ok);
        assert_eq!(result.error.code(), 1);
    }

    #[test]
    fn test_interpret_result() {
        let ok_result = FfiResult {
            is_ok: true,
            value: 42,
            error: FfiError::new(0, ""),
        };
        assert_eq!(interpret_result(ok_result).unwrap(), 42);

        let err_result = FfiResult {
            is_ok: false,
            value: 0,
            error: FfiError::new(5, "test error"),
        };
        assert!(interpret_result(err_result).is_err());
    }

    #[test]
    fn test_nullable_pointer() {
        let ptr = NullablePointer::new(0x1234 as *mut u8);
        assert!(ptr.is_some);

        let none = NullablePointer::none();
        assert!(!none.is_some);
    }
}