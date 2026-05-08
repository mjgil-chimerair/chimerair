# Rust ChimeraIR Fixture

This fixture provides a minimal Rust library for testing the Rust-to-ChimeraIR unified lowering path.

## Purpose

- Tests the `RustLowerToChimera` build node in `chimera-build`
- Validates that Rust source can be parsed and lowered to ChimeraIR textual format
- Exercises `#[no_mangle]` + `extern "C"` function exports
- Tests struct lowering with `#[repr(C)]`

## Building

The fixture is built using the unified lowering path (not native archive):

```bash
# Lower to ChimeraIR
cargo build --features std

# Or via chimera-cli when available
```

## Expected Artifacts

- `target/debug/librust_chimera_fixture.chimera` - ChimeraIR textual output
- `target/debug/librust_chimera_fixture.rsnap` - Rust snapshot for incremental builds

## Exported Symbols

- `add`, `subtract`, `multiply`, `divide` - basic arithmetic
- `max`, `min` - comparison operations
- `negate`, `is_zero` - unary operations
- `ZERO`, `ONE` - static constants
- `Point2D` - struct type
- `point_distance`, `point_origin` - struct operations