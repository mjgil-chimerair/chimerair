# Chimera Runtime

C/Rust/Zig ABI definitions and runtime support libraries.

## Status

**Tasks 54-56 Complete**: Canonical C header, Rust crate, and Zig module implemented.

## Components

### `include/` - C Headers

- `chimera_abi.h` - Canonical C ABI header with:
  - Status codes and error handling
  - Ownership and lifetime kinds
  - Slice and string types
  - Result types
  - Panic handling
  - Calling convention macros

### `rust/` - Rust ABI Support Crate

`chimera-rt` crate with `repr(C)` types:

- `Status`, `ErrorDomain`, `Ownership`, `Lifetime`, `CConv`
- `Slice`, `SliceMut`, `String`, `Result`
- `PanicPolicy`, `PanicInfo`
- `TargetArch`, `TargetOs`
- `AllocKind`, `AllocFn`

Features: `std` (default), `no_std`

### `zig/` - Zig ABI Support Module

- `chimera_abi.zig` - Zig extern structs matching C ABI layout

## Building

### Rust

```bash
cd rust
cargo build --release
cargo test --release
```

### C

```bash
gcc -Wall -Wextra -I../runtime/include your_file.c
```

## Version

Chimera ABI Version: 0.1.0