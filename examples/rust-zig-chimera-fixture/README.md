# Rust+Zig Mixed ChimeraIR Fixture

A minimal mixed-language fixture for testing unified Rust+Zig lowering to ChimeraIR.

## Purpose

This fixture demonstrates cross-language function calls between Rust and Zig at the ChimeraIR level:
- `rust/` directory contains a Rust library that exports simple math functions (`rust_add`, `rust_multiply`, etc.)
- `zig/` directory contains a Zig library that imports and re-exports those Rust functions via `extern fn` declarations

## Structure

```
rust-zig-chimera-fixture/
├── rust/               # Rust library
│   ├── Cargo.toml
│   └── src/lib.rs      # Exports rust_add, rust_subtract, etc.
├── zig/                # Zig library
│   ├── build.zig
│   └── src/main.zig    # Imports rust_* functions, exports zig_* wrappers
└── README.md
```

## ChimeraIR Merge

When built with unified lowering:
1. Rust library lowers to `rust_lib.chimera` with exports: `rust_add`, `rust_subtract`, etc.
2. Zig library lowers to `zig_lib.chimera` with imports: `import:rust_add`, `import:rust_subtract`, etc.
3. The MergeChimera node combines both, resolving imports against exports

## Building

```bash
# Build Rust library
cd rust && cargo build --lib

# Build Zig library
cd ../zig && zig build
```

## Expected Outputs

After ChimeraIR merge, the unified module should contain:
- All Rust exports (with C ABI)
- All Zig exports (wrapping Rust calls)
- Resolved import/export pairs

## Test Coverage

- Cross-language function call resolution
- ABI compatibility between Rust "C" and Zig ".c" calling conventions
- Merge diagnostics for unresolved imports
