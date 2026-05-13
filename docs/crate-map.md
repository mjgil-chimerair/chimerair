# ChimeraIR Crate Map

**Status**: v0.2.0 | **Normative for**: crate organization and dependency rules

This document maps every crate in the ChimeraIR Rust workspace with its responsibility, public API, dependency direction, and non-goals.

---

## Crate Overview

```
chimera-component     # NEW - Component identity types
chimera-artifact      # NEW - Artifact envelope types
chimera-package       # NEW - Runtime packaging for dlopen/cdylib
chimera-cli           # CLI entry point
chimera-meta          # Cross-language metadata
chimera-object        # Object file format
chimera-diagnostics   # Structured diagnostics
chimera-proof-bridge  # Proof obligation planning
chimera-build         # Build graph construction
chimera-link          # Link planning and execution
chimera-wrappergen    # Wrapper generation
chimera-cache         # Cache key and invalidation
chimera-manifest      # Manifest parsing and validation
chimera-adapter-c     # C adapter (non-authoritative)
chimera-adapter-rust  # Rust adapter (non-authoritative)
chimera-adapter-zig   # Zig adapter (non-authoritative)
chimera-c-schema      # C schema types
chimera-c-clang       # Authoritative C extraction
chimera-c-source      # C source handling
chimera-c-build       # C build orchestration
chimera-c-abi         # C ABI types
chimera-c-layout      # C layout types
chimera-c-dialect     # C dialect/types
chimera-c-to-chimera  # C→ChimeraIR lowering
chimera-c-cache       # C cache types
chimera-c-proof       # C proof types
chimera-rust-schema   # Rust schema types
chimera-rust-source   # Rust source handling
chimera-rust-cargo    # Cargo workspace ingestion
chimera-rustc-driver  # Authoritative Rust extraction
chimera-rust-mir-import  # MIR import
chimera-rust-dialect  # Rust dialect/types
chimera-rust-to-chimera # Rust→ChimeraIR lowering
chimera-rust-ownership # Rust ownership types
chimera-rust-abi      # Rust ABI types
chimera-rust-layout   # Rust layout types
chimera-rust-effects  # Rust effects types
chimera-rust-proof    # Rust proof types
chimera-rust-cache    # Rust cache types
zigmera-diagnostics   # Zig diagnostics
zigmera-zig-shim      # Zig compiler shim
zigmera-cli           # Zig-specific CLI
zigmera-schema        # Zig schema (.zsnap, .zdep, .zairpack)
zigmera-paths         # Artifact and cache paths
zigmera-hash          # BLAKE3/SHA-256 hashing
zigmera-io            # File I/O helpers
zigmera-target        # Target triple/ABI
```

---

## Core New Crates

### `chimera-component`

**Responsibility**: Component identity, specification, and build graph types.

**Public API**:
- `ComponentId` — stable unique identifier
- `ComponentKind` — cargo-package, zig-exe, zig-lib, c-source, prebuilt-native, chimera-module
- `Language` — rust, zig, c, unknown
- `ComponentSpec` — complete component definition
- `TargetSpec` — target triple and features
- `ProfileSpec` — optimization level, debug, LTO
- `ToolchainSpec` — toolchain overrides
- `ModuleMap` — named modules
- `ImportMap` — import path mappings
- `AbiEdge` — ABI edge between components
- `LinkMode` — direct-link, static-link, dynamic-link, runtime-dlopen, generated-wrapper
- `WrapperPolicy`, `ProofPolicy`
- `Symbol`, `CrateType`, `PanicPolicy`
- `ComponentGraph` — build graph with nodes and edges
- `ComponentNode` — node in the build graph
- `GraphEdge` — edge in the build graph
- `EdgeKind` — Build, Metadata, Wrapper, Proof, Runtime
- `GraphError` — graph operation errors

**Non-goals**: No build execution, no artifact production, no linking.

**Dependencies**: None (leaf crate).

---

### `chimera-artifact`

**Responsibility**: Artifact envelope types that flow through the build graph.

