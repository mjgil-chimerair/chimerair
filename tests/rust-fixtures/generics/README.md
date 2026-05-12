# Generics Rust FFI Fixture

This fixture provides generic functions and monomorphized wrappers for testing the Chimera Rust generics handling at FFI boundaries.

## Purpose

The generics fixture is used to verify monomorphization behavior:

1. Parse Rust source with `chimera-rust-source`
2. Extract effect facts with `chimera-rust-effects`
3. Lower to ChimeraIR with `chimera-rust-to-chimera`
4. Verify through `compiler-core` with concrete monomorphized types

## Type Tags

| Tag      | Value | Description |
|----------|-------|-------------|
| TYPE_TAG_I64 | 1 | i64 monomorph |
| TYPE_TAG_U64 | 2 | u64 monomorph |
| TYPE_TAG_I32 | 3 | i32 monomorph |
| TYPE_TAG_U32 | 4 | u32 monomorph |

## Types

### IdentityResult

| Field     | Type   | Description |
|-----------|--------|-------------|
| value     | u64    | Identity result value |
| type_tag  | u8     | Monomorph type tag |

### GenericPair

| Field     | Type   | Description |
|-----------|--------|-------------|
| first     | u64    | First value |
| second    | u64    | Second value |
| type_tag  | u8     | Monomorph type tag |

### GenericContainer

| Field     | Type   | Description |
|-----------|--------|-------------|
| value     | u64    | Contained value |
| is_some   | bool   | Whether value is present |
| type_tag  | u8     | Monomorph type tag |

## Exported Functions

| Function                  | Signature                              | Description |
|---------------------------|----------------------------------------|-------------|
| `identity_i64`            | `fn(i64) -> IdentityResult`            | Identity for i64 |
| `identity_u64`            | `fn(u64) -> IdentityResult`            | Identity for u64 |
| `identity_i32`            | `fn(i32) -> IdentityResult`            | Identity for i32 |
| `identity_u32`            | `fn(u32) -> IdentityResult`            | Identity for u32 |
| `swap_u64_u64`            | `fn(u64, u64) -> GenericPair`         | Swap (u64, u64) |
| `swap_i64_u64`            | `fn(i64, u64) -> GenericPair`          | Swap (i64, u64) |
| `swap_u32_u32`            | `fn(u32, u32) -> GenericPair`          | Swap (u32, u32) |
| `container_i64_new`       | `fn(i64) -> GenericContainer`         | Container for i64 |
| `container_u64_new`       | `fn(u64) -> GenericContainer`         | Container for u64 |
| `container_none`          | `fn() -> GenericContainer`             | Empty container |
| `container_i64_eq`       | `fn(GenericContainer, GenericContainer) -> bool` | Compare i64 containers |
| `container_u64_eq`        | `fn(GenericContainer, GenericContainer) -> bool` | Compare u64 containers |
| `max_i64`                 | `fn(i64, i64) -> i64`                  | Maximum of i64 |
| `max_u64`                 | `fn(u64, u64) -> u64`                  | Maximum of u64 |
| `max_i32`                 | `fn(i32, i32) -> i32`                  | Maximum of i32 |
| `max_u32`                 | `fn(u32, u32) -> u32`                  | Maximum of u32 |
| `min_i64`                 | `fn(i64, i64) -> i64`                  | Minimum of i64 |
| `min_u64`                 | `fn(u64, u64) -> u64`                  | Minimum of u64 |
| `array_i64_zeros`         | `fn(usize) -> *mut u64`                | Create i64 array |
| `array_u64_zeros`         | `fn(usize) -> *mut u64`                | Create u64 array |
| `array_i64_get`           | `fn(*mut u64, usize, usize) -> u64`   | Get i64 element |
| `array_u64_get`           | `fn(*mut u64, usize, usize) -> u64`   | Get u64 element |
| `array_i64_free`          | `fn(*mut u64, usize)`                  | Free i64 array |
| `array_u64_free`          | `fn(*mut u64, usize)`                  | Free u64 array |

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```