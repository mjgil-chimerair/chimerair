# Getting Started with Chimera

This guide walks you through building, testing, and using the Chimera polyglot IR toolchain.

## Prerequisites

- Rust 1.70+ (for `cargo` and the tools workspace)
- Lean 4 (for proof verification)
- LLVM/MLIR (for compiler-core)
- Zig (optional, for Zig module support)
- Git

## Clone and Setup

```bash
git clone https://github.com:mjgil/chimerair.git
cd chimerair
```

## Building the Rust Tools

The Rust tools are located in `tools/` and include:
- `chimera-cli` - Main CLI entrypoint
- `chimera-meta` - Metadata schema modeling
- `chimera-diagnostics` - Diagnostic codes and rendering
- `chimera-wrappergen` - Wrapper generation
- `chimera-proof-bridge` - Proof verification bridge
- `chimera-adapter-c/rust/zig` - Language adapters

To build all tools:

```bash
cd tools
cargo build --release
```

This produces the `chimera` binary at `target/release/chimera`.

## Running Tests

Run all workspace tests:

```bash
cd tools
cargo test --workspace
```

Run tests for a specific crate:

```bash
cargo test -p chimera-wrappergen
cargo test -p chimera-adapter-c
cargo test -p chimera-adapter-rust
cargo test -p chimera-adapter-zig
```

## Building the One-Binary Example

The examples directory contains a demo that compiles C, Rust, and Zig into one binary:

```bash
cd examples/one-binary
./build.sh
```

This builds:
- `c-reader/` - C file reader component
- `rust-config/` - Rust config parser
- `zig-checksum/` - Zig checksum module

## Using the CLI

### Check a project

```bash
chimera check --manifest Chimera.toml
```

### Build a project

```bash
chimera build --manifest Chimera.toml --output build/
```

### Link artifacts

```bash
chimera link object1.o object2.o --output mybinary
```

### Explain diagnostics

```bash
chimera explain proof_report.json
```

## Running the Test Suite

Run all Rust tool tests:

```bash
cd tools
cargo test --workspace --release
```

Run with verbose output:

```bash
cargo test --workspace -- --nocapture
```

## Proof Bridge

The proof bridge integrates with Lean to verify proof obligations. See `tools/crates/chimera-proof-bridge/` for details.

## Architecture Overview

- `ChimeraProof/` - Lean proof system
- `compiler-core/` - C++/MLIR compiler
- `tools/` - Rust toolchain workspace
- `runtime/` - C/Rust/Zig ABI implementations
- `examples/` - Demo applications

## Troubleshooting

### "cannot find Cargo.toml"

Make sure you're in the `tools/` directory when running cargo commands.

### "lean not found"

Install Lean 4 via:
```bash
lakefile.toml requires `lake` - install via `elan` or `nix`
```

### Build errors

Ensure you have the latest Rust stable:
```bash
rustup update
cargo build --workspace
```