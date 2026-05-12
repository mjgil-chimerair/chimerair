# Slice/String Rust FFI Fixture

This fixture provides `ch_slice` and `ch_borrow_str` wrapper types for testing the Chimera Rust slice/string lowering pipeline.

## Purpose

The slices fixture is used to verify the complete Rust slice and string lowering:

1. Parse Rust source with `chimera-rust-source`
2. Extract ABI facts with `chimera-rust-abi`
3. Lower to ChimeraIR with `chimera-rust-to-chimera` using `ch_slice`/`ch_borrow_str`
4. Verify through `compiler-core`

## ChSlice Type

The `ChSlice` type represents a borrowed slice descriptor:

| Field | Type | Description |
|-------|------|-------------|
| `data` | `*const u8` | Pointer to slice data |
| `len` | `usize` | Number of elements |

## ChBorrowStr Type

The `ChBorrowStr` type represents a borrowed string descriptor:

| Field | Type | Description |
|-------|------|-------------|
| `data` | `*const u8` | Pointer to UTF-8 string data |
| `len` | `usize` | Number of bytes |

## Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `sum_bytes` | `fn(ChSlice) -> u64` | Sum bytes in slice |
| `count_byte` | `fn(ChSlice, u8) -> usize` | Count byte occurrences |
| `slice_starts_with` | `fn(ChSlice, ChSlice) -> bool` | Check slice prefix |
| `slice_ends_with` | `fn(ChSlice, ChSlice) -> bool` | Check slice suffix |
| `string_length` | `fn(ChBorrowStr) -> usize` | Get string byte length |
| `slice_reverse` | `fn(ChSlice) -> ChSliceResult` | Reverse slice (returns result) |
| `slice_equal` | `fn(ChSlice, ChSlice) -> bool` | Compare slices |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```