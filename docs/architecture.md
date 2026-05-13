# Architecture

This document summarizes the current Chimera repository architecture, with emphasis on the incremental-build authority boundaries for Zig, Rust, and C.

## Layer Ownership

| Layer | Primary Location | Responsibility |
|-------|------------------|----------------|
| Proof model | `ChimeraProof/` | Lean models, theorem surfaces, release-gate proof checks |
| Compiler core | `compiler-core/` | Chimera dialect, verification, lowering, proof export |
| Tooling | `tools/` | CLI, manifests, caches, adapters, artifact handling |
| Runtime ABI | `runtime/` | C, Rust, and Zig ABI headers/modules |
| Examples | `examples/` | End-to-end fixtures and demo builds |

## Zig Incremental Ownership Boundaries

**Authority Rule**: There must be exactly one canonical Zig incremental engine.

| Repository | Owns | Authority |
|------------|------|-----------|
| `zigmera-zig` | Compiler-emitted `.zsnap`, `.zdep`, `.zairpack`, invalidation reports, schema/version metadata | Authoritative |
| `zigmera-lowering` | Semantic reuse and invalidation policy, graph population, graph diffing, cache decisions | Authoritative |
| `chimerair` | Orchestration, downstream integration, build graph scheduling | Authoritative |
| `chimera-adapter-zig` | Thin facade or fixture/fallback-only surface | **Non-authoritative** for production builds |

The recommended rule is:
- Keep compiler-emitted facts in `zigmera-zig`
- Keep cache, graph, diff, and invalidation in `zigmera-lowering`
- Keep orchestration in `chimerair`
- Reduce `chimera-adapter-zig` to either a thin compatibility wrapper over `zigmera-lowering`, or a fixture/fallback-only surface with no independent invalidation authority

If the in-tree adapter continues to maintain its own graph and invalidation engine independently, behavior drift is unavoidable.

See [zig-incremental-ownership-plan.md](zig-incremental-ownership-plan.md) for the full ownership plan and PR sequence.

## Rust And C Incremental Ownership Boundaries

The same authority rule should apply to Rust and C:

- Rust compiler-derived facts are owned by the Rust extraction path, with semantic-mode `chimera-rustc-driver` as the production authority and stable surface-only mode as a non-authoritative fallback
- C compiler-derived facts are owned by the Clang extraction path, with `chimera-c-clang` as the production authority and parser-only mode as a non-authoritative fallback
- `chimerair` owns orchestration and cross-language downstream invalidation, not language semantic rediscovery

See:

- [Polyglot Incremental Rollout Plan](polyglot-incremental-rollout-plan.md)
- [Rust Incremental Ownership Plan](rust-incremental-ownership-plan.md)
- [C Incremental Ownership Plan](c-incremental-ownership-plan.md)

## Performance Positioning

Chimera should not be positioned as a generic replacement for Cargo, Buck2, or Bazel.

The intended performance advantage is narrower:

- Cargo and `rustc` are likely to remain better at pure Rust clean builds and many small single-language builds
- Buck2 and Bazel are likely to remain better at general build-graph orchestration, hermetic execution, and remote caching at large repository scale
- Chimera can plausibly win on semantic cross-language incremental rebuilds when it knows more than file timestamps, source hashes, or opaque action keys

The architecture goal is therefore:

- do not compete on generic build-system breadth first
- compete on language-semantic reuse for Rust, C, and Zig
- skip wrapper, proof, metadata, object, and link work when ABI-, layout-, and public-surface facts prove reuse is safe
- preserve downstream artifacts across language boundaries when edits are private to one language's implementation

If Chimera only adds another orchestration layer on top of Cargo, Clang, Zig, Buck2, or Bazel, it will usually lose on overhead.

If Chimera becomes the system that can distinguish:

- private implementation edits
- exported signature changes
- layout changes
- effect or proof-surface changes

then it can beat generic builders on the subset of rebuilds where semantic invalidation is the dominant cost driver.

## Zig Integration Crates

The current Rust-side Zig integration is intentionally split into narrow crates so release-gate validation can reason about ownership and evidence per crate.

| Crate | Current Role |
|-------|--------------|
| `zigmera-schema` | Versioned `.zsnap`, `.zdep`, `.zairpack`, `.zchmeta`, and `.zchproof` structures |
| `zigmera-paths` | Workspace-relative artifact and cache path contracts under `.zigmera/` |
| `zigmera-hash` | Canonical BLAKE3/SHA-256 hashing with schema domain tags |
| `zigmera-io` | Atomic file and artifact write helpers |
| `zigmera-target` | Target triple, ABI, and compatibility modeling |
| `zigmera-diagnostics` | Zig integration diagnostics and reporting codes |
| `chimera-adapter-zig` | Current in-tree Zig adapter scaffolding and fixtures |

## Artifact Contract

The repo-wide artifact layout for Zig integration is:

```text
<workspace>/.zigmera/
  artifacts/<target>/<profile>/<kind>/<file>
  cache/<target>/<profile>/<kind>/<semantic-fingerprint>
```

The executable contract lives in:

- `tools/crates/zigmera-paths/src/artifact.rs`
- `tools/crates/zigmera-paths/src/cache.rs`
- `scripts/release-gate.sh`

## Related Docs

- [ChimeraIR Final Design](chimerair-final-design.md) — **Normative** design doc
- [Crate Map](crate-map.md) — crate responsibilities and dependencies
- [Contributor Guide](contributor-guide.md)
- [Artifact Flow](artifact-flow.md)
- [Cache](cache.md)
- [Zig Dialect](zig-dialect.md)
- [Proofs](proofs.md)
- [Polyglot Incremental Rollout Plan](polyglot-incremental-rollout-plan.md)
- [Zig Incremental Ownership Plan](zig-incremental-ownership-plan.md)
- [Rust Incremental Ownership Plan](rust-incremental-ownership-plan.md)
- [C Incremental Ownership Plan](c-incremental-ownership-plan.md)
- [Zig Integration](zig-integration.md)
