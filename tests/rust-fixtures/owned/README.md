# Ownership Rust FFI Fixture

This fixture provides `ChHandle` and `ChOwnedBytes` types for testing the Chimera Rust ownership/allocator lowering pipeline.

## Purpose

The ownership fixture is used to verify the complete Rust ownership lowering:

1. Parse Rust source with `chimera-rust-source`
2. Extract ownership facts with `chimera-rust-ownership`
3. Lower to ChimeraIR with `chimera-rust-to-chimera` using `ch_handle`/`ch_owned_bytes`
4. Verify through `compiler-core` with drop trampolines and allocator metadata

## ChHandle Type

The `ChHandle` type represents an owned handle with drop trampoline:

| Field | Type | Description |
|-------|------|-------------|
| `ptr` | `*mut c_void` | Pointer to allocated memory |
| `size` | `usize` | Size of allocation |
| `drop_trampoline` | `Option<unsafe fn(*mut c_void, usize)>` | Cleanup function |

## ChOwnedBytes Type

The `ChOwnedBytes` type represents owned byte buffer:

| Field | Type | Description |
|-------|------|-------------|
| `ptr` | `*mut u8` | Pointer to data |
| `len` | `usize` | Number of valid bytes |
| `capacity` | `usize` | Total capacity |

## Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `handle_create` | `fn(usize) -> ChHandle` | Create handle with given size |
| `handle_drop` | `fn(ChHandle) -> ()` | Drop handle using trampoline |
| `handle_size` | `fn(ChHandle) -> usize` | Get handle size |
| `handle_is_valid` | `fn(ChHandle) -> bool` | Check if handle is non-null |
| `owned_bytes_from_ptr` | `fn(*mut u8, usize, usize) -> ChOwnedBytes` | Create from raw pointer |
| `owned_bytes_len` | `fn(ChOwnedBytes) -> usize` | Get length |
| `owned_bytes_capacity` | `fn(ChOwnedBytes) -> usize` | Get capacity |
| `owned_bytes_is_empty` | `fn(ChOwnedBytes) -> bool` | Check if empty |
| `owned_bytes_get` | `fn(ChOwnedBytes, usize) -> u8` | Read byte at index |
| `owned_bytes_from_c_string` | `fn(*const u8, usize) -> ChOwnedBytes` | Create from C string |
| `owned_bytes_free` | `fn(ChOwnedBytes) -> ()` | Free owned bytes |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```