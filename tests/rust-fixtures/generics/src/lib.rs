//! Generics fixture demonstrating monomorphization at FFI boundaries.
//!
//! This fixture tests the chimera-rust-to-chimera crate's handling
//! of generic functions when lowered to ChimeraIR: monomorphization
//! produces concrete types from generic originals.

/// Result of a generic identity operation.
#[repr(C)]
pub struct IdentityResult {
    pub value: u64,
    pub type_tag: u8,
}

/// Type tag constants for monomorphized types.
pub const TYPE_TAG_I64: u8 = 1;
pub const TYPE_TAG_U64: u8 = 2;
pub const TYPE_TAG_I32: u8 = 3;
pub const TYPE_TAG_U32: u8 = 4;

/// Generic identity function - returns the same value.
fn generic_identity<T: Copy>(value: T) -> T {
    value
}

/// Monomorphized identity for i64.
#[no_mangle]
pub extern "C" fn identity_i64(value: i64) -> IdentityResult {
    IdentityResult {
        value: value as u64,
        type_tag: TYPE_TAG_I64,
    }
}

/// Monomorphized identity for u64.
#[no_mangle]
pub extern "C" fn identity_u64(value: u64) -> IdentityResult {
    IdentityResult {
        value,
        type_tag: TYPE_TAG_U64,
    }
}

/// Monomorphized identity for i32 (truncated to u64).
#[no_mangle]
pub extern "C" fn identity_i32(value: i32) -> IdentityResult {
    IdentityResult {
        value: (value as i64).wrapping_sub(i64::MIN) as u64,
        type_tag: TYPE_TAG_I32,
    }
}

/// Monomorphized identity for u32 (truncated to u64).
#[no_mangle]
pub extern "C" fn identity_u32(value: u32) -> IdentityResult {
    IdentityResult {
        value: value as u64,
        type_tag: TYPE_TAG_U32,
    }
}

/// Generic pair structure.
#[repr(C)]
pub struct GenericPair {
    pub first: u64,
    pub second: u64,
    pub type_tag: u8,
}

impl GenericPair {
    pub fn new(first: u64, second: u64, tag: u8) -> Self {
        GenericPair {
            first,
            second,
            type_tag: tag,
        }
    }

    pub fn first(&self) -> u64 {
        self.first
    }

    pub fn second(&self) -> u64 {
        self.second
    }
}

/// Swap a generic pair.
fn generic_swap<T: Copy, U: Copy>(a: T, b: U) -> (U, T) {
    (b, a)
}

/// Monomorphized swap for (u64, u64).
#[no_mangle]
pub extern "C" fn swap_u64_u64(a: u64, b: u64) -> GenericPair {
    let (b_out, a_out) = generic_swap(a, b);
    GenericPair::new(b_out, a_out, TYPE_TAG_U64)
}

/// Monomorphized swap for (i64, u64).
#[no_mangle]
pub extern "C" fn swap_i64_u64(a: i64, b: u64) -> GenericPair {
    let (b_out, a_out) = generic_swap(a, b);
    GenericPair::new(b_out as u64, a_out as u64, TYPE_TAG_U64)
}

/// Monomorphized swap for (u32, u32).
#[no_mangle]
pub extern "C" fn swap_u32_u32(a: u32, b: u32) -> GenericPair {
    let (b_out, a_out) = generic_swap(a, b);
    GenericPair::new(b_out as u64, a_out as u64, TYPE_TAG_U32)
}

/// A generic container that holds one value.
#[repr(C)]
pub struct GenericContainer {
    pub value: u64,
    pub is_some: bool,
    pub type_tag: u8,
}

impl GenericContainer {
    pub fn some_i64(value: i64) -> Self {
        GenericContainer {
            value: value as u64,
            is_some: true,
            type_tag: TYPE_TAG_I64,
        }
    }

