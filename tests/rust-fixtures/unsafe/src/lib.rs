//! Unsafe fixture demonstrating raw pointer operations and unsafe extern.
//!
//! This fixture tests the chimera-rust-effects and compiler-core unsafe
//! boundary handling: raw pointer dereference, unsafe extern "C" functions,
//! and TCB (trusted computing base) trust ledger verification.

/// Trust level for unsafe operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Untrusted = 0,
    Trusted = 1,
    TCB = 2,
}

impl TrustLevel {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => TrustLevel::Untrusted,
            1 => TrustLevel::Trusted,
            2 => TrustLevel::TCB,
            _ => TrustLevel::Untrusted,
        }
    }
}

/// Trust ledger entry recording an unsafe operation.
#[repr(C)]
pub struct TrustLedgerEntry {
    pub operation_id: u32,
    pub trust_level: u8,
    pub was_checked: bool,
    pub line_number: u32,
}

impl TrustLedgerEntry {
    pub fn new(operation_id: u32, trust_level: TrustLevel, line_number: u32) -> Self {
        TrustLedgerEntry {
            operation_id,
            trust_level: trust_level.to_u8(),
            was_checked: false,
            line_number,
        }
    }

    pub fn marked_checked(&mut self) {
        self.was_checked = true;
    }

    pub fn trust_level(&self) -> TrustLevel {
        TrustLevel::from_u8(self.trust_level)
    }
}

/// Trust ledger tracking unsafe operations.
pub struct TrustLedger {
    entries: Vec<TrustLedgerEntry>,
}

impl TrustLedger {
    pub fn new() -> Self {
        TrustLedger { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: TrustLedgerEntry) {
        self.entries.push(entry);
    }

    pub fn get_entries(&self) -> &[TrustLedgerEntry] {
        &self.entries
    }

    pub fn verify_all_checked(&self) -> bool {
        self.entries.iter().all(|e| e.was_checked)
    }
}

/// Raw pointer dereference result.
#[repr(C)]
pub struct DerefResult {
    pub value: u64,
    pub was_valid: bool,
}

impl DerefResult {
    pub fn ok(value: u64) -> Self {
        DerefResult {
            value,
            was_valid: true,
        }
    }

    pub fn invalid() -> Self {
        DerefResult {
            value: 0,
            was_valid: false,
        }
    }
}

/// A raw pointer that wraps a u64 value.
pub struct RawU64 {
    ptr: *mut u64,
    is_valid: bool,
}

impl RawU64 {
    pub fn new(value: u64) -> Self {
        let boxed = Box::new(value);
        RawU64 {
            ptr: Box::into_raw(boxed),
            is_valid: true,
        }
    }

    pub fn from_ptr(ptr: *mut u64) -> Self {
        RawU64 {
            ptr,
            is_valid: !ptr.is_null(),
        }
    }

