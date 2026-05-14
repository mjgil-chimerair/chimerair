-- ChimeraProof Runtime: ABI Support
-- Runtime ABI artifacts for C/Rust/Zig.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.CanonicalStructs

namespace Chimera.Runtime

/--
C header content for chimera_abi.h.
This represents the canonical C ABI types and macros.
-/
def chimera_abi_h_content := "
/* chimera_abi.h - Chimera ABI runtime support
 * Automatically generated from ChimeraProof canonical structs.
 * Do not edit manually.
 */

#ifndef CHIMERA_ABI_H
#define CHIMERA_ABI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* Chimera ABI version */
#define CHIMERA_ABI_VERSION 1

/* Status codes: 0 = success, non-zero = error */
typedef int32_t ch_status;

/* Status code constructors */
#define CHIMERA_STATUS_OK 0
#define CHIMERA_STATUS_ERR(err_domain, err_code) ((ch_status)((err_domain) * 65536 + (err_code)))

/* Error domain IDs */
#define CHIMERA_DOMAIN_NONE 0
#define CHIMERA_DOMAIN_CHIMERA 1
#define CHIMERA_DOMAIN_C_ERRNO 2
#define CHIMERA_DOMAIN_RUST_RESULT 3
#define CHIMERA_DOMAIN_RUST_PANIC 4
#define CHIMERA_DOMAIN_ZIG_ERROR 5
#define CHIMERA_DOMAIN_ZIG_PANIC 6
#define CHIMERA_DOMAIN_USER 256

/* ch_error layout */
struct ch_error {
    uint32_t domain;
    uint32_t code;
    uint32_t flags;
    void *message_ptr;
    uint64_t message_len;
    void *payload_ptr;
    void (*payload_drop_fn)(void *ctx);
    void *payload_drop_ctx;
};

/* ch_allocator layout */
struct ch_allocator {
    uint64_t id;
    uint32_t kind;
    void *ptr;
};

/* ch_slice layout (ptr + len) */
struct ch_slice {
    void *ptr;
    uint64_t len;
};

/* ch_borrow_str layout */
struct ch_borrow_str {
    uint8_t *ptr;
    uint64_t len;
    uint32_t lifetime;
};

/* ch_owned_bytes layout */
struct ch_owned_bytes {
    uint8_t *ptr;
    uint64_t len;
    uint64_t capacity;
    uint64_t allocator_id;
};

/* ch_handle layout */
struct ch_handle {
    void *ptr;
    void (*drop_fn)(void *);
    uint64_t size;
};

/* Panic policy IDs */
#define CHIMERA_PANIC_FORBIDDEN 0
#define CHIMERA_PANIC_CATCH 1
#define CHIMERA_PANIC_ABORT 2

/* Safety class IDs */
#define CHIMERA_SAFETY_VERIFIED 0
#define CHIMERA_SAFETY_GENERATED 1
#define CHIMERA_SAFETY_TRUSTED 2
#define CHIMERA_SAFETY_UNSAFE 3

/* Allocator kinds */
#define CHIMERA_ALLOC_SYSTEM 0
#define CHIMERA_ALLOC_NULL 1
#define CHIMERA_ALLOC_SHARED 2
#define CHIMERA_ALLOC_LANG_OWNED 3
#define CHIMERA_ALLOC_CUSTOM 4

/* Effect bits */
#define CHIMERA_EFF_NONE 0
#define CHIMERA_EFF_MAY_ERROR 1
#define CHIMERA_EFF_MAY_PANIC 2
#define CHIMERA_EFF_MAY_ALLOC 4
#define CHIMERA_EFF_MAY_DEALLOC 8
#define CHIMERA_EFF_MAY_BLOCK 16
#define CHIMERA_EFF_MAY_FFI 32

#endif /* CHIMERA_ABI_H */
"

/--
Rust ABI support module content.
-/
def chimera_abi_rust_content := "
// chimera_abi crate - Chimera ABI runtime support for Rust
// Automatically generated from ChimeraProof canonical structs.

use std::os::raw::c_int;

/// Chimera ABI version
pub const CHIMERA_ABI_VERSION: u32 = 1;

/// Status code where 0 = success, non-zero = error
pub type Status = c_int;

/// Status code constants
pub const STATUS_OK: Status = 0;
pub const STATUS_ERR_MASK: Status = 0x80000000;

/// Error domain IDs
pub const DOMAIN_NONE: u32 = 0;
pub const DOMAIN_CHIMERA: u32 = 1;
pub const DOMAIN_C_ERRNO: u32 = 2;
pub const DOMAIN_RUST_RESULT: u32 = 3;
pub const DOMAIN_RUST_PANIC: u32 = 4;
pub const DOMAIN_ZIG_ERROR: u32 = 5;
pub const DOMAIN_ZIG_PANIC: u32 = 6;
pub const DOMAIN_USER: u32 = 256;

/// Panic policy
#[repr(u32)]
pub enum PanicPolicy {
    Forbidden = 0,
    Catch = 1,
    Abort = 2,
}

/// Safety class
#[repr(u32)]
pub enum SafetyClass {
    Verified = 0,
    Generated = 1,
    Trusted = 2,
    Unsafe = 3,
}

/// Allocator kind
#[repr(u32)]
pub enum AllocatorKind {
    System = 0,
    Null = 1,
    Shared = 2,
    LanguageOwned = 3,
    Custom = 4,
}