    pub fn some_u64(value: u64) -> Self {
        GenericContainer {
            value,
            is_some: true,
            type_tag: TYPE_TAG_U64,
        }
    }

    pub fn none() -> Self {
        GenericContainer {
            value: 0,
            is_some: false,
            type_tag: 0,
        }
    }

    pub fn unwrap(&self) -> u64 {
        self.value
    }

    pub fn is_some(&self) -> bool {
        self.is_some
    }
}

/// Container wrapping i64.
#[no_mangle]
pub extern "C" fn container_i64_new(value: i64) -> GenericContainer {
    GenericContainer::some_i64(value)
}

/// Container wrapping u64.
#[no_mangle]
pub extern "C" fn container_u64_new(value: u64) -> GenericContainer {
    GenericContainer::some_u64(value)
}

/// Generic Option wrapper - None variant.
#[no_mangle]
pub extern "C" fn container_none() -> GenericContainer {
    GenericContainer::none()
}

/// Check if two generic containers are equal.
fn generic_eq<T: PartialEq>(a: T, b: T) -> bool {
    a == b
}

/// Compare two i64 containers.
#[no_mangle]
pub extern "C" fn container_i64_eq(a: GenericContainer, b: GenericContainer) -> bool {
    if !a.is_some || !b.is_some || a.type_tag != TYPE_TAG_I64 || b.type_tag != TYPE_TAG_I64 {
        return false;
    }
    a.value == b.value
}

/// Compare two u64 containers.
#[no_mangle]
pub extern "C" fn container_u64_eq(a: GenericContainer, b: GenericContainer) -> bool {
    if !a.is_some || !b.is_some || a.type_tag != TYPE_TAG_U64 || b.type_tag != TYPE_TAG_U64 {
        return false;
    }
    a.value == b.value
}

/// Generic maximum operation.
fn generic_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

/// Maximum of two i64.
#[no_mangle]
pub extern "C" fn max_i64(a: i64, b: i64) -> i64 {
    generic_max(a, b)
}

/// Maximum of two u64.
#[no_mangle]
pub extern "C" fn max_u64(a: u64, b: u64) -> u64 {
    generic_max(a, b)
}

/// Maximum of two i32.
#[no_mangle]
pub extern "C" fn max_i32(a: i32, b: i32) -> i32 {
    generic_max(a, b)
}

/// Maximum of two u32.
#[no_mangle]
pub extern "C" fn max_u32(a: u32, b: u32) -> u32 {
    generic_max(a, b)
}

/// Generic minimum operation.
fn generic_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// Minimum of two i64.
#[no_mangle]
pub extern "C" fn min_i64(a: i64, b: i64) -> i64 {
    generic_min(a, b)
}

/// Minimum of two u64.
#[no_mangle]
pub extern "C" fn min_u64(a: u64, b: u64) -> u64 {
    generic_min(a, b)
}

/// Generic array creation with default values.
fn generic_array<T: Default + Copy>(len: usize) -> Vec<T> {
    vec![T::default(); len]
}

/// Create an array of i64 with given length, all zeros.
#[no_mangle]
pub extern "C" fn array_i64_zeros(len: usize) -> *mut u64 {
    let arr: Vec<i64> = generic_array(len);
    let mut boxed: Vec<i64> = arr.into();
    boxed.shrink_to_fit();
    let ptr = boxed.as_mut_ptr() as *mut u64;
    std::mem::forget(boxed);
    ptr
}

/// Create an array of u64 with given length, all zeros.
#[no_mangle]
pub extern "C" fn array_u64_zeros(len: usize) -> *mut u64 {
    let arr: Vec<u64> = generic_array(len);
    let mut boxed: Vec<u64> = arr.into();
    boxed.shrink_to_fit();
    let ptr = boxed.as_mut_ptr();
    std::mem::forget(boxed);
    ptr
}

