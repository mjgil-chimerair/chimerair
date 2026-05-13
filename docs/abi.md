# Chimera ABI Reference

This document describes the Chimera ABI (Application Binary Interface) for cross-language interoperability between C, Rust, and Zig.

## Overview

The Chimera ABI defines how functions and data are represented at the binary level when crossing language boundaries. It covers:
- Type representations (semantic and physical)
- Calling conventions
- Memory layout
- Error handling
- Ownership and borrowing semantics

## Type System

### Semantic Types (ChType)

Semantic types describe the high-level type system used in function contracts:

| Type | Description |
|------|-------------|
| `i8`, `i16`, `i32`, `i64` | Signed integers |
| `u8`, `u16`, `u32`, `u64` | Unsigned integers |
| `f32`, `f64` | Floating point |
| `status` | Chimera status code (0 = success) |
| `error` | Error code with domain |
| `allocator` | Allocator handle |
| `ptr τ` | Borrowed pointer to τ |
| `rawptr` | Unchecked raw pointer |
| `borrow τ` | Borrowed reference (call-lifetime) |
| `borrowMut τ` | Mutable borrow with lifetime |
| `owned τ` | Owned resource (caller transfers) |
| `out τ` | Output parameter (callee writes) |
| `inout τ` | In-out parameter (read/modify/write) |
| `slice τ` | Slice (pointer + length) |
| `str` | String (slice of bytes) |
| `opaque` | Opaque handle (resource type) |
| `result ok err` | Fallible result (status + out params) |

### Physical Types (PhysType)

Physical types describe the low-level memory representation:

| Constructor | Description |
|-------------|-------------|
| `.void` | No return value |
| `.int w s` | Integer of width w bits, signedness s |
| `.float w` | Float of width w bits (32 or 64) |
| `.ptr t` | Pointer to physical type t |
| `.array n t` | Array of n elements of type t |
| `.struct fields` | Struct with named fields |
| `.fnptr cc params ret` | Function pointer |

## Calling Conventions

The calling convention determines how arguments are passed:

| Convention | Platform | Description |
|------------|----------|-------------|
| `.sysv` | Linux, BSD, macOS (x86-64) | System V AMD64 ABI |
| `.windows` | Windows (x86-64) | Microsoft x64 ABI |
| `.aapcs` | ARM (32-bit) | ARM AAPCS ABI |
| `.wasm` | WebAssembly | WebAssembly ABI |

## Memory Layout

### Primitive Sizes

| Type | Size (bytes) | Alignment (bytes) |
|------|--------------|-------------------|
| i8/u8/f32 | 1 | 1 |
| i16/u16 | 2 | 2 |
| i32/u32/f64 | 4 | 4 |
| i64/u64 | 8 | 8 |
| pointer | 8 (64-bit) / 4 (32-bit) | ptr |

### Struct Layout

Structs are laid out sequentially with:
1. Each field aligned to its natural alignment
2. Padding added to maintain alignment
3. Total struct size padded to alignment of largest field

### Array Layout

Arrays are contiguous memory with no padding between elements.

## Error Handling

### Result Representation

Functions that can fail use the status/out-param convention:

```lean
// Fallible function signature
fn parse_config(path: Borrowed<Str>) -> Result<Owned<Config>, Error>

// Lowers to:
// - Return: ch_status (0 = success, non-zero = error)
// - out_ok: Pointer to Config (on success)
// - out_err: Pointer to ch_error (on failure)
```

### Status Codes

- `0` = Success
- Non-zero = Error (specific codes defined by error domain)

### Canonical Error Structure (ch_error)

```lean
ch_error {
  domain : u32      // ErrorDomain ID
  code   : u32      // Domain-specific error code
  flags  : u32      // Reserved
  msg    : pointer  // UTF-8 message (optional)
  len    : u32      // Message length (0 if no message)
  payload : pointer // Additional data (domain-specific)
  payload_len : u32
  drop_fn : pointer // Optional cleanup function
}
```

## Ownership Model

### Ownership Tags

| Tag | Description |
|-----|-------------|
| `.own` | Exclusive owner (can drop, mutate) |
| `.readBorrow` | Shared borrow (read-only) |
| `.writeBorrow` | Exclusive borrow (read-write) |
| `.raw` | Unchecked raw pointer |

### Lifetime Annotations

- `.call` — Borrow lives only during the call
- `.static` — Borrow outlives the call (e.g., static string)
- `.owner` — Borrow tied to owner's lifetime

### Drop Policy

Types indicate whether they require cleanup:

```lean
// Primitive types (no drop needed)
RequiresDrop = false

// Owned resources (drop needed)
RequiresDrop = true

// Result payloads (drop on error path)
RequiresDrop = true
```

## Canonical Structs

These structs are defined for cross-language boundaries:

| Struct | Purpose |
|--------|---------|
| `ch_status` | Return status code |
| `ch_error` | Extended error with domain, code, message |
| `ch_allocator` | Allocator handle |
| `ch_slice` | Pointer + length for byte slices |
| `ch_string` | UTF-8 string (ptr + len) |
| `ch_owned_bytes` | Owned byte buffer |
| `ch_handle` | Opaque resource handle |

## Lowering Rules

Semantic types lower to physical types:

| Semantic | Physical | Notes |
|----------|----------|-------|
| `i32` | `.int 32 .unsigned` | |
| `u64` | `.int 64 .unsigned` | |
| `f64` | `.float 64` | |
| `ptr τ` | `.ptr (lower τ)` | Target pointer width |
| `owned T` | `.ptr (lower T)` | Passed by pointer |
| `result ok err` | `.void` + out params | Status return |
| `slice T` | `.struct [ptr, len]` | Pointer + natural size |
| `str` | `.struct [ptr, len]` | UTF-8 bytes |

## Function Forms

Functions are classified by their ABI form:

| Form | Description | Error Handling |
|------|-------------|----------------|
| `.infallible` | Cannot fail | None |
| `.fallible` | Returns Result | ch_status + out params |
| `.constructor` | Creates owned resource | Ownership transfer |
| `.destructor` | Releases resources | None |
| `.callback` | Callable from foreign code | Platform convention |
| `.unsafeRaw` | Raw FFI with minimal safety | Caller responsibility |

## Trust Model

Functions have a trust classification:

| Class | Description |
|-------|-------------|
| `.verified` | Proven correct by ChimeraIR |
| `.generated` | Generated by trusted tool |
| `.trusted` | User-marked as trusted C code |
| `.unsafeContract` | Explicitly marked unsafe |

## Layout Schema (.chmeta)

Modules declare layouts via `.chmeta` JSON:

```json
{
  "layouts": {
    "my_struct": {
      "fields": [
        {"name": "id", "type": "i32", "offset": 0, "size": 4, "align": 4},
        {"name": "data", "type": "ptr", "offset": 8, "size": 8, "align": 8}
      ],
      "size": 16,
      "align": 8
    }
  }
}
```

## Target-Specific Notes

### x86_64 Linux (sysv)

- Integer: 1, 2, 4, 8 bytes
- Float: 4, 8 bytes
- Pointer: 8 bytes
- Aggregate: passed by value when ≤ 16 bytes, by reference otherwise

### x86_64 Windows

- Integer: 1, 2, 4, 8 bytes
- Float: 4, 8 bytes
- Pointer: 8 bytes
- First 4 args in RCX, RDX, R8, R9

### wasm32

- Integer: 1, 2, 4, 8 bytes
- Float: 4, 8 bytes
- Pointer: 4 bytes
- Linear memory only