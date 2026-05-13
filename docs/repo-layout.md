# Repository Layout

This document describes the canonical structure of the Chimera monorepo.

## Overview

Chimera is a polyglot IR system for composing C, Rust, and Zig into a single binary via a shared ABI and verified IR. The repository is organized as a monorepo with distinct layers.

## Directory Structure

```
chimera/
├── ChimeraProof/          # Lean/Lake proof system and formal model
├── docs/                   # Design documents, specs, and task lists
├── compiler-core/          # C++/MLIR compiler core (Task 9-34)
├── tools/                  # Rust CLI tooling (Task 35-53)
├── runtime/                # C/Rust/Zig runtime ABI definitions (Task 54-59)
├── examples/               # Multi-language demo projects (Task 60-64)
└── .git/
```

## Production Code Status (as of 2026-05-06)

| Directory | Status | Tasks |
|-----------|--------|-------|
| `compiler-core/` | Bootstrap | Task 9-18 |
| `tools/` | Production | Task 103-172 |
| `runtime/` | Production | Task 124-134 |
| `examples/` | Bootstrap | Task 135-145 |

## ChimeraProof/

The Lean/Lake proof system that provides:
- Formal model of ChimeraIR, ABI types, ownership, memory, errors, effects, layouts
- Verified validators for `.chmeta`, ABI signatures, layouts, ownership rules
- Proof-producing compiler integration
- Trusted-boundary accounting

### Module Organization

```
ChimeraProof/
├── lakefile.toml          # Lake build configuration
├── Chimera/
│   ├── Foundation/        # Low-level primitives (Word, Target, Alignment, Symbol, FinMap, Bytes, Result)
│   ├── ABI/               # Application Binary Interface (Type, PhysicalType, Layout, Lowering, Signature, Contract)
│   ├── Memory/            # Memory model (Pointer, Allocator, Block, Borrow, Permission, Ownership, Drop)
│   ├── Error/             # Error handling (Status, ErrorDomain, ChError, Panic, Bridge)
│   ├── Effects/           # Effect lattice modeling
│   ├── IR/                # Intermediate representation (Module, WellFormed)
│   ├── Link/              # Linking logic (SymbolTable, Resolve, Compose)
│   ├── Wrapper/           # Wrapper AST and code generation
│   ├── Checkers/          # Static analysis checkers
│   ├── Theorems/          # Test suites for each module
│   ├── Generators/        # (reserved for future code generators)
│   └── Tactics/           # (reserved for future proof tactics)
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `Foundation` | Fixed-width words, target triples, alignment, symbol identity, finite maps, byte sequences |
| `ABI` | Semantic and physical ABI types, layout computation, lowering rules, signature compatibility |
| `Memory` | Pointer model, ownership, borrowing, allocators, drop semantics |
| `Error` | Status codes, error domains, panic handling, C/Rust/Zig error bridge |
| `Effects` | Effect lattice, inference, composition rules |
| `IR` | Module model, operations, well-formedness predicates |
| `Link` | Symbol resolution, duplicate detection, module composition |
| `Checkers` | Metadata, layout, signature, contract, ownership, allocator, result, panic, effect, link checkers |
| `Wrapper` | Wrapper AST, C/Rust/Zig code generation |
| `Theorems` | Test suites validating each subsystem |

## docs/

Design documents and task tracking:

| Document | Purpose |
|----------|---------|
| `spec.md` | High-level architectural vision |
| `lean.md` | Lean proof system plan and module design |
| `task-list-7.md` | Current task list (172 tasks in 11 sections) |
| `repo-layout.md` | This document |

## Generated Artifact Exclusions

The following patterns should never be committed to source control:

| Pattern | Reason |
|---------|--------|
| `.zig-cache/` | Zig build cache, regenerated on every build |
| `build/` | Generated build artifacts |
| `target/` | Cargo build output |
| `*.o`, `*.a`, `*.so`, `*.dylib` | Compiled object files |
| `*.olean`, `*.ilean` | Lean build cache |

## Build Commands

```bash
# Build the proof system
cd ChimeraProof && lake build

# Run tests (requires test driver configuration)
cd ChimeraProof && lake test

# Check build targets
cd ChimeraProof && lake check-build

# Clean build artifacts
cd ChimeraProof && lake clean
```

## Dependency Direction

Modules depend on each other in a specific direction to avoid circular imports:

```
Foundation ← ABI ← Memory ← Error ← Effects ← IR ← Link ← Checkers ← Wrapper
                ↑                    ↑
                └────────────────────┘
