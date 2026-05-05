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
