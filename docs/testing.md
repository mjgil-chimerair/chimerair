# Testing

This document describes the testing infrastructure for the ChimeraIR proof system.

## Top-Level Test System

For running all tests across all components:

```bash
# Run all tests
./test.sh

# Or use the richer orchestrator
./scripts/test-all.sh

# Run specific test suite
./scripts/test-all.sh --test runtime    # Runtime tests only
./scripts/test-all.sh --test tools      # Tools tests only
./scripts/test-all.sh --test proof      # Proof system tests only

# Skip specific suites
./scripts/test-all.sh --skip-proof --skip-compiler
```

See `docs/build.md` for more build and test options.

## Test Structure

### Test Organization

Tests are organized in two locations:

1. **`Chimera/Theorems/`** - Theorem tests proving correctness properties
   - `FoundationTest.lean` - Tests for Word, Target, Symbol, Bytes
   - `ABITest.lean` - Tests for Type, PhysicalType, Layout, Lowering
   - `MemoryTest.lean` - Tests for Pointer, Allocator, Block, Ownership
   - `ErrorTest.lean` - Tests for Status, ChError, Panic, Bridge
   - `EffectsTest.lean` - Tests for EffectLattice
   - `IRTest.lean` - Tests for Module, WellFormed
   - `LinkTest.lean` - Tests for Resolve, Compose
   - `CheckersTest.lean` - Tests for metadata, layout, ownership checkers
   - `WrapperTest.lean` - Tests for AST, Generator
   - `EndToEnd.lean` - End-to-end proof theorems

2. **`Chimera/Tests/`** - Smoke tests verifying imports and basic behavior
   - `SmokeTest.lean` - Root import verification tests

## Running Tests

### Build Verification

```bash
cd ChimeraProof
lake build
```

All code must compile without errors before tests can run.

### Smoke Tests

Smoke tests verify that each module imports correctly:

```bash
cd ChimeraProof
./test.sh
```

This runs the smoke test suite which executes `lake build` to verify all modules import correctly.

Note: Full `lake test` integration requires resolving circular dependencies between
`Chimera.IR`, `Chimera.Checkers`, and `Chimera.Link` modules. The build-based test
verification via `./test.sh` is the current working approach.

### Theorem Tests

Theorem tests are part of the main build and are verified by `lake build`:

```bash
cd ChimeraProof
lake build Chimera.Theorems
```

### CI Testing

At the repo root, `./test.sh` currently runs:

```bash
# 1. Proof build
./ChimeraProof/test.sh

# 2. Root placeholder gate
./scripts/check-placeholders.sh

# 3. Rust tools workspace
(cd tools && cargo test --workspace)

# 4. Compiler-core CTest smoke suite
(cd compiler-core/build && cmake --build . && ctest --output-on-failure)

# 5. Runtime conformance
(cd runtime && bash test_conformance.sh)

# 6. Runtime sanitizer conformance
(cd runtime && bash test_sanitizers.sh)
```

If `compiler-core/build` exists, both `./test.sh` and `./scripts/test-all.sh`
first rebuild the configured compiler targets and then fail if CMake has
registered zero tests instead of treating the empty `ctest` run as a pass.

The compiler-core layer currently runs executable smoke coverage for the built
`chimerac` driver, including parse, sidecar-emission, and `--lower-llvm`
checks, plus the standalone `chimera-fuzz-smoke` executable for parser,
metadata, and C API fuzz-entry smoke coverage, and the standalone
`chimera-benchmark-smoke` executable for parse/verify/pass/lowering/emission
timing coverage. It also runs the standalone `chimera-pass-smoke` executable
for the tracked verification compatibility pipeline and pass-preset coverage.
The older lit corpus remains in-tree for future toolchain expansion, but it is
not the root health signal today.

The runtime layer now has two enforced paths:
- ABI/runtime conformance via `runtime/test_conformance.sh`
- sanitizer/header/ASan smoke validation via `runtime/test_sanitizers.sh`

The GitHub Actions workflow mirrors that runtime sanitizer path with a dedicated
`runtime-sanitizers` job that runs `cd runtime && ./test_sanitizers.sh`.

In CI, the following commands are run:

```bash
cd ChimeraProof

# 1. Build all targets
lake build

# 2. Run smoke tests
./test.sh

# 3. Verify no unauthorized placeholders
./scripts/check-placeholders.sh

# 4. Verify documentation exists
./scripts/check-docs.sh

# 5. Verify Zig release contracts
bash tests/version-manifest.sh
bash tests/completion-ledger.sh
bash tests/release-gate-contracts.sh
```

## Test Coverage

### Minimum Coverage Requirements

| Module | Required Tests |
|--------|----------------|
| Foundation | Word arithmetic, Target compatibility, Symbol FQN |
| ABI | Type lowering, Layout computation, Signature compatibility |
| Memory | Pointer operations, Allocator allocation, Ownership transfers |
| Error | Status codes, Error domains, Panic boundaries |
| Effects | Effect lattice, Pure function properties |
| IR | Module well-formedness, Import resolution |
| Link | Symbol resolution, Module composition |
| Checkers | Metadata validation, Layout validation |

## Adding New Tests

When adding features:

1. Add theorem tests in the appropriate `Theorems/` file
2. Add smoke tests in `Tests/SmokeTest.lean` if adding new modules
3. Verify `lake build` passes
4. Document test coverage in this file
