const std = @import("std");

//! Zig wrapper library for Rust+Zig conformance fixture.
//!
//! Imports Rust functions via extern declarations and exports wrappers,
//! demonstrating cross-language function calls at the ChimeraIR level.

// Import Rust functions (become ChimeraIR imports with import: prefix)
extern fn rust_add(a: i32, b: i32) i32;
extern fn rust_subtract(a: i32, b: i32) i32;
extern fn rust_multiply(a: i32, b: i32) i32;
extern fn rust_divide(a: i32, b: i32) i32;
extern fn rust_max(a: i32, b: i32) i32;
extern fn rust_min(a: i32, b: i32) i32;
extern fn rust_negate(a: i32) i32;
extern fn rust_is_zero(a: i32) bool;

/// Add two i32 values (delegates to Rust).
export fn zig_add(a: i32, b: i32) i32 {
    return rust_add(a, b);
}

/// Subtract two i32 values (delegates to Rust).
export fn zig_subtract(a: i32, b: i32) i32 {
    return rust_subtract(a, b);
}

/// Multiply two i32 values (delegates to Rust).
export fn zig_multiply(a: i32, b: i32) i32 {
    return rust_multiply(a, b);
}

/// Divide two i32 values (delegates to Rust).
export fn zig_divide(a: i32, b: i32) i32 {
    return rust_divide(a, b);
}

/// Return the maximum of two i32 values (delegates to Rust).
export fn zig_max(a: i32, b: i32) i32 {
    return rust_max(a, b);
}

/// Return the minimum of two i32 values (delegates to Rust).
export fn zig_min(a: i32, b: i32) i32 {
    return rust_min(a, b);
}

/// Negate an i32 value (delegates to Rust).
export fn zig_negate(a: i32) i32 {
    return rust_negate(a);
}

/// Check if an i32 value is zero (delegates to Rust).
export fn zig_is_zero(a: i32) bool {
    return rust_is_zero(a);
}

/// Combined operation: (a + b) * c using Rust functions.
export fn zig_combined_op(a: i32, b: i32, c: i32) i32 {
    const sum = rust_add(a, b);
    return rust_multiply(sum, c);
}

/// Compute a complex expression: max(a, b) + min(a, b)
export fn zig_complex_op(a: i32, b: i32) i32 {
    const maximum = rust_max(a, b);
    const minimum = rust_min(a, b);
    return rust_add(maximum, minimum);
}