//! Chimera runtime ABI support for Rust
//!
//! Provides `repr(C)` Chimera ABI types, FFI helpers, and conversion utilities.
//!
//! # Safety
//!
//! This crate deals with low-level FFI and raw pointer manipulation. All public
//! functions that dereference raw pointers require `unsafe` blocks, and callers
//! must ensure the preconditions are met.

#![warn(unused)]
#![warn(missing_docs)]

use std::fmt;

/// Version information
pub const CHIMERA_ABI_VERSION_MAJOR: u32 = 0;
pub const CHIMERA_ABI_VERSION_MINOR: u32 = 1;
pub const CHIMERA_ABI_VERSION_PATCH: u32 = 0;
pub const CHIMERA_ABI_VERSION_STRING: &str = "0.1.0";

/// Status codes for Chimera ABI operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    Error = 1,
    InvalidArg = 2,
    InvalidState = 3,
    NotFound = 4,
    OutOfMemory = 5,
    BufferTooSmall = 6,
    TypeMismatch = 7,
    BorrowExclusive = 8,
    UseAfterMove = 9,
    DoubleFree = 10,
    Panic = 11,
}

impl Status {
    /// Check if status indicates success
    pub fn is_ok(self) -> bool {
        self == Status::Ok
    }

    /// Check if status indicates an error
    pub fn is_error(self) -> bool {
        self != Status::Ok
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Ok
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Ok => write!(f, "OK"),
            Status::Error => write!(f, "Error"),
            Status::InvalidArg => write!(f, "Invalid argument"),
            Status::InvalidState => write!(f, "Invalid state"),
            Status::NotFound => write!(f, "Not found"),
            Status::OutOfMemory => write!(f, "Out of memory"),
            Status::BufferTooSmall => write!(f, "Buffer too small"),
            Status::TypeMismatch => write!(f, "Type mismatch"),
            Status::BorrowExclusive => write!(f, "Borrow exclusive"),
            Status::UseAfterMove => write!(f, "Use after move"),
            Status::DoubleFree => write!(f, "Double free"),
            Status::Panic => write!(f, "Panic"),
        }
    }
}

/// Error domains for rich error categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorDomain {
    None = 0,
    Io = 1,
    Memory = 2,
    Type = 3,
    Ownership = 4,
    Validation = 5,
    Runtime = 6,
}

/// Extended error information (FFI compatible)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Error {
    pub status: Status,
    pub domain: ErrorDomain,
    pub code: i32,
    message: *const std::os::raw::c_char,
    file: *const std::os::raw::c_char,
    line: i32,
}

impl Error {
    /// Create a new error
    ///
    /// # Safety
    ///
    /// The message and file pointers must be valid null-terminated C strings
    /// that live at least as long as the Error itself.
    pub unsafe fn new(
        status: Status,
        domain: ErrorDomain,
        code: i32,
        message: *const std::os::raw::c_char,
        file: *const std::os::raw::c_char,
        line: i32,
    ) -> Self {
        Self {
            status,
            domain,
            code,
            message,
            file,
            line,
        }
    }
}

/// Ownership kinds for borrowed/owned values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Ownership {
    Borrowed = 0,
    BorrowedMut = 1,
    Owned = 2,
    Raw = 3,
}

/// Lifetime kinds for borrowed values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Lifetime {
    Call = 0,
    Static = 1,
    Owner = 2,
}

/// Calling convention identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CConv {
    C = 0,
    SysV = 1,
    Win64 = 2,
    Wasm = 3,
    Chimera = 4,
}

/// Target architecture identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TargetArch {
    X86_64 = 0,
    Aarch64 = 1,
    Wasm32 = 2,
    Wasm64 = 3,
}

/// Target OS identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TargetOs {
    Linux = 0,
    Macos = 1,
    Windows = 2,
    Wasi = 3,
}

/// Allocator allocation kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AllocKind {
    New = 0,
    Malloc = 1,
    Resize = 2,
    Free = 3,
}

/// Allocator callback function type (FFI compatible)
pub type AllocFn = unsafe extern "C" fn(
    user_data: *mut std::os::raw::c_void,
    kind: AllocKind,
    ptr: *mut std::os::raw::c_void,
    old_size: usize,
    new_size: usize,
) -> *mut std::os::raw::c_void;