    pub fn as_ptr(&self) -> *mut u64 {
        self.ptr
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// SAFETY: Caller must guarantee the pointer is valid and aligned.
    pub unsafe fn deref(&self) -> u64 {
        *self.ptr
    }

    pub fn into_raw(self) -> *mut u64 {
        let p = self.ptr;
        std::mem::forget(self);
        p
    }
}

impl Drop for RawU64 {
    fn drop(&mut self) {
        if self.is_valid {
            unsafe {
                let _ = Box::from_raw(self.ptr);
            }
        }
    }
}

/// Create a new raw u64 on the heap.
#[no_mangle]
pub extern "C" fn raw_u64_create(value: u64) -> *mut u64 {
    Box::into_raw(Box::new(value))
}

/// Dereference a raw u64 pointer.
#[no_mangle]
pub unsafe extern "C" fn raw_u64_deref(ptr: *mut u64) -> DerefResult {
    if ptr.is_null() {
        return DerefResult::invalid();
    }
    DerefResult::ok(*ptr)
}

/// Create a raw u64 and dereference it safely.
#[no_mangle]
pub extern "C" fn raw_u64_new_and_deref(value: u64) -> DerefResult {
    let raw = RawU64::new(value);
    let result = unsafe { raw.deref() };
    drop(raw);
    DerefResult::ok(result)
}

/// Write a value to a raw pointer.
#[no_mangle]
pub unsafe extern "C" fn raw_u64_write(ptr: *mut u64, value: u64) -> bool {
    if ptr.is_null() {
        return false;
    }
    *ptr = value;
    true
}

/// Swap values at two raw pointer locations.
#[no_mangle]
pub unsafe extern "C" fn raw_u64_swap(ptr1: *mut u64, ptr2: *mut u64) -> bool {
    if ptr1.is_null() || ptr2.is_null() {
        return false;
    }
    let temp = *ptr1;
    *ptr1 = *ptr2;
    *ptr2 = temp;
    true
}

/// Unsafe extern "C" function that performs a TCB operation.
///
/// This function is marked unsafe because it performs an operation
/// that requires the caller to verify certain trust properties.
#[no_mangle]
pub unsafe extern "C" fn tcb_verify_and_commit(
    entry: *mut TrustLedgerEntry,
    expected_op_id: u32,
) -> bool {
    if entry.is_null() {
        return false;
    }
    let entry = &mut *entry;
    if entry.operation_id != expected_op_id {
        return false;
    }
    entry.marked_checked();
    true
}

/// Record an unsafe operation in the trust ledger.
#[no_mangle]
pub extern "C" fn record_unsafe_operation(
    ledger: *mut TrustLedgerEntry,
    op_id: u32,
    trust_level: TrustLevel,
    line: u32,
) {
    if !ledger.is_null() {
        unsafe {
            *ledger = TrustLedgerEntry::new(op_id, trust_level, line);
        }
    }
}

/// Verify a trust ledger entry was properly checked.
#[no_mangle]
pub unsafe extern "C" fn verify_trust_entry(entry: *mut TrustLedgerEntry) -> bool {
    if entry.is_null() {
        return false;
    }
    let entry = &*entry;
    entry.was_checked
}

/// Add two numbers using raw pointer arithmetic.
#[no_mangle]
pub unsafe extern "C" fn raw_pointer_add(ptr: *const u64, offset: usize) -> u64 {
    let ptr = ptr.wrapping_add(offset);
    if ptr.is_null() {
        return 0;
    }
    *ptr
}

/// Check if a pointer is aligned.
#[no_mangle]
pub unsafe extern "C" fn pointer_is_aligned(ptr: *const u64, alignment: u64) -> bool {
    if ptr.is_null() {
        return false;
    }
    let addr = ptr as u64;
    addr % alignment == 0
}

/// Unsafe function that requires caller to uphold invariants.
#[no_mangle]
pub unsafe extern "C" fn unsafe_invariant_check(
    ptr: *mut u64,
    len: usize,
    min_value: u64,
) -> bool {
    if ptr.is_null() || len == 0 {
        return false;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    slice.iter().all(|&v| v >= min_value)
}

/// Trust level query.
#[no_mangle]
pub extern "C" fn get_minimum_trust_level() -> TrustLevel {
    TrustLevel::TCB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_u64_create_and_deref() {
        let ptr = raw_u64_create(42);
        let result = unsafe { raw_u64_deref(ptr) };
        assert!(result.was_valid);
        assert_eq!(result.value, 42);
        unsafe { let _ = Box::from_raw(ptr); }
    }

    #[test]
    fn test_raw_u64_new_and_deref() {
        let result = raw_u64_new_and_deref(100);
        assert!(result.was_valid);
        assert_eq!(result.value, 100);
    }

    #[test]
    fn test_raw_u64_deref_null() {
        let result = unsafe { raw_u64_deref(std::ptr::null_mut()) };
        assert!(!result.was_valid);
    }

    #[test]
    fn test_raw_u64_write() {
        let ptr = raw_u64_create(0);
        let success = unsafe { raw_u64_write(ptr, 999) };
        assert!(success);
        let result = unsafe { raw_u64_deref(ptr) };
        assert_eq!(result.value, 999);
        unsafe { let _ = Box::from_raw(ptr); }
    }

    #[test]
    fn test_raw_u64_swap_success() {
        let ptr1 = raw_u64_create(10);
        let ptr2 = raw_u64_create(20);
        let success = unsafe { raw_u64_swap(ptr1, ptr2) };
        assert!(success);
        let r1 = unsafe { raw_u64_deref(ptr1) };
        let r2 = unsafe { raw_u64_deref(ptr2) };
        assert_eq!(r1.value, 20);
        assert_eq!(r2.value, 10);
        unsafe {
            let _ = Box::from_raw(ptr1);
            let _ = Box::from_raw(ptr2);
        }
    }

    #[test]
    fn test_trust_ledger_entry() {
        let mut entry = TrustLedgerEntry::new(1, TrustLevel::Trusted, 100);
        assert_eq!(entry.operation_id, 1);
        assert_eq!(entry.trust_level(), TrustLevel::Trusted);
        assert!(!entry.was_checked);
        entry.marked_checked();
        assert!(entry.was_checked);
    }

    #[test]
    fn test_record_and_verify_unsafe_op() {
        let mut entry = TrustLedgerEntry::new(42, TrustLevel::TCB, 200);
        let success = unsafe { tcb_verify_and_commit(&mut entry, 42) };
        assert!(success);
        assert!(entry.was_checked);
    }

    #[test]
    fn test_verify_trust_entry_wrong_op_id() {
        let mut entry = TrustLedgerEntry::new(99, TrustLevel::Trusted, 50);
        let success = unsafe { tcb_verify_and_commit(&mut entry, 100) };
        assert!(!success);
        assert!(!entry.was_checked);
    }

    #[test]
    fn test_trust_level_conversion() {
        assert_eq!(TrustLevel::Untrusted.to_u8(), 0);
        assert_eq!(TrustLevel::Trusted.to_u8(), 1);
        assert_eq!(TrustLevel::TCB.to_u8(), 2);
        assert_eq!(TrustLevel::from_u8(0), TrustLevel::Untrusted);
        assert_eq!(TrustLevel::from_u8(1), TrustLevel::Trusted);
        assert_eq!(TrustLevel::from_u8(2), TrustLevel::TCB);
    }

    #[test]
    fn test_get_minimum_trust_level() {
        let level = get_minimum_trust_level();
        assert_eq!(level, TrustLevel::TCB);
    }

    #[test]
    fn test_unsafe_invariant_check_valid() {
        let values = vec![10u64, 20, 30, 40, 50];
        let ptr = values.as_ptr() as *mut u64;
        let len = values.len();
        let result = unsafe { unsafe_invariant_check(ptr, len, 5) };
        assert!(result);
    }

    #[test]
    fn test_unsafe_invariant_check_invalid() {
        let values = vec![3u64, 20, 30, 40, 50];
        let ptr = values.as_ptr() as *mut u64;
        let len = values.len();
        let result = unsafe { unsafe_invariant_check(ptr, len, 5) };
        assert!(!result);
    }

    #[test]
    fn test_pointer_is_aligned_valid() {
        let value = 0u64;
        let ptr = &value as *const u64;
        let result = unsafe { pointer_is_aligned(ptr, 8) };
        assert!(result);
    }

    #[test]
    fn test_pointer_is_aligned_invalid() {
        let value = 0u64;
        let ptr = &value as *const u64;
        let result = unsafe { pointer_is_aligned(ptr, 7) };
        assert!(!result);
    }
}