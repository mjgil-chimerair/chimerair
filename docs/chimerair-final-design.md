# ChimeraIR Final Design

**Status**: v0.2.0-design | **Authoritative for**: all implementation, tests, and CI gates

## Overview

ChimeraIR vNext is:

> **A component-based, artifact-envelope-driven, ABI-edge-aware polyglot build orchestration system.**

It replaces source-oriented build description with a model where **components provide and consume ABI surfaces**. Components emit artifact envelopes containing objects, metadata, proofs, wrappers, link specs, and public surfaces. ChimeraIR orchestrates the build graph, while language-specific adapters own semantic truth.

This document is **normative**. All other design documents are archival or supplementary.

---

## 1. Core Concepts

### 1.1 Components

A component is a first-class build entity with a language, kind, and ABI surface:

```toml
[[components]]
id = "rust_core"
language = "rust"
kind = "cargo-package"
manifest = "rust/Cargo.toml"
package = "my_core"
crate_types = ["staticlib", "cdylib"]
features = []
panic_policy = "abort"
target = { triple = "x86_64-unknown-linux-gnu" }
```

```toml
[[components]]
id = "zig_cli"
language = "zig"
kind = "zig-exe"
root = "src/cli/main.zig"
target = { triple = "x86_64-unknown-linux-gnu" }
```

Supported component kinds:
- `cargo-package` — Rust Cargo package
- `zig-exe` — Zig executable
- `zig-lib` — Zig library
- `c-source` — C translation unit group
- `prebuilt-native` — Prebuilt static/shared library
- `chimera-module` — Already-produced ChimeraIR module

Component fields:

```rust
struct Component {
    id: ComponentId,
    language: Language,
    kind: ComponentKind,
    roots: Vec<PathBuf>,          // root sources or manifests
    package_manifest: Option<PathBuf>,
    package_name: Option<String>,
    crate_types: Vec<CrateType>,
    features: Vec<String>,
    panic_policy: PanicPolicy,
    target: Option<TargetSpec>,
    profile: Option<ProfileSpec>,
    exported_symbols: Vec<Symbol>,
    imported_symbols: Vec<Symbol>,
    module_map: ModuleMap,
    import_map: ImportMap,
    include_dirs: Vec<PathBuf>,
    defines: Vec<(String, Option<String>)>,
}
```

### 1.2 ABI Edges

An ABI edge describes a dependency between two components with specific symbols, linking mode, and delivery policy:

```toml
[[abi_edges]]
consumer = "zig_cli"
provider = "rust_core"
symbols = ["vcore_create", "vcore_destroy", "vcore_analyze"]
mode = "runtime-dlopen"
wrapper = "auto"
proof = "required"
runtime_arg = "--rust-lib"
```

Modes:

| Mode | Behavior |
|------|----------|
| `direct-link` | Provider object/archive/shared-lib participates in native link |
| `static-link` | Provider static archive participates in native link |
| `dynamic-link` | Provider shared library linked at compile time with rpath/install-name |
| `runtime-dlopen` | Provider cdylib packaged, loaded at runtime via dlopen |
| `generated-wrapper` | Chimera generates a C/Rust/Zig wrapper from provider's `.chmeta` contract |

ABI edge fields:

```rust
struct AbiEdge {
    consumer: ComponentId,
    provider: ComponentId,
    symbols: Vec<Symbol>,
    mode: LinkMode,
    wrapper_policy: WrapperPolicy,
    proof_policy: ProofPolicy,
    runtime_delivery: Option<String>,
}
```

### 1.3 Artifact Envelopes

Every language backend returns a `LanguageBuildResult`:

```rust
struct LanguageBuildResult {
    component_id: ComponentId,
    language: Language,
    status: BuildStatus,
    primary_outputs: ArtifactSet,
    link: NativeLinkSpec,
    metadata: MetadataArtifacts,
    proof: ProofArtifacts,
    wrappers_required: Vec<WrapperRequest>,
    public_surface: PublicSurface,
    invalidation: InvalidationReport,
    diagnostics: Vec<Diagnostic>,
}
```