/// Allocator configuration (FFI compatible)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Allocator {
    pub alloc: AllocFn,
    pub user_data: *mut std::os::raw::c_void,
    pub header_size: usize,
}

/// Canonical lowered allocator descriptor (FFI compatible)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ChAllocator {
    pub id: u64,
    pub kind: u32,
    pub ptr: *mut std::os::raw::c_void,
}

/// Default system allocator
#[cfg(feature = "alloc")]
extern "C" {
    pub static chimera_default_allocator: Allocator;
}

/// Register a custom allocator for use by Chimera runtime
///
/// # Safety
///
/// The allocator must be valid and last for the duration of the program
/// unless explicitly unregistered.
#[cfg(feature = "alloc")]
pub unsafe fn allocator_register(allocator: *mut Allocator) -> Status {
    extern "C" {
        fn chimera_allocator_register(allocator: *mut Allocator) -> Status;
    }
    chimera_allocator_register(allocator)
}

/// Get the currently registered allocator
#[cfg(feature = "alloc")]
pub unsafe fn allocator_get_current() -> *mut Allocator {
    extern "C" {
        fn chimera_allocator_get_current() -> *mut Allocator;
    }
    chimera_allocator_get_current()
}

/// Allocate memory using the current allocator
///
/// # Safety
///
/// The returned pointer must be used appropriately and freed via dealloc.
#[cfg(feature = "alloc")]
pub unsafe fn alloc(size: usize) -> *mut std::os::raw::c_void {
    extern "C" {
        fn chimera_alloc(size: usize) -> *mut std::os::raw::c_void;
    }
    chimera_alloc(size)
}

/// Free memory using the current allocator
///
/// # Safety
///
/// The pointer must have been allocated via alloc with the correct size.
#[cfg(feature = "alloc")]
pub unsafe fn dealloc(ptr: *mut std::os::raw::c_void, size: usize) {
    extern "C" {
        fn chimera_dealloc(ptr: *mut std::os::raw::c_void, size: usize);
    }
    chimera_dealloc(ptr, size);
}

/// Drop callback type for opaque payloads
pub type DropFn = unsafe extern "C" fn(*mut std::os::raw::c_void, usize);

/// Register a drop callback for an owned payload
///
/// # Safety
///
/// The pointer must be valid and owned by the caller.
#[cfg(feature = "alloc")]
pub unsafe fn register_drop(ptr: *mut std::os::raw::c_void, size: usize, drop_fn: Option<DropFn>) {
    extern "C" {
        fn chimera_register_drop(ptr: *mut std::os::raw::c_void, size: usize, drop_fn: Option<DropFn>);
    }
    chimera_register_drop(ptr, size, drop_fn);
}

/// Drop a registered payload and clean up
///
/// # Safety
///
/// The pointer must have been registered via register_drop.
#[cfg(feature = "alloc")]
pub unsafe fn drop_payload(ptr: *mut std::os::raw::c_void) {
    extern "C" {
        fn chimera_drop(ptr: *mut std::os::raw::c_void);
    }
    chimera_drop(ptr);
}

/// Owned-byte cleanup helper
///
/// # Safety
///
/// The pointer must have been allocated via the current allocator.
#[cfg(feature = "alloc")]
pub unsafe fn drop_bytes(ptr: *mut std::os::raw::c_void, len: usize) {
    extern "C" {
        fn chimera_drop_bytes(ptr: *mut std::os::raw::c_void, len: usize) -> ();
    }
    chimera_drop_bytes(ptr, len);
}

/// Owned handle drop helper - drops Box<T> style handles
///
/// # Safety
///
/// The pointer must be a valid owned handle allocated by the current allocator.
/// This is a placeholder that requires runtime linkage to chimera runtime.
#[cfg(feature = "alloc")]
pub unsafe fn drop_handle<T>(_ptr: *mut T) {
    // Placeholder: requires chimera runtime linkage for actual implementation.
    // The runtime provides chimera_drop_handle which handles Box/Vec/String drops.
}

