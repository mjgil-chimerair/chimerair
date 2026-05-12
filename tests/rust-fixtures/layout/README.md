# Layout Rust FFI Fixture

This fixture provides `#[repr(C)]` structs for testing the Chimera Rust layout extraction pipeline.

## Purpose

The layout fixture is used to verify the complete Rust layout extraction and verification:

1. Parse Rust source with `chimera-rust-source`
2. Extract layout facts with `chimera-rust-layout`
3. Lower to ChimeraIR with `chimera-rust-to-chimera`
4. Verify through `compiler-core` with generated layout assertions

## Exported Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `point_distance` | `fn(Point2D, Point2D) -> f64` | Calculate distance between two points |
| `rectangle_area` | `fn(Rectangle) -> f64` | Calculate area of a rectangle |
| `point_in_rectangle` | `fn(Point2D, Rectangle) -> bool` | Check if point is inside rectangle |
| `size_of_point` | `fn() -> usize` | Get size of Point2D (16 bytes) |
| `align_of_point` | `fn() -> usize` | Get alignment of Point2D (8 bytes) |
| `size_of_mixed` | `fn() -> usize` | Get size of MixedFields (24 bytes) |
| `size_of_packed` | `fn() -> usize` | Get size of PackedStruct (6 bytes, no padding) |

## Structs

| Struct | Size | Alignment | Notes |
|--------|------|-----------|-------|
| `Point2D` | 16 | 8 | Two f64 fields |
| `MixedFields` | 24 | 8 | Has padding between fields |
| `Rectangle` | 32 | 8 | Nested Point2D |
| `AlignedStruct` | 8 | 8 | Uses #[repr(align(8))] implicit |
| `PackedStruct` | 6 | 1 | No padding (#[repr(C, packed)]) |
| `WithArray` | 28 | 8 | Contains [u8; 16] array |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```