Where `ArtifactSet`:

```rust
struct ArtifactSet {
    objects: Vec<PathBuf>,
    archives: Vec<PathBuf>,
    shared_libs: Vec<PathBuf>,
    executables: Vec<PathBuf>,
    chimera_ir: Vec<PathBuf>,       // .chimera / .chir
    metadata: Vec<PathBuf>,         // .zsnap, .rdep, .chmeta
    proofs: Vec<PathBuf>,           // .chproof
    snapshots: Vec<PathBuf>,        // .zsnap, .rsnap
    depgraphs: Vec<PathBuf>,        // .zdep, .rdepgraph
}
```

### 1.4 Public Surface

Components expose a public surface used for semantic invalidation:

```rust
struct PublicSurface {
    abi_fingerprint: Fingerprint,
    layout_fingerprint: Fingerprint,
    effect_fingerprint: Fingerprint,
    ownership_fingerprint: Fingerprint,
    panic_policy_fingerprint: Fingerprint,
    exported_symbols: Vec<Symbol>,
    imported_symbols: Vec<Symbol>,
}
```

### 1.5 Invalidation Report

```rust
struct InvalidationReport {
    private_body_changed: bool,
    abi_changed: bool,
    layout_changed: bool,
    effects_changed: bool,
    proof_surface_changed: bool,
    wrappers_stale: bool,
    link_stale: bool,
    runtime_package_stale: bool,
}
```

### 1.6 Native Link Spec

```rust
struct NativeLinkSpec {
    objects: Vec<PathBuf>,
    static_archives: Vec<PathBuf>,
    shared_libraries: Vec<PathBuf>,
    library_search_paths: Vec<PathBuf>,
    link_libraries: Vec<String>,
    linker_args: Vec<String>,
    rpaths: Vec<PathBuf>,
    runtime_files: Vec<PathBuf>,
    system_libraries: Vec<String>,
}
```

---

## 2. Crate Topology

| Crate | Responsibility |
|-------|----------------|
| `chimera-component` | **NEW** — ComponentId, ComponentKind, Language, ComponentSpec, TargetSpec, ProfileSpec, ModuleMap, ImportMap |
| `chimera-artifact` | **NEW** — LanguageBuildResult, ArtifactSet, NativeLinkSpec, PublicSurface, InvalidationReport, RuntimeDelivery |
| `chimera-manifest` | Manifest parsing, validation, schema migration v0.1→v0.2 |
| `chimera-build` | Build graph construction, scheduling, artifact dependencies |
| `chimera-link` | Link planning, NativeLinkSpec merging, link diagnostics |
| `chimera-wrappergen` | Wrapper generation from ABI edges and public surfaces |
| `chimera-cache` | Cache key formulation, semantic invalidation propagation |
| `chimera-proof-bridge` | Proof obligation planning, ChimeraProof integration |
| `chimera-package` | Runtime delivery for dlopen/cdylib |
| `chimera-diagnostics` | Structured diagnostics, explanation records |
| `chimera-cli` | CLI: build, check, graph, manifest normalize, explain, package, doctor |
| `chimera-meta` | Cross-language metadata consolidation |
| `chimera-rust-cargo` | Cargo workspace ingestion, compiler-artifact JSON parsing |
| `chimera-rustc-driver` | Authoritative Rust semantic extraction |
| `chimera-adapter-rust` | Non-authoritative Rust facade/fallback |
| `chimera-adapter-zig` | Non-authoritative Zig facade/fallback |
| `chimera-adapter-c` | Non-authoritative C facade/fallback |
| `chimera-c-clang` | Authoritative C semantic extraction |
| `chimera-rust-schema` | Rust artifact schema |
| `chimera-c-schema` | C artifact schema |
| `zigmera-schema` | Zig artifact schema (.zsnap, .zdep, .zairpack) |
| `zigmera-paths` | Artifact and cache path contracts |
| `zigmera-hash` | BLAKE3/SHA-256 hashing with schema domain tags |
| `zigmera-io` | Atomic file and artifact write helpers |
| `zigmera-target` | Target triple, ABI, compatibility |
| `zigmera-diagnostics` | Zig integration diagnostics |
| `zigmera-zig-shim` | Zig compiler shim for incremental facts |