/// Vec-style buffer drop helper - drops a buffer with length
///
/// # Safety
///
/// The pointer must be a valid owned buffer of `len` elements.
/// This is a placeholder that requires runtime linkage to chimera runtime.
#[cfg(feature = "alloc")]
pub unsafe fn drop_vec(_ptr: *mut u8, _len: usize, _capacity: usize) {
    // Placeholder: requires chimera runtime linkage for actual implementation.
    // The runtime provides chimera_drop_vec which handles Vec-style buffers.
}

/// Slice type for borrowed sequences (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Slice {
    /// Pointer to slice data
    pub data: *const std::os::raw::c_void,
    /// Length in bytes
    pub len: usize,
}

/// Canonical lowered byte slice (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ChSlice {
    pub ptr: *const std::os::raw::c_void,
    pub len: u64,
}

impl Slice {
    /// Create a slice from a pointer and length
    ///
    /// # Safety
    ///
    /// The pointer must be valid for `len` bytes.
    pub unsafe fn from_ptr(data: *const std::os::raw::c_void, len: usize) -> Self {
        Self { data, len }
    }

    /// Check if slice is empty
    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Get the length of the slice
    pub fn len(self) -> usize {
        self.len
    }
}

/// Mutable slice type (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct SliceMut {
    pub data: *mut std::os::raw::c_void,
    pub len: usize,
}

impl SliceMut {
    /// Create a mutable slice from a pointer and length
    ///
    /// # Safety
    ///
    /// The pointer must be valid for `len` bytes and exclusive access required.
    pub unsafe fn from_ptr(data: *mut std::os::raw::c_void, len: usize) -> Self {
        Self { data, len }
    }
}

/// String type (always UTF-8, FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct String {
    data: *const std::os::raw::c_char,
    len: usize,
    capacity: usize,
}

impl String {
    /// Create an empty string
    pub fn new() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0,
            capacity: 0,
        }
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if string is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}

/// Canonical borrowed UTF-8 string (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ChBorrowStr {
    pub ptr: *const u8,
    pub len: u64,
    pub lifetime: u32,
}

/// Canonical owned byte buffer (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ChOwnedBytes {
    pub ptr: *mut u8,
    pub len: u64,
    pub capacity: u64,
    pub allocator_id: u64,
}

/// Canonical opaque owned handle (FFI compatible)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ChHandle {
    pub ptr: *mut std::os::raw::c_void,
    pub drop_fn: Option<extern "C" fn(*mut std::os::raw::c_void)>,
    pub size: u64,
}

/// Result type (FFI compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Result {
    is_ok: bool,
    _padding: [u8; 7],
}

impl Result {
    /// Check if result is OK
    pub fn is_ok(&self) -> bool {
        self.is_ok
    }

    /// Check if result is Err
    pub fn is_err(&self) -> bool {
        !self.is_ok
    }
}

/// Panic behavior on panic boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PanicPolicy {
    Abort = 0,
    Unwind = 1,
    Rust = 2,
}

/// Runtime mode - controls which runtime features are linked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RuntimeMode {
    /// Core runtime - minimal types, no standard library
    Core = 0,
    /// Standard runtime - full standard library
    Std = 1,
    /// No standard library - embedded/no_std targets
    NoStd = 2,
    /// Actor runtime - concurrent component support
    Actor = 3,
}

impl RuntimeMode {
    /// Check if this mode includes standard library
    pub fn has_std(self) -> bool {
        matches!(self, RuntimeMode::Std | RuntimeMode::Actor)
    }

    /// Check if this mode includes core runtime
    pub fn has_core(self) -> bool {
        matches!(self, RuntimeMode::Core | RuntimeMode::NoStd)
    }

    /// Check if this mode supports actor features
    pub fn has_actor(self) -> bool {
        matches!(self, RuntimeMode::Actor)
    }
}

impl Default for RuntimeMode {
    fn default() -> Self {
        RuntimeMode::NoStd
    }
}

/// Panic catcher - converts panics to configured error or aborts
///
/// # Safety
///
/// This function must be called from a valid FFI boundary where the caller
/// has set up the out_error pointer appropriately.
#[cfg(feature = "std")]
pub unsafe fn catch_panic<F, T>(f: F) -> std::result::Result<T, Status>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err(|_| Status::Panic)
}

