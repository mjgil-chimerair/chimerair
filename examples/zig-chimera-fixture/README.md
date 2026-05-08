# Zig Chimera Fixture

Minimal Zig fixture for unified lowering to ChimeraIR.

This fixture exports simple `export` functions with C ABI for testing
the Zig-to-ChimeraIR lowering path without native archive emission.

## Building

```bash
cd examples/zig-chimera-fixture
zig build
```

## Features

- Export functions with C ABI
- Simple integer operations (add, subtract, multiply, divide)
- Comparison functions (max, min)
- Bitwise operations (negate, is_zero)
- Constants (ZERO, ONE)
- Struct lowering (Point2D with extern struct)
- Point distance calculation

## ChimeraIR Output

This fixture is designed to emit ChimeraIR directly via the lowering
pipeline in `chimera-zig-to-chimera` crate. The lowered output should
contain:
- Function signatures with C ABI
- Type definitions with layout facts
- Export symbol metadata
- Effect annotations (none for pure functions)

## Testing

```bash
zig build test
```