**Public API**:
- `LanguageBuildResult` — complete build result from language backend
- `ArtifactSet` — objects, archives, shared_libs, executables, chimera_ir, metadata, proofs, snapshots, depgraphs
- `NativeLinkSpec` — link inputs for native linking
- `MetadataArtifacts` — .chmeta, .zsnap, .rsnap, .zdep, .rdep, .zairpack
- `ProofArtifacts` — .chproof, lean_proofs
- `PublicSurface` — abi/layout/effect/ownership/panic_policy fingerprints + symbols
- `InvalidationReport` — private_body/abi/layout/effects/proof_surface/wrappers/link/runtime changes
- `RuntimeDelivery` — runtime files and search paths
- `ArtifactManifest` — persisted artifact manifest with version validation
- `Fingerprint`, `BuildStatus`, `Diagnostic`, `WrapperRequest`, `RuntimeFile`

**Non-goals**: No build scheduling, no linking execution, no manifest parsing.

**Dependencies**: `chimera-component`.

---

### `chimera-package`

**Responsibility**: Runtime packaging for dlopen/cdylib delivery.

**Public API**:
- `Packager` — runtime file packager
- `PackageError` — error types
- `PlatformSettings` — platform-specific lib ext, prefix, rpath

**Non-goals**: No build graph management, no linking.

**Dependencies**: `chimera-artifact`.

---

## Existing Crates

### `chimera-cli`

**Responsibility**: CLI entry point, command parsing, user-facing output.

**Public API**:
- `build`, `check`, `graph`, `manifest normalize`, `explain`, `package`, `doctor` commands

**Non-goals**: No direct build logic (delegates to chimera-build).

**Dependencies**: `chimera-build`, `chimera-manifest`, `chimera-diagnostics`.

---

### `chimera-build`

**Responsibility**: Build graph construction and scheduling.

**Public API**:
- `BuildGraph` — component graph with nodes and edges
- `BuildPlan` — scheduled build steps
- `BuildConfig` — build configuration

**Non-goals**: No linking execution (delegates to chimera-link), no wrapper generation (delegates to chimera-wrappergen).

**Dependencies**: `chimera-component`, `chimera-artifact`, `chimera-manifest`.

---

### `chimera-manifest`

**Responsibility**: Manifest parsing and validation.

**Public API**:
- `Manifest` — parsed Chimera.toml
- `ManifestParser` — TOML parser
- `ValidationError` — validation error types

**Non-goals**: No build execution, no artifact handling.

**Dependencies**: `chimera-component` (for types).

---

### `chimera-link`

**Responsibility**: Link planning and execution.

**Public API**:
- `LinkPlan` — planned link operation
- `LinkerConfig` — linker configuration

**Non-goals**: No build scheduling (chimera-build handles), no wrapper generation.

**Dependencies**: `chimera-artifact`.

---

### `chimera-wrappergen`

**Responsibility**: Wrapper generation from ABI edges and public surfaces.

**Public API**:
- `WrapperGenerator` — generates C/Rust/Zig wrappers
- `WrapperTarget` — target language

**Non-goals**: No build scheduling, no linking.

**Dependencies**: `chimera-component`, `chimera-artifact`.

---

### `chimera-cache`

**Responsibility**: Cache key formulation and semantic invalidation propagation.

**Public API**:
- `CacheKey` — composite cache key
- `CacheEntry` — cached build result

**Non-goals**: No build execution, no artifact production.

**Dependencies**: `chimera-artifact`.

---

### `chimera-proof-bridge`

**Responsibility**: Proof obligation planning and ChimeraProof integration.

**Public API**:
- `ProofObligation` — proof requirement
- `ProofPlan` — planned proof verification

**Non-goals**: No build scheduling.

**Dependencies**: `chimera-artifact`.

---

### `chimera-diagnostics`

**Responsibility**: Structured diagnostics and explanation records.

**Public API**:
- `Diagnostic` — diagnostic message
- `DiagnosticCode` — error/warning codes
- `Explanation` — build explanation

**Non-goals**: No build execution.