/// Panic info passed across boundary
#[derive(Debug, Clone)]
#[repr(C)]
pub struct PanicInfo {
    pub message: *const std::os::raw::c_char,
    pub message_len: usize,
    pub file: *const std::os::raw::c_char,
    pub line: i32,
    pub reason: *const std::os::raw::c_char,
}

/// Canonical lowered error payload (FFI compatible)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ChError {
    pub domain: u32,
    pub code: u32,
    pub flags: u32,
    pub message_ptr: *const std::os::raw::c_void,
    pub message_len: u64,
    pub payload_ptr: *mut std::os::raw::c_void,
    pub payload_drop_fn: Option<extern "C" fn(*mut std::os::raw::c_void)>,
    pub payload_drop_ctx: *mut std::os::raw::c_void,
}

impl ChError {
    /// Create a success error (no error)
    pub fn success() -> Self {
        Self {
            domain: 0,
            code: 0,
            flags: 0,
            message_ptr: std::ptr::null(),
            message_len: 0,
            payload_ptr: std::ptr::null_mut(),
            payload_drop_fn: None,
            payload_drop_ctx: std::ptr::null_mut(),
        }
    }

    /// Check if this represents no error (success)
    pub fn is_success(&self) -> bool {
        self.domain == 0 && self.code == 0
    }

    /// Check if this represents an error
    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Create an error with domain and code
    ///
    /// # Safety
    ///
    /// The message pointer must be a valid null-terminated C string if non-null.
    pub unsafe fn new_error(domain: u32, code: u32, message: *const std::os::raw::c_char) -> Self {
        let message_len = if !message.is_null() {
            // Calculate length of null-terminated string
            let mut len = 0;
            while *message.add(len) != 0 {
                len += 1;
            }
            len
        } else {
            0
        };

        Self {
            domain,
            code,
            flags: 0,
            message_ptr: message as *const std::os::raw::c_void,
            message_len: message_len as u64,
            payload_ptr: std::ptr::null_mut(),
            payload_drop_fn: None,
            payload_drop_ctx: std::ptr::null_mut(),
        }
    }
}

/// Trait for converting Rust error types to ChError
pub trait IntoChError {
    /// Convert self to a ChError
    fn into_ch_error(self) -> ChError;
}

impl IntoChError for ChError {
    fn into_ch_error(self) -> ChError {
        self
    }
}

impl IntoChError for () {
    fn into_ch_error(self) -> ChError {
        ChError::success()
    }
}

impl IntoChError for std::convert::Infallible {
    fn into_ch_error(self) -> ChError {
        match self {}
    }
}

