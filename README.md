# ChimeraIR

ChimeraIR is a polyglot compiler infrastructure project that composes C, Rust, and Zig through a shared intermediate representation, runtime ABI, and proof-oriented validation layer.

## Status

ChimeraIR is under active development. The project includes a Lean 4 proof system (`ChimeraProof/`), compiler core (`compiler-core/`), Rust tooling (`tools/`), runtime headers (`runtime/`), and integration examples (`examples/`).

## Repository Layout

- `ChimeraProof/`: Lean 4 proof models and validation logic
- `compiler-core/`: C++ and MLIR compiler components
- `tools/`: Rust CLI, adapters, cache, and build orchestration
- `runtime/`: shared ABI headers and language-specific runtime support
- `examples/`: sample C, Rust, and Zig integration projects
- `docs/`: architecture, build, testing, and design documentation

## Building

```bash
./build.sh
```

To build the shared Chimera CLI directly:

```bash
cd tools
cargo build --release -p chimera-cli
```

That produces `tools/target/release/chimera`, which is the entrypoint used for
the three current binary variants in `../chimera-beam`:

```bash
HOST_TRIPLE=x86_64-unknown-linux-gnu
CHIMERA=tools/target/release/chimera

cd ../chimera-beam
"$CHIMERA" build --manifest Chimera.toml --target "$HOST_TRIPLE" --output ./build-abi
"$CHIMERA" build --manifest Chimera.adapter.toml --target "$HOST_TRIPLE" --output ./build-adapter
"$CHIMERA" build --manifest Chimera.separate.toml --target "$HOST_TRIPLE" --output ./build-semantic
```

These correspond to:

- ABI binary via `Chimera.toml`
- Chimera adapter binary via `Chimera.adapter.toml`
- Chimera semantic binary via `Chimera.separate.toml`

Each build currently emits `chimera_binary` in the selected output directory.

## Testing

```bash
./test.sh
```

## Key Documentation

- [Getting Started](docs/getting-started.md)
- [Build Guide](docs/build.md)
- [Testing Guide](docs/testing.md)
- [Repository Layout](docs/repo-layout.md)
- [Architecture Overview](docs/architecture.md)
- [Final Design](docs/chimerair-final-design.md)
- [CLI Guide](docs/cli-guide.md)

## License

This repository is licensed under the `0BSD` license. See `LICENSE`.
