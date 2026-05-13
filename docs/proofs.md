# Proofs

This document summarizes the current proof surface for the Zig integration and release-gate package.

## Implemented Baseline

The repository already contains the Lean ZigAdapter scaffold under `ChimeraProof/Chimera/ZigAdapter/` with modules for:

- dependency graph modeling
- invalidation surfaces
- AIR snapshots
- layout fingerprints
- ABI fingerprints
- comptime cache modeling
- error-union and defer lowering surfaces

That scaffold is the implemented baseline for task `111`.

## Release-Gate Proof Boundary

The current release-gate implementation enforces:

- version manifest consistency
- completion-ledger evidence presence
- proof-report JSON and `.chproof` shape validation
- no-placeholder policy
- docs-link checks
- patched-Zig release-gate discovery and smoke validation
- standalone Lean invalidation theorem checks for `Chimera/ZigAdapter/Invalidation.lean`
- Rust contract tests for schema, hashing, artifact/cache path layout, and Zig cache-proof emission
- standalone Lean checks for `Chimera/ZigAdapter/ComptimeCache.lean` and `Chimera/ZigAdapter/ProofInput.lean`
- clean-checkout contract reproduction through `tests/release-gate-clean-checkout.sh`

Run it with:

```bash
bash scripts/release-gate.sh --contracts-only
```

The full gate:

```bash
bash scripts/release-gate.sh
```

adds authoritative patched-Zig validation, the full Rust workspace test suite,
Lean build, compiler-core build/tests, and runtime checks. It intentionally
fails when only the placeholder `third_party/zig` checkout is available.

## Invalidation Soundness Assumptions

Task `114` is implemented at the current proof-surface level with Lean theorems for:

- private body changes that stay private
- public ABI changes invalidating all reachable downstream dependents
- layout changes invalidating dependent metadata and exports
- comptime changes invalidating the full reachable dependency slice
- export-surface changes invalidating downstream link consumers

These theorems assume the emitted `SemanticDependencyGraph` is the authority for
dependency reachability. They prove the invalidation result matches graph reachability
for the named scenarios, but they do not yet prove that patched Zig or compiler-core
extract every required edge from real programs.

## Cache Soundness Assumptions

Task `115` is implemented at the current Lean cache-model level with theorems for:

- matching cache inputs permitting reuse
- schema-version drift rejecting reuse
- target drift rejecting reuse
- build-option drift rejecting reuse
- semantic-fingerprint drift rejecting reuse
- dependency-fingerprint drift rejecting reuse

These theorems assume the current `ComptimeCacheKeyComponents` model is the full
cache identity surface. They prove reuse and rejection from that model, but they
do not yet prove that patched Zig emits every required real-world cache input or
that compiler-core consumes those proof facts end to end.

## Layout Preservation Assumptions

Task `116` is implemented at the current layout-fact model level with theorems for:

- plain struct layout preservation
- packed struct layout preservation
- extern struct layout preservation
- optional-value layout preservation
- slice layout preservation
- error-union layout preservation

These theorems assume patched Zig has already emitted correct `LayoutFact`
records for size, alignment, and field offsets. They prove Chimera lowering
preserves those facts once supplied, but they do not yet prove extraction from
real Zig compiler layout tables end to end.

## Result/Error Lowering Assumptions

Task `117` is implemented at the current result-lowering fact level with theorems for:

- small-payload error-union lowering
- large-payload error-union lowering
- error-only lowering
- status-channel preservation
- error-domain preservation
- payload constraint preservation

These theorems assume patched Zig has already emitted the relevant payload-shape
and error-domain facts. They prove Chimera lowering preserves the success/error
distinction and payload/status constraints once supplied, but they do not yet
prove extraction from real Zig lowering artifacts end to end.

## Ownership/Defer Soundness Assumptions

Task `118` is implemented at the current defer and ownership-boundary fact level
with theorems for:

- merged defer cleanup order preservation
- errdefer remaining on the error path only
- owned boundary crossings being rejected without cleanup metadata
- owned boundary crossings being accepted when drop and allocator metadata are present

These theorems assume patched Zig has already emitted the relevant cleanup
ordering and ownership-boundary metadata. They prove Chimera lowering preserves
that ordering and gating once supplied, but they do not yet prove extraction
from real Zig defer/ownership artifacts end to end.

## Incomplete Theorem Work

The following Zig-specific proof tasks remain incomplete:

- `119`: full release proof gate across all repos and external Zig fixtures, including CI execution against the real patched Zig fork and clean-checkout full-gate evidence

This file is intentionally conservative. It documents the current proof surface and release-gate checks without claiming theorem completion that the workspace does not yet provide.