/// Convert a Rust Result<T, E> to ChError with out pointer
///
/// # Safety
///
/// The out_error pointer must point to valid memory for writing.
#[cfg(feature = "std")]
pub unsafe fn result_to_ch_error<T, E>(
    result: std::result::Result<T, E>,
    out_ok: *mut T,
    out_error: *mut ChError,
) -> Status
where
    E: IntoChError,
{
    match result {
        Ok(value) => {
            if !out_ok.is_null() {
                out_ok.write(value);
            }
            if !out_error.is_null() {
                out_error.write(ChError::success());
            }
            Status::Ok
        }
        Err(e) => {
            if !out_error.is_null() {
                out_error.write(e.into_ch_error());
            }
            Status::Error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_status_is_ok() {
        assert!(Status::Ok.is_ok());
        assert!(!Status::Error.is_ok());
    }

    #[test]
    fn test_status_is_error() {
        assert!(!Status::Ok.is_error());
        assert!(Status::Error.is_error());
    }

    #[test]
    fn test_slice_new() {
        // Safety: null pointer with zero length is valid
        let slice = unsafe { Slice::from_ptr(std::ptr::null(), 0) };
        assert!(slice.is_empty());
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_string_default() {
        let s = String::new();
        assert!(s.is_empty());
    }

    #[test]
    fn test_result_is_ok() {
        let ok_result = Result { is_ok: true, _padding: [0; 7] };
        let err_result = Result { is_ok: false, _padding: [0; 7] };
        assert!(ok_result.is_ok());
        assert!(!ok_result.is_err());
        assert!(err_result.is_err());
        assert!(!err_result.is_ok());
    }

    #[test]
    fn test_ownership_variants() {
        assert_eq!(Ownership::Borrowed as i32, 0);
        assert_eq!(Ownership::BorrowedMut as i32, 1);
        assert_eq!(Ownership::Owned as i32, 2);
        assert_eq!(Ownership::Raw as i32, 3);
    }

    #[test]
    fn test_lifetime_variants() {
        assert_eq!(Lifetime::Call as i32, 0);
        assert_eq!(Lifetime::Static as i32, 1);
        assert_eq!(Lifetime::Owner as i32, 2);
    }

    #[test]
    fn test_cconv_variants() {
        assert_eq!(CConv::C as i32, 0);
        assert_eq!(CConv::SysV as i32, 1);
        assert_eq!(CConv::Win64 as i32, 2);
        assert_eq!(CConv::Wasm as i32, 3);
        assert_eq!(CConv::Chimera as i32, 4);
    }

    #[test]
    fn test_target_arch() {
        assert_eq!(TargetArch::X86_64 as i32, 0);
        assert_eq!(TargetArch::Aarch64 as i32, 1);
        assert_eq!(TargetArch::Wasm32 as i32, 2);
    }

    #[test]
    fn test_target_os() {
        assert_eq!(TargetOs::Linux as i32, 0);
        assert_eq!(TargetOs::Macos as i32, 1);
        assert_eq!(TargetOs::Windows as i32, 2);
        assert_eq!(TargetOs::Wasi as i32, 3);
    }

    #[test]
    fn test_panic_policy() {
        assert_eq!(PanicPolicy::Abort as i32, 0);
        assert_eq!(PanicPolicy::Unwind as i32, 1);
        assert_eq!(PanicPolicy::Rust as i32, 2);
    }

    #[test]
    fn test_alloc_kind() {
        assert_eq!(AllocKind::New as i32, 0);
        assert_eq!(AllocKind::Malloc as i32, 1);
        assert_eq!(AllocKind::Resize as i32, 2);
        assert_eq!(AllocKind::Free as i32, 3);
    }

    #[test]
    fn test_runtime_mode_defaults() {
        assert_eq!(RuntimeMode::default(), RuntimeMode::NoStd);
    }

    #[test]
    fn test_runtime_mode_has_std() {
        assert!(!RuntimeMode::Core.has_std());
        assert!(RuntimeMode::Std.has_std());
        assert!(!RuntimeMode::NoStd.has_std());
        assert!(RuntimeMode::Actor.has_std()); // Actor requires std runtime
    }

    #[test]
    fn test_runtime_mode_has_core() {
        assert!(RuntimeMode::Core.has_core());
        assert!(!RuntimeMode::Std.has_core());
        assert!(RuntimeMode::NoStd.has_core());
        assert!(!RuntimeMode::Actor.has_core());
    }

    #[test]
    fn test_runtime_mode_has_actor() {
        assert!(!RuntimeMode::Core.has_actor());
        assert!(!RuntimeMode::Std.has_actor());
        assert!(!RuntimeMode::NoStd.has_actor());
        assert!(RuntimeMode::Actor.has_actor());
    }

    #[test]
    fn canonical_struct_sizes_and_alignments_match_lean() {
        assert_eq!(size_of::<ChError>(), 56);
        assert_eq!(align_of::<ChError>(), 8);
        assert_eq!(size_of::<ChAllocator>(), 24);
        assert_eq!(align_of::<ChAllocator>(), 8);
        assert_eq!(size_of::<ChSlice>(), 16);
        assert_eq!(align_of::<ChSlice>(), 8);
        assert_eq!(size_of::<ChBorrowStr>(), 24);
        assert_eq!(align_of::<ChBorrowStr>(), 8);
        assert_eq!(size_of::<ChOwnedBytes>(), 32);
        assert_eq!(align_of::<ChOwnedBytes>(), 8);
        assert_eq!(size_of::<ChHandle>(), 24);
        assert_eq!(align_of::<ChHandle>(), 8);
    }

    #[test]
    fn canonical_struct_field_offsets_match_lean() {
        let err = ChError {
            domain: 0,
            code: 0,
            flags: 0,
            message_ptr: std::ptr::null(),
            message_len: 0,
            payload_ptr: std::ptr::null_mut(),
            payload_drop_fn: None,
            payload_drop_ctx: std::ptr::null_mut(),
        };
        let err_base = &err as *const _ as usize;
        assert_eq!((&err.domain as *const _ as usize) - err_base, 0);
        assert_eq!((&err.code as *const _ as usize) - err_base, 4);
        assert_eq!((&err.flags as *const _ as usize) - err_base, 8);
        assert_eq!((&err.message_ptr as *const _ as usize) - err_base, 16);
        assert_eq!((&err.message_len as *const _ as usize) - err_base, 24);
        assert_eq!((&err.payload_ptr as *const _ as usize) - err_base, 32);
        assert_eq!((&err.payload_drop_fn as *const _ as usize) - err_base, 40);
        assert_eq!((&err.payload_drop_ctx as *const _ as usize) - err_base, 48);

        let handle = ChHandle { ptr: std::ptr::null_mut(), drop_fn: None, size: 0 };
        let handle_base = &handle as *const _ as usize;
        assert_eq!((&handle.ptr as *const _ as usize) - handle_base, 0);
        assert_eq!((&handle.drop_fn as *const _ as usize) - handle_base, 8);
        assert_eq!((&handle.size as *const _ as usize) - handle_base, 16);
    }

    #[test]
    fn test_catch_panic_success() {
        #[cfg(feature = "std")]
        {
            let result = unsafe { catch_panic(|| 42) };
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 42);
        }
    }

    #[test]
    fn test_catch_panic_panics() {
        #[cfg(feature = "std")]
        {
            let result = unsafe { catch_panic::<_, ()>(|| panic!("test")) };
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Status::Panic);
        }
    }

    #[test]
    fn test_catch_panic_with_result() {
        #[cfg(feature = "std")]
        {
            let result = unsafe { catch_panic(|| Ok::<i32, ()>(100)) };
            assert!(result.is_ok());
            assert_eq!(result.unwrap().unwrap(), 100);
        }
    }

    #[test]
    fn test_ch_error_success() {
        let err = ChError::success();
        assert!(err.is_success());
        assert!(!err.is_error());
        assert_eq!(err.domain, 0);
        assert_eq!(err.code, 0);
    }

    #[test]
    fn test_ch_error_new_error() {
        // Safety: null is valid for message
        let err = unsafe { ChError::new_error(1, 42, std::ptr::null()) };
        assert!(err.is_error());
        assert!(!err.is_success());
        assert_eq!(err.domain, 1);
        assert_eq!(err.code, 42);
        assert!(err.message_ptr.is_null());
    }

    #[test]
    fn test_result_to_ch_error_ok() {
        #[cfg(feature = "std")]
        {
            let result: std::result::Result<i32, ()> = Ok(42);
            let mut out_ok: i32 = 0;
            let mut out_error = ChError::success();

            let status = unsafe {
                result_to_ch_error(result, &mut out_ok, &mut out_error)
            };

            assert_eq!(status, Status::Ok);
            assert_eq!(out_ok, 42);
            assert!(out_error.is_success());
        }
    }

    #[test]
    fn test_result_to_ch_error_err() {
        #[cfg(feature = "std")]
        {
            let result: std::result::Result<i32, ChError> = Err(unsafe { ChError::new_error(2, 100, std::ptr::null()) });
            let mut out_ok: i32 = 0;
            let mut out_error = ChError::success();

            let status = unsafe {
                result_to_ch_error(result, &mut out_ok, &mut out_error)
            };

            assert_eq!(status, Status::Error);
            assert!(out_error.is_error());
            assert_eq!(out_error.domain, 2);
            assert_eq!(out_error.code, 100);
        }
    }

    #[test]
    fn test_into_ch_error_trait_unit() {
        let err: ChError = ().into_ch_error();
        assert!(err.is_success());
    }

    #[test]
    fn test_into_ch_error_trait_ch_error() {
        let input = unsafe { ChError::new_error(5, 500, std::ptr::null()) };
        let err: ChError = input.into_ch_error();
        assert!(err.is_error());
        assert_eq!(err.domain, 5);
        assert_eq!(err.code, 500);
    }
}
