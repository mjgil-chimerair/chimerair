//! Slice/String fixture demonstrating ch_slice and ch_borrow_str wrapper types.
//!
//! This fixture tests the chimera-rust-to-chimera crate's handling of slices
//! and strings at FFI boundaries using the ch_slice/ch_borrow_str conventions.

/// A slice descriptor for FFI boundary passing.
///
/// This represents the chimera-rust runtime ChSlice type:
/// - data: pointer to slice data
/// - len: number of elements
#[repr(C)]
pub struct ChSlice {
    pub data: *const u8,
    pub len: usize,
}

impl ChSlice {
    pub unsafe fn from_ref(s: &[u8]) -> Self {
        ChSlice {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    pub unsafe fn as_slice(&self) -> &[u8] {
        std::slice::from_raw_parts(self.data, self.len)
    }

    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        std::slice::from_raw_parts_mut(self.data as *mut u8, self.len)
    }
}

/// A borrowed string descriptor for FFI boundary passing.
///
/// This represents the chimera-rust runtime ChBorrowStr type:
/// - data: pointer to UTF-8 string data (not null-terminated)
/// - len: number of bytes (not including null terminator)
#[repr(C)]
pub struct ChBorrowStr {
    pub data: *const u8,
    pub len: usize,
}

impl ChBorrowStr {
    pub unsafe fn from_str(s: &str) -> Self {
        ChBorrowStr {
            data: s.as_ptr(),
            len: s.len(),
        }
    }

    pub unsafe fn as_str(&self) -> &str {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.len))
    }
}

/// A result type for slice operations that can fail.
#[repr(C)]
pub struct ChSliceResult {
    pub is_ok: bool,
    pub slice: ChSlice,
    pub error_code: u32,
}

impl ChSliceResult {
    pub fn ok(slice: ChSlice) -> Self {
        ChSliceResult {
            is_ok: true,
            slice,
            error_code: 0,
        }
    }

    pub fn err(error_code: u32) -> Self {
        ChSliceResult {
            is_ok: false,
            slice: ChSlice { data: std::ptr::null(), len: 0 },
            error_code,
        }
    }
}

/// Calculate sum of bytes in a slice.
#[no_mangle]
pub extern "C" fn sum_bytes(slice: ChSlice) -> u64 {
    let bytes = unsafe { slice.as_slice() };
    bytes.iter().map(|&b| b as u64).sum()
}

/// Count occurrences of a byte in a slice.
#[no_mangle]
pub extern "C" fn count_byte(slice: ChSlice, byte: u8) -> usize {
    let bytes = unsafe { slice.as_slice() };
    bytes.iter().filter(|&&b| b == byte).count()
}

/// Check if slice starts with a given prefix.
#[no_mangle]
pub extern "C" fn slice_starts_with(slice: ChSlice, prefix: ChSlice) -> bool {
    let bytes = unsafe { slice.as_slice() };
    let pref = unsafe { prefix.as_slice() };
    bytes.starts_with(pref)
}

/// Check if slice ends with a given suffix.
#[no_mangle]
pub extern "C" fn slice_ends_with(slice: ChSlice, suffix: ChSlice) -> bool {
    let bytes = unsafe { slice.as_slice() };
    let suff = unsafe { suffix.as_slice() };
    bytes.ends_with(suff)
}

/// Get string length (for testing ChBorrowStr).
#[no_mangle]
pub extern "C" fn string_length(s: ChBorrowStr) -> usize {
    unsafe { s.as_str().len() }
}

/// Reverse a slice and return result through out pointer (real FFI pattern).
/// The reversed data is written to the provided output slice.
///
/// Real FFI pattern: caller provides output buffer, callee writes to it.
/// This demonstrates the ch_slice FFI convention.
#[no_mangle]
pub extern "C" fn slice_reverse_into(
    input: ChSlice,
    mut output: ChSlice,
    output_capacity: usize,
) -> bool {
    let in_bytes = unsafe { input.as_slice() };
    let out_bytes = unsafe { output.as_mut_slice() };

    if out_bytes.len() < in_bytes.len() || output_capacity < in_bytes.len() {
        return false;
    }

    for (i, &b) in in_bytes.iter().enumerate() {
        out_bytes[i] = b;
    }
    true
}

