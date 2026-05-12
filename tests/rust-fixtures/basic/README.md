# Basic Rust FFI Smoke Test Fixture

This fixture provides a minimal set of `#[no_mangle] extern "C"` functions for testing the Chimera Rust compiler integration pipeline.

## Purpose

The basic fixture is used to verify the complete Rust→ChimeraIR→compiler-core pipeline:

1. Parse Rust source with `chimera-rust-source`
2. Emit `.rsnap` semantic snapshots
3. Lower to Rust dialect with `chimera-rust-dialect`
4. Lower to ChimeraIR with `chimera-rust-to-chimera`
5. Verify through `compiler-core`

## Exported Functions

| Function   | Signature                    | Description                      |
| ---------- | --------------------------- | -------------------------------- |
| `add`      | `fn(i32, i32) -> i32`       | Add two i32 values               |
| `multiply` | `fn(i32, i32) -> i32`       | Multiply two i32 values          |
| `max`      | `fn(i32, i32) -> i32`       | Return the larger of two i32s    |
| `min`      | `fn(i32, i32) -> i32`       | Return the smaller of two i32s   |
| `negate`   | `fn(i32) -> i32`            | Negate an i32 value              |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Usage with Chimera CLI

```bash
# Parse and validate
chimera rust validate --input src/lib.rs

# Lower to ChimeraIR
chimera rust lower --input src/lib.rs --output test.chimera

# Verify through compiler-core
chimerac --input test.chimera --emit-metadata --emit-object
```

## Notes

- All functions use C calling convention (`extern "C"`)
- No panics, allocations, or FFI-unsafe operations
- No generics or trait bounds
- Suitable for stable (non-nightly) toolchain testing
