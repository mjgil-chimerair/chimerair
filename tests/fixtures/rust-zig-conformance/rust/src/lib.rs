//! Rust math library for Rust+Zig conformance fixture.
//!
//! Exports simple math functions with extern "C" ABI for testing
//! cross-language unified lowering at the ChimeraIR level.

/// Add two i32 values.
#[no_mangle]
pub extern "C" fn rust_add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two i32 values.
#[no_mangle]
pub extern "C" fn rust_subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiply two i32 values.
#[no_mangle]
pub extern "C" fn rust_multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divide two i32 values (panic on divide by zero).
#[no_mangle]
pub extern "C" fn rust_divide(a: i32, b: i32) -> i32 {
    a / b
}

/// Return the maximum of two i32 values.
#[no_mangle]
pub extern "C" fn rust_max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// Return the minimum of two i32 values.
#[no_mangle]
pub extern "C" fn rust_min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

/// Negate an i32 value.
#[no_mangle]
pub extern "C" fn rust_negate(a: i32) -> i32 {
    -a
}

/// Check if an i32 value is zero.
#[no_mangle]
pub extern "C" fn rust_is_zero(a: i32) -> bool {
    a == 0
}