/// Check if slice is empty.
#[no_mangle]
pub extern "C" fn slice_is_empty(slice: ChSlice) -> bool {
    slice.len == 0
}

/// Compare two slices for equality.
#[no_mangle]
pub extern "C" fn slice_equal(a: ChSlice, b: ChSlice) -> bool {
    let a_bytes = unsafe { a.as_slice() };
    let b_bytes = unsafe { b.as_slice() };
    a_bytes == b_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_bytes() {
        let data = [1u8, 2, 3, 4, 5];
        let slice = unsafe { ChSlice::from_ref(&data) };
        assert_eq!(sum_bytes(slice), 15);
    }

    #[test]
    fn test_count_byte() {
        let data = [b'h', b'e', b'l', b'l', b'o'];
        let slice1 = unsafe { ChSlice::from_ref(&data) };
        let slice2 = unsafe { ChSlice::from_ref(&data) };
        assert_eq!(count_byte(slice1, b'l'), 2);
        assert_eq!(count_byte(slice2, b'x'), 0);
    }

    #[test]
    fn test_slice_starts_with() {
        let data = [b'h', b'e', b'l', b'l', b'o'];
        let slice1 = unsafe { ChSlice::from_ref(&data) };
        let slice2 = unsafe { ChSlice::from_ref(&data) };
        let prefix = unsafe { ChSlice::from_ref(&[b'h', b'e'])};
        let not_prefix = unsafe { ChSlice::from_ref(&[b'w', b'o'])};
        assert!(slice_starts_with(slice1, prefix));
        assert!(!slice_starts_with(slice2, not_prefix));
    }

    #[test]
    fn test_slice_ends_with() {
        let data = [b'h', b'e', b'l', b'l', b'o'];
        let slice1 = unsafe { ChSlice::from_ref(&data) };
        let slice2 = unsafe { ChSlice::from_ref(&data) };
        let suffix = unsafe { ChSlice::from_ref(&[b'l', b'o'])};
        let not_suffix = unsafe { ChSlice::from_ref(&[b'w', b'o'])};
        assert!(slice_ends_with(slice1, suffix));
        assert!(!slice_ends_with(slice2, not_suffix));
    }

    #[test]
    fn test_string_length() {
        let s = "hello";
        let bs = unsafe { ChBorrowStr::from_str(s) };
        assert_eq!(string_length(bs), 5);
    }

    #[test]
    fn test_slice_equal() {
        let a_data = [b'h', b'e', b'l', b'l', b'o'];
        let b_data = [b'h', b'e', b'l', b'l', b'o'];
        let c_data = [b'w', b'o', b'r', b'l', b'd'];

        let a1 = unsafe { ChSlice::from_ref(&a_data) };
        let a2 = unsafe { ChSlice::from_ref(&a_data) };
        let b = unsafe { ChSlice::from_ref(&b_data) };
        let c = unsafe { ChSlice::from_ref(&c_data) };

        assert!(slice_equal(a1, b));
        assert!(!slice_equal(a2, c));
    }

    #[test]
    fn test_slice_reverse_into() {
        let input_data = [1u8, 2, 3, 4, 5];
        let mut output_data = [0u8; 5];

        let input = unsafe { ChSlice::from_ref(&input_data) };
        let output = unsafe { ChSlice::from_ref(&output_data) };

        let result = slice_reverse_into(input, output, 5);
        assert!(result);
        assert_eq!(output_data, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_slice_is_empty() {
        let empty_data: [u8; 0] = [];
        let non_empty_data = [1u8, 2, 3];

        let empty = unsafe { ChSlice::from_ref(&empty_data) };
        let non_empty = unsafe { ChSlice::from_ref(&non_empty_data) };

        assert!(slice_is_empty(empty));
        assert!(!slice_is_empty(non_empty));
    }
}