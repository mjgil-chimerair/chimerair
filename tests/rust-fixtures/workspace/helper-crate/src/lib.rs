//! Helper crate for workspace fixture.

/// Add two numbers.
#[no_mangle]
pub extern "C" fn helper_add(a: i32, b: i32) -> i32 {
    a + b
}

/// Multiply two numbers.
#[no_mangle]
pub extern "C" fn helper_mul(a: i32, b: i32) -> i32 {
    a * b
}

/// Negate a value.
#[no_mangle]
pub extern "C" fn helper_neg(a: i32) -> i32 {
    -a
}

/// Feature flag check.
#[no_mangle]
pub extern "C" fn get_feature_flags() -> u32 {
    let flags: u32 = 0;
    #[cfg(feature = "std")]
    {
        flags |= 1;
    }
    #[cfg(feature = "nightly")]
    {
        flags |= 2;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_add() {
        assert_eq!(helper_add(2, 3), 5);
    }

    #[test]
    fn test_helper_mul() {
        assert_eq!(helper_mul(3, 4), 12);
    }

    #[test]
    fn test_helper_neg() {
        assert_eq!(helper_neg(5), -5);
    }

    #[test]
    fn test_get_feature_flags_default() {
        let flags = get_feature_flags();
        // With no features enabled, flags should be 0
        assert_eq!(flags, 0);
    }

    #[test]
    fn test_helper_add_zero() {
        assert_eq!(helper_add(0, 0), 0);
    }

    #[test]
    fn test_helper_mul_zero() {
        assert_eq!(helper_mul(0, 99), 0);
    }
}