//! Basic smoke test fixture for Rust FFI.
//!
//! This fixture exports simple `#[no_mangle]` functions for testing
//! the chimera-rust-source parser and schema definitions.

/// Add two i32 values.
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two i32 values.
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Return the larger of two i32 values.
#[no_mangle]
pub extern "C" fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// Return the smaller of two i32 values.
#[no_mangle]
pub extern "C" fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

/// Negate an i32 value.
#[no_mangle]
pub extern "C" fn negate(a: i32) -> i32 {
    -a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(-2, 5), -10);
    }

    #[test]
    fn test_max() {
        assert_eq!(max(1, 2), 2);
        assert_eq!(max(-5, -3), -3);
    }

    #[test]
    fn test_min() {
        assert_eq!(min(1, 2), 1);
        assert_eq!(min(-5, -3), -5);
    }

    #[test]
    fn test_negate() {
        assert_eq!(negate(5), -5);
        assert_eq!(negate(-3), 3);
        assert_eq!(negate(0), 0);
    }
}