**Dependencies**: None.

---

## Language Adapter Crates

### `chimera-adapter-c` (non-authoritative)

**Responsibility**: C adapter facade/fallback only.

**Non-goals**: NOT production C invalidation engine. Production use requires `chimera-c-clang`.

**Dependencies**: `chimera-c-schema`, `chimera-c-source`.

---

### `chimera-c-clang` (authoritative)

**Responsibility**: Authoritative C semantic extraction via Clang.

**Non-goals**: No independent invalidation authority in chimerair.

**Dependencies**: `chimera-c-source`, `chimera-c-abi`, `chimera-c-layout`.

---

### `chimera-adapter-rust` (non-authoritative)

**Responsibility**: Rust adapter facade/fallback only.

**Non-goals**: NOT production Rust invalidation engine. Production use requires `chimera-rustc-driver`.

**Dependencies**: `chimera-rust-schema`, `chimera-rust-source`.

---

### `chimera-rustc-driver` (authoritative)

**Responsibility**: Authoritative Rust semantic extraction via rustc driver.

**Non-goals**: No independent invalidation authority in chimerair.

**Dependencies**: `chimera-rust-source`, `chimera-rust-abi`, `chimera-rust-layout`, `chimera-rust-ownership`, `chimera-rust-effects`, `chimera-rust-proof`.

---

### `chimera-adapter-zig` (non-authoritative)

**Responsibility**: Zig adapter facade/fallback only.

**Non-goals**: NOT production Zig invalidation engine. Production use requires `zigmera-zig`/`zigmera-lowering`.

**Dependencies**: `zigmera-schema`.

---

## Zig Integration Crates

### `zigmera-schema`

**Responsibility**: Zig artifact schema (.zsnap, .zdep, .zairpack, .zchmeta, .zchproof).

**Dependencies**: None.

---

### `zigmera-paths`

**Responsibility**: Workspace-relative artifact and cache path contracts.

**Dependencies**: None.

---

### `zigmera-hash`

**Responsibility**: Canonical BLAKE3/SHA-256 hashing with schema domain tags.

**Dependencies**: None.

---

### `zigmera-zig-shim`

**Responsibility**: Zig compiler shim for incremental facts.

**Dependencies**: `zigmera-schema`, `zigmera-hash`.

---

## Dependency Direction Rules

```
chimera-component  <-- chimera-artifact, chimera-manifest, chimera-build, chimera-wrappergen
chimera-artifact   <-- chimera-build, chimera-link, chimera-package, chimera-cache, chimera-proof-bridge
chimera-package    <-- chimera-build
chimera-cli        <-- chimera-build
chimera-build      <-- chimera-cli
chimera-manifest   <-- chimera-cli, chimera-build
chimera-link       <-- chimera-build
chimera-wrappergen <-- chimera-build
chimera-cache      <-- chimera-build
chimera-proof-bridge <-- chimera-build
```

**No cycles allowed**. The graph must be a valid DAG.

---

## Schema Version

All artifact-carrying crates must include schema version in serialized forms:
- `chimera-component`: v0.1.0
- `chimera-artifact`: v0.1.0
- `zigmera-schema`: per-artifact-type version

Version mismatch causes build failure.

---

## Non-Goals per Crate

| Crate | Non-goals |
|-------|-----------|
| `chimera-component` | No build execution, no artifact production |
| `chimera-artifact` | No build scheduling, no linking execution |
| `chimera-package` | No build graph management, no linking |
| `chimera-manifest` | No build execution, no artifact handling |
| `chimera-adapter-*` | No independent production invalidation engine |
| `chimera-build` | No wrapper generation, no proof execution |
| `chimera-link` | No build scheduling |
| `chimera-wrappergen` | No build scheduling, no linking |
| `chimera-cache` | No build execution, no artifact production |
| `chimera-proof-bridge` | No build scheduling |

---

## See Also

- [ChimeraIR Final Design](chimerair-final-design.md)
- [Artifact Flow](artifact-flow.md)
- [Project Manifest](project-manifest.md)