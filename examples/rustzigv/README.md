# rustzigv Chimera example

This example demonstrates compiling the rustzigv Rust workspace through ChimeraIR.

## rustzigv Background

rustzigv is a V programming language compiler that combines Zig and Rust:
- **Zig side**: Frontend (lexer, parser, AST, diagnostics, HIR, codegen_c)
- **Rust side**: Semantic engine (Sema, HIR, type checking, resolution, MIR, etc.)

The Rust workspace has 14+ crates:
- `rustzigv-core` - Core semantic engine (cdylib/staticlib)
- `rustzigv-ffi` - FFI bindings to Rust
- `rustzigv-sema`, `rustzigv-hir`, `rustzigv-typeck`, `rustzigv-resolve` - Semantic analysis
- `rustzigv-ir`, `rustzigv-vtir`, `rustzigv-mir` - IR representations
- `rustzigv-abi` - ABI handling
- `rustzigv-passes` - Pass manager and optimizations
- `rustzigv-lints` - Linting
- `rustzigv-diagnostics`, `rustzigv-source` - Diagnostics and source handling
- `rustzigv-cli` - CLI wrapper
- `rustzigv-test-support` - Test utilities

## Chimera Integration

This example shows how rustzigv's Rust workspace can be compiled through Chimera's
polyglot MLIR pipeline instead of directly with `cargo build --workspace`.

The build flow would be:
1. Chimera reads `rust/Cargo.toml` via `chimera-rust-cargo`
2. Extracts workspace metadata, crate types, dependencies
3. For cdylib crates, captures FFI boundaries
4. Compiles through MLIR pipeline to `.chimera` format
5. Links with Zig/C components for final binary

## Building

```bash
cd chimerair
cargo build --release -p chimera-cli
./target/release/chimera build --manifest examples/rustzigv/Chimera.toml --output examples/rustzigv/build
```

## Notes

The checked-in manifest uses a placeholder path for the external `rustzigv`
workspace. Update `examples/rustzigv/Chimera.toml` to point at your local
`rustzigv` checkout before running the example.

rustzigv is primarily built using Zig's build system (`build.zig`), which:
1. First runs `cargo build --workspace` for the Rust crates
2. Then compiles Zig modules that link against the Rust cdylibs

Chimera could replace the cargo build step with its own MLIR-based compilation
and could potentially be used to link the final rustzigv binary as well.
