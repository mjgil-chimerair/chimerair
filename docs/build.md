# Chimera Build Guide

## Top-Level Build

For a full build of all components (proof, compiler, tools, runtime):

```bash
./scripts/build-all.sh
```

For specific components:

```bash
./scripts/build-all.sh --skip-proof --skip-compiler  # Rust tools + runtime only
./scripts/build-all.sh --build-type debug           # Debug build
./scripts/build-all.sh -j 4                          # 4 parallel jobs
```

Generated repository-pack artifacts such as `repomix-output.xml` are not part
of the tracked source tree. If you generate them locally for analysis, treat
them as disposable outputs rather than checked-in build inputs.

## Top-Level Test

To run all tests:

```bash
./test.sh

# Or use the configurable orchestrator
./scripts/test-all.sh
```

For specific test suites:

```bash
./scripts/test-all.sh --test runtime    # Runtime tests only
./scripts/test-all.sh --test tools      # Tools tests only
./scripts/test-all.sh --skip-proof      # Skip proof tests
```

---

## ChimeraProof Build Guide

### Prerequisites

- Lean 4.29.1 or later
- Lake (included with Lean)
- CMake 3.25+
- Ninja build system
- LLVM 17 with MLIR
- GCC 11+ or Clang 17+
- Rust 1.75+ (stable)
- Zig 0.13+ (optional)

### Toolchain Pinning

All toolchain versions are pinned in `toolchain.toml` and `toolchains/versions.toml`:

```bash
# Verify your toolchains match pinned versions
bash toolchains/setup-toolchains.sh

# Supported platforms (from toolchain.toml)
linux_x86_64       = "x86_64-unknown-linux-gnu"
linux_aarch64      = "aarch64-unknown-linux-gnu"
macos_x86_64       = "x86_64-apple-darwin"
macos_aarch64      = "aarch64-apple-darwin"
windows_x86_64     = "x86_64-pc-windows-gnu"
wasm32             = "wasm32-unknown-unknown"
```

Verify your installation:

```bash
lean --version
lake --version
cmake --version
ninja --version
```

## Compiler-Core Build Guide

### Prerequisites

- LLVM 17.0.6 with MLIR (exact version pinned)
- CMake 3.25+
- Ninja build system

### Building

```bash
mkdir -p compiler-core/build
cd compiler-core/build
cmake .. -G Ninja \
  -DLLVM_DIR=/usr/lib/llvm-17/lib/cmake/llvm \
  -DMLIR_DIR=/usr/lib/llvm-17/lib/cmake/mlir
ninja
```

### Verifying Build

```bash
# Verify chimerac driver was built
ls -la compiler-core/build/bin/chimerac

# Rebuild the registered compiler smoke targets and run them
cd compiler-core/build
cmake --build .
ctest --output-on-failure
```

The top-level test drivers rebuild `compiler-core/build` before running `ctest`
and treat `Total Tests: 0` as a failure once `compiler-core/build` exists, so a
configured compiler build must register its ctest suite correctly. The current
registered coverage exercises the `chimerac` driver and sidecar emission path
directly.

The current production lowering entrypoint is the driver flag:

```bash
cd compiler-core/build
./bin/chimerac --lower-llvm ../test/Dialect/Lowering/llvm-lowering.mlir
```

That path lowers the current func/arith/control-flow surface to textual LLVM
dialect MLIR and is covered by the `chimerac-lower-llvm-smoke` ctest.

The current production fuzz-smoke entrypoint is:

```bash
cd compiler-core/build
./bin/chimera-fuzz-smoke
```

That path runs the parser, metadata, and C API fuzz-input handlers on
representative byte samples and is covered by the `chimera-fuzz-smoke` ctest.

The current production benchmark-smoke entrypoint is:

```bash
cd compiler-core/build
./bin/chimera-benchmark-smoke
```

That path runs benchmark coverage for parse, verify, canonicalization passes,
LLVM lowering, and textual emission on a representative module and is covered
by the `chimera-benchmark-smoke` ctest.

The current production pass-verification smoke entrypoint is:

```bash
cd compiler-core/build
./bin/chimera-pass-smoke
```

That path exercises the tracked verification compatibility pipeline plus the
current `check-only`, `wrapper-gen`, `object-emit`, and `proof-obligations`
compiler-core pass presets and is covered by the `chimera-pass-smoke` ctest.

### Clean Build

```bash
rm -rf compiler-core/build
mkdir compiler-core/build
# Then rebuild as above
```

## ChimeraProof Build Guide

From the `ChimeraProof/` directory:

```bash
lake build
```

## Running Tests

```bash
cd ChimeraProof
./test.sh
```

This runs the smoke test suite via `lake build` which verifies all modules import and type-check correctly.

Note: The formal `lake test` command requires resolving circular dependencies between
`Chimera.IR`, `Chimera.Checkers`, and `Chimera.Link` modules. The build-based test
verification via `./test.sh` is the current working approach.

At the repo root, `./test.sh` uses that build-based proof check, then runs the root
placeholder gate, the Rust tools workspace tests, runtime conformance, and the
runtime sanitizer smoke/conformance path.

To run the sanitizer-specific runtime validation directly:

```bash
cd runtime
bash test_sanitizers.sh
```

## Cleaning

```bash
cd ChimeraProof
lake clean
```

## Module Structure

```
ChimeraProof/
├── Chimera/           -- Core proof system modules
│   ├── Foundation/    -- Word, Bytes, Symbol, Target, Alignment, FinMap
│   ├── ABI/          -- Type, PhysicalType, Layout, Lowering, Signature, Contract
│   ├── Memory/       -- Block, Pointer, Permission, Ownership, Borrow, Allocator, Drop
│   ├── Error/        -- Status, ErrorDomain, ChError, Panic, Bridge
│   ├── Effects/      -- EffectLattice, Inference, Composition
│   ├── IR/           -- Module, Operation, WellFormed, Passes
│   ├── Link/         -- SymbolTable, Resolve, Compose
│   ├── Checkers/     -- Metadata, Layout, Ownership, Allocator, Result, Panic, Full
│   ├── Wrapper/      -- AST, Generator
│   ├── Theorems/     -- Soundness, EndToEnd
│   └── Tests/        -- Smoke tests (not yet wired to lake test)
├── docs/
└── lakefile.toml
```

## Common Build Issues

### Unknown namespace errors
If you see "unknown namespace" errors, check that imports reference correct module paths. Submodule namespaces must be imported before use.

### Sorry placeholders
The codebase contains `sorry` placeholders for unproved theorems. CI rejects `sorry` in production code.