/// Effect flags
#[repr(u32)]
pub struct EffectFlags(u32);

impl EffectFlags {
    pub const NONE: EffectFlags = EffectFlags(0);
    pub const MAY_ERROR: EffectFlags = EffectFlags(1);
    pub const MAY_PANIC: EffectFlags = EffectFlags(2);
    pub const MAY_ALLOC: EffectFlags = EffectFlags(4);
    pub const MAY_DEALLOC: EffectFlags = EffectFlags(8);
    pub const MAY_BLOCK: EffectFlags = EffectFlags(16);
    pub const MAY_FFI: EffectFlags = EffectFlags(32);
}

/// Error structure
#[repr(C)]
pub struct ChError {
    pub domain: u32,
    pub code: u32,
    pub flags: u32,
    pub message_ptr: *const std::os::raw::c_void,
    pub message_len: u64,
    pub payload_ptr: *const std::os::raw::c_void,
    pub payload_drop_fn: Option<extern \"C\" fn(*mut std::os::raw::c_void)>,
    pub payload_drop_ctx: *mut std::os::raw::c_void,
}

/// Allocator structure
#[repr(C)]
pub struct ChAllocator {
    pub id: u64,
    pub kind: u32,
    pub ptr: *mut std::os::raw::c_void,
}

/// Slice structure
#[repr(C)]
pub struct ChSlice {
    pub ptr: *mut std::os::raw::c_void,
    pub len: u64,
}

/// Borrow structure
#[repr(C)]
pub struct ChBorrow {
    pub ptr: *mut u8,
    pub len: u64,
    pub lifetime: u32,
}

/// Owned bytes structure
#[repr(C)]
pub struct ChOwnedBytes {
    pub ptr: *mut u8,
    pub len: u64,
    pub capacity: u64,
    pub allocator_id: u64,
}

/// Handle structure
#[repr(C)]
pub struct ChHandle {
    pub ptr: *mut std::os::raw::c_void,
    pub drop_fn: Option<extern \"C\" fn(*mut std::os::raw::c_void)>,
    pub size: u64,
}
"

/--
Zig comptime ABI constants.
-/
def chimera_abi_zig_content := "// chimera_abi.zig - Chimera ABI runtime support for Zig
// Automatically generated from ChimeraProof canonical structs.

pub const CHIMERA_ABI_VERSION: u32 = 1;

pub const STATUS_OK: i32 = 0;
pub const STATUS_ERR_MASK: i32 = -2147483648; // 0x80000000

pub const DOMAIN_NONE: u32 = 0;
pub const DOMAIN_CHIMERA: u32 = 1;
pub const DOMAIN_C_ERRNO: u32 = 2;
pub const DOMAIN_RUST_RESULT: u32 = 3;
pub const DOMAIN_RUST_PANIC: u32 = 4;
pub const DOMAIN_ZIG_ERROR: u32 = 5;
pub const DOMAIN_ZIG_PANIC: u32 = 6;
pub const DOMAIN_USER: u32 = 256;

pub const PANIC_FORBIDDEN: u32 = 0;
pub const PANIC_CATCH: u32 = 1;
pub const PANIC_ABORT: u32 = 2;

pub const SAFETY_VERIFIED: u32 = 0;
pub const SAFETY_GENERATED: u32 = 1;
pub const SAFETY_TRUSTED: u32 = 2;
pub const SAFETY_UNSAFE: u32 = 3;

pub const ALLOC_SYSTEM: u32 = 0;
pub const ALLOC_NULL: u32 = 1;
pub const ALLOC_SHARED: u32 = 2;
pub const ALLOC_LANG_OWNED: u32 = 3;
pub const ALLOC_CUSTOM: u32 = 4;

pub const EFF_NONE: u32 = 0;
pub const EFF_MAY_ERROR: u32 = 1;
pub const EFF_MAY_PANIC: u32 = 2;
pub const EFF_MAY_ALLOC: u32 = 4;
pub const EFF_MAY_DEALLOC: u32 = 8;
pub const EFF_MAY_BLOCK: u32 = 16;
pub const EFF_MAY_FFI: u32 = 32;

// Error structure
pub const Error: type = extern struct {
    domain: u32,
    code: u32,
    flags: u32,
    message_ptr: [*]const u8,
    message_len: u64,
    payload_ptr: ?*anyopaque,
    payload_drop_fn: ?fn (?*anyopaque) callconv(.C) void,
    payload_drop_ctx: ?*anyopaque,
};

// Allocator structure
pub const Allocator: type = extern struct {
    id: u64,
    kind: u32,
    ptr: ?*anyopaque,
};

// Slice structure
pub const Slice: type = extern struct {
    ptr: [*]const u8,
    len: u64,
};

// Borrow structure
pub const Borrow: type = extern struct {
    ptr: [*]const u8,
    len: u64,
    lifetime: u32,
};

// Owned bytes structure
pub const OwnedBytes: type = extern struct {
    ptr: [*]u8,
    len: u64,
    capacity: u64,
    allocator_id: u64,
};

// Handle structure
pub const Handle: type = extern struct {
    ptr: ?*anyopaque,
    drop_fn: ?fn (?*anyopaque) callconv(.C) void,
    size: u64,
};
"

end Chimera.Runtime