**Non-goals per crate**:
- `chimera-adapter-*` crates do NOT own independent production invalidation engines
- `chimera-manifest` does NOT execute builds — only parses and validates
- `chimera-build` does NOT generate wrappers or proofs — it schedules nodes that do

---

## 3. Build Graph

### 3.1 Node Types

| NodeKind | Description |
|----------|-------------|
| `ComponentBuild` | Language-specific compile of a component |
| `MetadataEmit` | Extract public surface, fingerprints, dependency graph |
| `WrapperGenerate` | Generate wrappers for ABI edges |
| `ProofVerify` | Verify proof obligations |
| `LinkPlan` | Merge NativeLinkSpec from all components |
| `NativeLink` | Execute native linker |
| `PackageRuntime` | Package runtime files for dlopen edges |

### 3.2 Edge Types

| EdgeKind | Description |
|----------|-------------|
| `BuildEdge` | Component must be built before dependent |
| `MetadataEdge` | Metadata must be extracted before use |
| `WrapperEdge` | Wrapper must be generated before link |
| `ProofEdge` | Proof must pass before link |
| `RuntimeEdge` | Runtime files must be packaged before execution |

### 3.3 DAG Requirements

- Component graph must be a valid DAG (no cycles)
- ABI edges do not create cycles if component kinds are compatible
- Build scheduler must produce deterministic topological order

### 3.4 Implementation

The component graph model is implemented in `chimera-component` as:

```rust
// In chimera-component/src/graph.rs
pub struct ComponentGraph {
    nodes: HashMap<ComponentId, ComponentNode>,
    edges: Vec<GraphEdge>,
}

pub struct ComponentNode {
    pub id: ComponentId,
    pub kind: ComponentKind,
    pub language: Language,
    pub is_target: bool,
    pub dependencies: Vec<ComponentId>,
}

pub enum EdgeKind {
    Build,
    Metadata,
    Wrapper,
    Proof,
    Runtime,
}

pub struct GraphEdge {
    pub from: ComponentId,
    pub to: ComponentId,
    pub kind: EdgeKind,
    pub abi_edge: Option<AbiEdge>,
}
```

Key methods:
- `ComponentGraph::new()` - create empty graph
- `add_node(node)` - add component node
- `add_edge(edge)` - add edge (validates endpoints exist)
- `has_cycle()` - detect cycles via DFS
- `topological_order()` - get deterministic build order
- `validate()` - check for cycles, missing deps, invalid edges

---

## 4. Manifest Schema (v0.2)

```toml
version = "0.2.0"
name = "polyglot-app"
chimera_version = "0.1.0"

[defaults]
target = { triple = "x86_64-unknown-linux-gnu" }
profile = { opt_level = 3 }

[[components]]
id = "rust_core"
language = "rust"
kind = "cargo-package"
manifest = "rust/Cargo.toml"
package = "my_core"
crate_types = ["staticlib", "cdylib"]
features = []
panic_policy = "abort"

[[components]]
id = "zig_cli"
language = "zig"
kind = "zig-exe"
root = "src/cli/main.zig"

[components.zig_modules]
name = "ffi"
path = "src/zig/ffi.zig"

[components.zig_modules]
name = "source"
path = "src/zig/source.zig"

[[abi_edges]]
consumer = "zig_cli"
provider = "rust_core"
symbols = ["vcore_create", "vcore_destroy", "vcore_analyze"]
mode = "runtime-dlopen"
wrapper = "auto"
proof = "required"
runtime_arg = "--rust-lib"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = []

[runtime]
mode = "std"
output = "executable"
```

### 4.1 Compatibility

`[[sources]]` is deprecated but supported as a compatibility alias that lowers into `[[components]]`. New projects must use `[[components]]`.

### 4.2 Validation Error Codes

Manifest semantic validation produces diagnostic codes for common errors:

