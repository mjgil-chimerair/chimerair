//! Minimal Rust fixture for unified lowering to ChimeraIR.
//!
//! This fixture exports simple `#[no_mangle]` functions with extern "C" ABI
//! for testing the Rust-to-ChimeraIR lowering path without native archive emission.

/// Add two i32 values.
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Subtract two i32 values.
#[no_mangle]
pub extern "C" fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

/// Multiply two i32 values.
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divide two i32 values (panic on divide by zero).
#[no_mangle]
pub extern "C" fn divide(a: i32, b: i32) -> i32 {
    a / b
}

/// Return the maximum of two i32 values.
#[no_mangle]
pub extern "C" fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

/// Return the minimum of two i32 values.
#[no_mangle]
pub extern "C" fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

/// Negate an i32 value.
#[no_mangle]
pub extern "C" fn negate(a: i32) -> i32 {
    -a
}

/// Check if a i32 value is zero.
#[no_mangle]
pub extern "C" fn is_zero(a: i32) -> bool {
    a == 0
}

/// Return constant zero.
#[no_mangle]
pub static ZERO: i32 = 0;

/// Return constant one.
#[no_mangle]
pub static ONE: i32 = 1;

/// A simple struct for testing struct lowering.
#[repr(C)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

/// Calculate distance from origin (sqrt(x*x + y*y)).
#[no_mangle]
pub extern "C" fn point_distance(p: Point2D) -> f32 {
    ((p.x * p.x + p.y * p.y) as f32).sqrt()
}

/// Create a point at origin.
#[no_mangle]
pub extern "C" fn point_origin() -> Point2D {
    Point2D { x: 0, y: 0 }
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
    fn test_subtract() {
        assert_eq!(subtract(5, 3), 2);
        assert_eq!(subtract(1, 1), 0);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(3, 4), 12);
        assert_eq!(multiply(-2, 5), -10);
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2), 5);
        assert_eq!(divide(7, 2), 3);
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

    #[test]
    fn test_is_zero() {
        assert!(is_zero(0));
        assert!(!is_zero(1));
        assert!(!is_zero(-1));
    }

    #[test]
    fn test_point_distance() {
        let p = Point2D { x: 3, y: 4 };
        assert_eq!(point_distance(p), 5.0);
    }

    #[test]
    fn test_point_origin() {
        let p = point_origin();
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }
}