```

- `Foundation` is the base layer with no dependencies
- Higher layers depend only on lower layers
- No circular dependencies allowed

## tools/

The Rust workspace contains CLI tooling and libraries for the Chimera toolchain.

### Crates

| Crate | Purpose |
|-------|---------|
| `chimera-cli` | Main CLI entrypoint (check, build, link, explain, clean) |
| `chimera-meta` | Metadata schema modeling for `.chmeta` |
| `chimera-object` | `.cho` artifact modeling |
| `chimera-diagnostics` | Diagnostic codes and rendering |
| `chimera-proof-bridge` | Lean proof integration |
| `chimera-build` | Build graph orchestration |
| `chimera-link` | Link planning and invocation |
| `chimera-wrappergen` | C/Rust/Zig wrapper generation |
| `chimera-cache` | Content-addressed artifact cache |
| `chimera-manifest` | `Chimera.toml` manifest parsing |
| `chimera-adapter-c` | C header parsing and layout validation |
| `chimera-adapter-rust` | Rust repr(C)/extern validation |
| `chimera-adapter-zig` | Zig export fn/extern struct validation |

### Build Commands

```bash
# Build all tools
cd tools && cargo build --release

# Run workspace tests
cd tools && cargo test --workspace

# Run tests for specific crate
cargo test -p chimera-wrappergen
```

## runtime/

The runtime directory contains cross-language ABI implementations:

| Directory | Purpose |
|-----------|---------|
| `include/` | C header files (chimera_abi.h, chimera_conformance.h) |
| `rust/` | Rust chimera-rt crate |
| `zig/` | Zig ABI module |

### Build Commands

```bash
# Build Rust runtime
cd runtime/rust && cargo build --release

# Run runtime tests
cargo test
```

## Toolchain Requirements

The one-binary build path requires external tools to be installed. This documents which are mandatory vs optional.

### Mandatory Tools

| Tool | Purpose | Minimum Version |
|------|---------|-----------------|
| C compiler (gcc or clang) | Compile C source files | gcc 9+ or clang 11+ |
| Rust toolchain | Compile Rust source, build rlib archives | Rust 1.75+ |
| lld or platform linker | Link object files into final binary | lld 15+ or platform-native |

### Optional Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| Zig | Compile Zig source files | Only needed for Zig components |
| chimerac (compiler-core driver) | Process ChimeraIR (.mlir/.chir) inputs | Optional for pure C/Rust/Zig builds |
| chimera-proof-bridge | Verify emitted `.chproof` sidecars | Only needed when proof verification is enabled |

### Linker Discovery

The build orchestrator searches for linkers in this order:

1. `CHIMERA_LINKER` environment variable (if set to an executable path or command name resolvable via `PATH`)
2. `build/bin/chimera-link` (in-repo chimera-link)
3. `target/debug/chimera-link`
4. `target/release/chimera-link`
5. `tools/target/debug/chimera-link`
6. `tools/target/release/chimera-link`
7. `/usr/local/bin/chimera-link`, `/usr/bin/chimera-link`
8. `lld` variants and compiler-driver fallbacks such as `cc`, `clang`, and `gcc`

If no linker is found, build fails with an actionable error message suggesting installation.

### Proof Bridge Discovery

When proof verification is enabled, the build orchestrator searches for the proof bridge in this order:

1. `CHIMERA_PROOF_BRIDGE` environment variable (if set to an executable path or command name resolvable via `PATH`)
2. `build/bin/chimera-proof-bridge`
3. `target/debug/chimera-proof-bridge`
4. `target/release/chimera-proof-bridge`
5. `tools/target/debug/chimera-proof-bridge`
6. `tools/target/release/chimera-proof-bridge`
7. `/usr/local/bin/chimera-proof-bridge`, `/usr/bin/chimera-proof-bridge`

The supported executable contract is:

```text
chimera-proof-bridge verify <proof-sidecar>
```

The bridge validates the `.chproof` sidecar structure and exits non-zero on malformed or unsupported input.

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `CHIMERA_LINKER` | Preferred linker executable name or path |
| `CHIMERA_COMPILER_DRIVER` | Path to chimerac (optional) |
| `CHIMERA_PROOF_BRIDGE` | Preferred proof bridge executable name or path (optional) |