| Code | Name | Description |
|------|------|-------------|
| `E101` | KIND_LANG_MISMATCH | Component kind/language incompatibility (e.g., `zig-exe` with `rust` language) |
| `E102` | TARGET_INCONSISTENT | Target triple mismatch between native library components |
| `E103` | ABI_COMPONENT_MISSING | ABI edge references unknown consumer/provider component |
| `E104` | RUNTIME_DELIVERY_INVALID | Runtime delivery rule violation (e.g., missing runtime_arg for dlopen) |
| `E105` | POLICY_INCOMPATIBLE | Wrapper/proof policy incompatibility (e.g., proof=disabled with generated-wrapper) |
| `E106` | OUTPUT_KIND_INCOMPATIBLE | Output kind incompatible with component type |
| `E107` | CRATE_TYPE_TARGET_MISMATCH | Crate type and target triple mismatch |
| `E108` | REQUIRED_FIELD_MISSING | Required field is missing |
| `E109` | SYMBOL_MISMATCH | Symbol export/import mismatch |

---

## 5. Link Planning

Final link consumes merged `NativeLinkSpec`:

```text
RustComponent.link
ZigComponent.link
CComponent.link
GeneratedWrappers.link
Runtime.link
        ↓
   LinkPlan
        ↓
   NativeLink
        ↓
   Package
```

This removes the current "CargoBuild skipped from link inputs" problem:
- Rust `staticlib` → static_archives
- Rust `cdylib` (dlopen mode) → runtime_files, not linked
- Rust `rlib` (dependency) → not directly linked, via wrapper

---

## 6. Authoritative Boundaries

| Layer | Authority |
|-------|-----------|
| `chimerair` | Orchestration, downstream invalidation, graph scheduling |
| `zigmera-zig` / `zigmera-lowering` | Zig semantic truth, incremental facts |
| `chimera-rustc-driver` | Rust semantic truth, ABI/layout/ownership/effects |
| `chimera-c-clang` | C semantic truth, layout/ABI |
| `chimera-adapter-*` | **Non-authoritative** — facade or fallback only |

Production builds require authoritative mode for each language.

---

## 7. Semantic Invalidation

Chimera's advantage is distinguishing:

- **Private body edits** — no downstream rebuild
- **ABI signature changes** — wrappers and link invalidated
- **Layout changes** — proof surface affected
- **Effect surface changes** — proof obligations affected
- **Ownership changes** — FFI safety proofs invalidated

Invalidation propagates via `InvalidationReport`:

| Change | Effect |
|--------|--------|
| `abi_changed` | `wrappers_stale`, `link_stale` |
| `layout_changed` | `proof_surface_changed`, `wrappers_stale` |
| `effects_changed` | `proof_surface_changed` |
| `private_body_changed` | No downstream effect |

---

## 8. Migration Path

1. Add `chimera-component` and `chimera-artifact` crates
2. Extend `chimera-manifest` with `[[components]]` and `[[abi_edges]]`
3. Keep `[[sources]]` as compatibility alias
4. Upgrade `CargoBuild` to emit `RustArtifactRef` → `LanguageBuildResult`
5. Upgrade `ZigCompile` to consume `ZigCompileContext`
6. Change `Link` to consume merged `NativeLinkSpec`
7. Add `chimera-package` for runtime delivery
8. Update all adapters to return `LanguageBuildResult`

---

## 9. Supersession Rules

| Doc | Status |
|-----|--------|
| `docs/chimerair-final-design.md` | **Normative** |
| `docs/design-9.md` | Archival — superseded by this doc |
| `docs/architecture.md` | Informative — aligned with this doc |
| `docs/artifact-flow.md` | Informative — aligned with this doc |
| `docs/project-manifest.md` | To be updated with v0.2 schema |
| `docs/task-list-9.md` | **Active task list** for implementation |

---

## 10. Completion Criteria

Every task in `task-list-9.md` must have:
- **Code**: Implementation in the appropriate crate
- **Tests**: Unit/integration tests proving correctness
- **Docs**: Updated documentation
- **CI**: Passing gates in `.github/workflows/`

A machine-readable completion ledger maps requirements to evidence. Tasks without all evidence are not "Complete".