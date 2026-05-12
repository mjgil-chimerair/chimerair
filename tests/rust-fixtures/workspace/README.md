# Cargo Workspace Rust FFI Fixture

This fixture provides a multi-crate workspace for testing the Chimera Rust Cargo integration handling.

## Purpose

The workspace fixture is used to verify Cargo integration behavior:

1. Parse Rust source with `chimera-rust-source`
2. Extract Cargo metadata with `chimera-rust-cargo`
3. Lower to ChimeraIR with `chimera-rust-to-chimera`
4. Verify through `compiler-core` with workspace-aware handling

## Structure

```
workspace/
├── Cargo.toml          # Workspace manifest
├── main-crate/         # Main crate with FFI exports
│   ├── build.rs        # Build script
│   ├── Cargo.toml
│   └── src/lib.rs
├── helper-crate/       # Dependency crate
│   ├── Cargo.toml
│   └── src/lib.rs
└── macro-crate/        # Proc-macro crate
    ├── Cargo.toml
    └── src/lib.rs
```

## Crates

### main-crate

Main crate with FFI exports. Depends on helper-crate and macro-crate.

**Exported Functions:**
- `workspace_add(a, b)` - Call helper_add and wrap result
- `workspace_mul(a, b)` - Call helper_mul and wrap result
- `workspace_pipeline(a, b)` - Negate then add
- `workspace_features()` - Get feature flags
- `workspace_identity(a)` - Pass-through wrapper

### helper-crate

Dependency crate with basic math functions and feature flags.

**Exported Functions:**
- `helper_add(a, b)` - Add two numbers
- `helper_mul(a, b)` - Multiply two numbers
- `helper_neg(a)` - Negate a value
- `get_feature_flags()` - Get active feature flags (std=1, nightly=2)

### macro-crate

Proc-macro crate exporting `#[with_panic_handler]` attribute and `make_wrapper!` macro.

**Exported Macros:**
- `#[with_panic_handler]` - Attribute macro wrapping functions
- `make_wrapper!` - Generates a Wrapper struct

## WorkspaceResult Type

| Field       | Type   | Description |
|-------------|--------|-------------|
| value       | i32    | Result value |
| from_helper | bool   | Whether result came from helper crate |
| build_ran   | bool   | Whether build.rs was executed |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
cargo test -p helper-crate
```