/// Get element at index from i64 array.
#[no_mangle]
pub unsafe extern "C" fn array_i64_get(ptr: *mut u64, len: usize, index: usize) -> u64 {
    if ptr.is_null() || index >= len {
        return 0;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    slice[index] as u64
}

/// Get element at index from u64 array.
#[no_mangle]
pub unsafe extern "C" fn array_u64_get(ptr: *mut u64, len: usize, index: usize) -> u64 {
    if ptr.is_null() || index >= len {
        return 0;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    slice[index]
}

/// Free i64 array.
#[no_mangle]
pub unsafe extern "C" fn array_i64_free(ptr: *mut u64, len: usize) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr as *mut i64, len, len);
    }
}

/// Free u64 array.
#[no_mangle]
pub unsafe extern "C" fn array_u64_free(ptr: *mut u64, len: usize) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_i64() {
        let result = identity_i64(42);
        assert_eq!(result.value, 42);
        assert_eq!(result.type_tag, TYPE_TAG_I64);
    }

    #[test]
    fn test_identity_u64() {
        let result = identity_u64(100);
        assert_eq!(result.value, 100);
        assert_eq!(result.type_tag, TYPE_TAG_U64);
    }

    #[test]
    fn test_swap_u64_u64() {
        let pair = swap_u64_u64(10, 20);
        assert_eq!(pair.first, 20);
        assert_eq!(pair.second, 10);
    }

    #[test]
    fn test_swap_i64_u64() {
        let pair = swap_i64_u64(-5, 100);
        assert_eq!(pair.first, 100);
        assert_eq!(pair.second, u64::MAX - 4);
    }

    #[test]
    fn test_container_i64_new() {
        let container = container_i64_new(42);
        assert!(container.is_some());
        assert_eq!(container.type_tag, TYPE_TAG_I64);
    }

    #[test]
    fn test_container_u64_new() {
        let container = container_u64_new(42);
        assert!(container.is_some());
        assert_eq!(container.type_tag, TYPE_TAG_U64);
    }

    #[test]
    fn test_container_none() {
        let container = container_none();
        assert!(!container.is_some());
    }

    #[test]
    fn test_container_i64_eq_true() {
        let a = container_i64_new(42);
        let b = container_i64_new(42);
        assert!(container_i64_eq(a, b));
    }

    #[test]
    fn test_container_i64_eq_false() {
        let a = container_i64_new(42);
        let b = container_i64_new(100);
        assert!(!container_i64_eq(a, b));
    }

    #[test]
    fn test_container_u64_eq_true() {
        let a = container_u64_new(42);
        let b = container_u64_new(42);
        assert!(container_u64_eq(a, b));
    }

    #[test]
    fn test_max_i64() {
        assert_eq!(max_i64(10, 20), 20);
        assert_eq!(max_i64(20, 10), 20);
        assert_eq!(max_i64(-5, 5), 5);
    }

    #[test]
    fn test_max_u64() {
        assert_eq!(max_u64(10, 20), 20);
        assert_eq!(max_u64(20, 10), 20);
    }

    #[test]
    fn test_min_i64() {
        assert_eq!(min_i64(10, 20), 10);
        assert_eq!(min_i64(20, 10), 10);
    }

    #[test]
    fn test_min_u64() {
        assert_eq!(min_u64(10, 20), 10);
        assert_eq!(min_u64(20, 10), 10);
    }

    #[test]
    fn test_array_u64_zeros() {
        let len = 5;
        let ptr = array_u64_zeros(len);
        let result = unsafe { array_u64_get(ptr, len, 0) };
        assert_eq!(result, 0);
        unsafe { array_u64_free(ptr, len); }
    }

    #[test]
    fn test_array_u64_get_set() {
        let len = 3;
        let ptr = array_u64_zeros(len);
        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr, len);
            slice[1] = 99;
            let val = array_u64_get(ptr, len, 1);
            assert_eq!(val, 99);
            array_u64_free(ptr, len);
        }
    }
}