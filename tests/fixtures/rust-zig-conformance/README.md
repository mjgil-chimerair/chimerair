# Rust+Zig Conformance Fixture

> Task 48: Product-style conformance fixture for mixed-language product builds

## Purpose

This fixture provides a product-style example for Rust+Zig unified lowering. It demonstrates cross-language function calls where a Zig wrapper library imports and re-exports functions from a Rust math library.

## Structure

```
rust-zig-conformance/
├── Chimera.toml          # Chimera build manifest
├── rust/                 # Rust library
│   ├── Cargo.toml
│   └── src/lib.rs        # Exports: rust_add, rust_subtract, etc.
└── zig/                  # Zig library
    ├── build.zig
    └── src/main.zig      # Imports rust_* via extern, exports zig_*
```

## Components

### rust_math (Rust library)
- `rust_add(a, b)` - Add two i32 values
- `rust_subtract(a, b)` - Subtract two i32 values
- `rust_multiply(a, b)` - Multiply two i32 values
- `rust_divide(a, b)` - Divide two i32 values (panics on divide by zero)
- `rust_max(a, b)` - Return maximum of two i32 values
- `rust_min(a, b)` - Return minimum of two i32 values
- `rust_negate(a)` - Negate an i32 value
- `rust_is_zero(a)` - Check if i32 value is zero

### zig_wrapper (Zig library)
- `zig_add(a, b)` - Delegates to `rust_add`
- `zig_subtract(a, b)` - Delegates to `rust_subtract`
- `zig_multiply(a, b)` - Delegates to `rust_multiply`
- `zig_divide(a, b)` - Delegates to `rust_divide`
- `zig_max(a, b)` - Delegates to `rust_max`
- `zig_min(a, b)` - Delegates to `rust_min`
- `zig_negate(a)` - Delegates to `rust_negate`
- `zig_is_zero(a)` - Delegates to `rust_is_zero`
- `zig_combined_op(a, b, c)` - Computes `(a + b) * c`
- `zig_complex_op(a, b)` - Computes `max(a, b) + min(a, b)`

## Building

```bash
# Build with unified lowering (default)
cargo build --manifest-path rust/Cargo.toml
zig build --prefix zig/

# Or use chimera CLI if available
chimerac build --config Chimera.toml
```

## Expected Artifacts

After unified lowering and LLVM emission:
- Unified ChimeraIR module with merged Rust+Zig functions
- LLVM IR with cross-language inlining decisions applied
- Final native binary with Rust and Zig code linked together

## Test Coverage

- Cross-language function call resolution (import:rust_* resolved)
- ABI compatibility verification (C calling convention)
- Merge diagnostics for unresolved imports
- Optimization decisions (inline eligibility, constant propagation)
- Effect barrier placement for panic-capable functions (rust_divide)

## Relationship to Other Fixtures

| Fixture | Purpose |
|---------|---------|
| `rust-chimera-fixture` | Minimal Rust-only unified lowering |
| `zig-chimera-fixture` | Minimal Zig-only unified lowering |
| `rust-zig-chimera-fixture` | Minimal mixed Rust+Zig fixture (same dir) |
| `rust-zig-conformance` | **Product-style conformance fixture (this)** |

The `rust-zig-conformance` fixture is more comprehensive than `rust-zig-chimera-fixture`:
- Includes a `Chimera.toml` manifest for full build system testing
- More complex Zig functions that chain multiple Rust calls
- Designed for conformance testing rather than basic merge verification