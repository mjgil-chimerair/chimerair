# Zig Integration

This document describes how the chimera repo integrates with the patched Zig compiler.

## Ownership Boundaries

**WARNING**: The in-tree `chimera-adapter-zig` is NON-AUTHORITATIVE for production builds.

| Component | Authority | Repository |
|-----------|-----------|------------|
| Compiler-emitted artifacts (`.zsnap`, `.zdep`, `.zairpack`) | Authoritative | `zigmera-zig` |
| Semantic reuse and invalidation | Authoritative | `zigmera-lowering` |
| Build orchestration | Authoritative | `chimerair` |
| In-tree Zig adapter | **Non-authoritative** | `chimera-adapter-zig` |

See [zig-incremental-ownership-plan.md](zig-incremental-ownership-plan.md) for the full ownership plan.

## Fork Location

**Private repository**: https://github.com/mjgil/zigmera-zig
**Branch**: `zigmera/snapshot-v1`

## Submodule Setup

The `third_party/zig` directory is a placeholder for the patched Zig compiler:

```bash
# Clone the fork
git clone git@github.com:mjgil/zigmera-zig.git third_party/zig

# Checkout the integration branch
cd third_party/zig
git checkout zigmera/snapshot-v1
```

Or use as a git submodule:

```bash
git submodule add git@github.com:mjgil/zigmera-zig.git third_party/zig
cd third_party/zig
git checkout zigmera/snapshot-v1
```

## Building the Patched Zig

```bash
cd third_party/zig
mkdir build && cd build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release \
    -DLLVM_DIR=/path/to/llvm/lib/cmake/llvm
ninja
```

## CI Integration

The submodule is only required for:
- **Integration test mode**: `--integration-tests` flag
- **Full release gate testing**

In this repo's GitHub Actions workflow, the authoritative-path contract is
exercised by a local fixture checkout created with
`scripts/setup-authoritative-zig-fixture.sh`. That proves the gate wiring,
discovery order, CLI-flag checks, and fork-local `scripts/test-zigmera.sh`
invocation path. It does not replace running the same gate against the real
external `zigmera-zig` checkout before calling the integration complete.

For the real external path, CI can now clone a configured authoritative checkout
with `scripts/prepare-zig-authoritative-checkout.sh`. Set:

- `CHIMERA_ZIG_GIT_URL`
- `CHIMERA_ZIG_GIT_REF` (optional)
- `CHIMERA_ZIG_GIT_TOKEN` for private HTTPS access

The workflow now validates that configuration explicitly with
`scripts/check-zig-authoritative-ci-config.sh` before it attempts the clone, so
partial or malformed authoritative CI setup fails with a direct config error.

The `zig-release-authoritative` job uses that path and then runs the same
`scripts/run-zig-release-integration.sh require-authoritative` gate as local
release checks. On success it writes and uploads
`zig-authoritative-ci-evidence.json`, which is validated by
`scripts/validate-zig-authoritative-ci-evidence.py`.

The authoritative release gate now requires more than a binary. A valid patched
Zig checkout must provide:

- a real source checkout shape (`.git` plus `CMakeLists.txt`, `build.zig`, or `src/Compilation.zig`)
- a runnable integration script at `scripts/test-zigmera.sh`
- a patched Zig binary whose `--help` output exposes the Chimera emission flags

Normal development (`cargo build`, `ninja`) does NOT require the submodule.

### CI Job for Submodule Hash Check

```yaml
zig-submodule-hash:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        submodules: recursive
    - name: Verify submodule commit
      run: |
        cd third_party/zig
        echo "Checking zigmera-zig commit..."
        git log -1 --format="%H"
```

## Artifact Flow

```
zigmera-zig (compiler) 
    │
    ├─> .zsnap ──> zigmera-lowering/zair-import
    ├─> .zdep ───> zigmera-lowering/zdep
    ├─> .zairpack ─> zigmera-lowering/zair-import
    │
    └─> chimera-adapter-zig (Rust)
            │
            └─> .zchmeta, .chir, .chproof
                    │
                    └─> chimera/compiler-core
```

## Snapshot Emission Flags

The patched Zig supports these CLI flags:

```bash
--emit-zigmera-snapshot        # Emit .zsnap artifact
--emit-zdep                    # Emit dependency graph
--emit-zairpack               # Emit AIR bundle  
--emit-zigmera-invalidation-report  # Emit rebuild reasons
```

## Upstream Sync

To sync with upstream Zig:

```bash
cd third_party/zig
git fetch upstream
git rebase upstream/master
# Resolve conflicts in:
#   src/AstGen.cpp, src/Sema.cpp, src/AIR.cpp
#   src/InternPool.cpp, src/Linker.cpp
ninja && ../../scripts/test-zigmera-integration.sh
```

## Release-Gate Integration Script

The authoritative gate calls the fork-local integration runner:

```bash
cd third_party/zig
scripts/test-zigmera.sh
```

`CHIMERA_ZIG_BIN` is exported by the main repo gate so the script can reuse the
already-built patched Zig binary instead of rediscovering it.

## Fallback Mode

When the patched Zig is unavailable, the adapter operates in:
- `fixture-mode` - Use pre-recorded `.zsnap` files
- `cache-scrape-mode` - Extract from Zig build cache

Outputs are marked `non-authoritative` in fallback mode.

## See Also

- [docs/zig-compiler-fork.md](./zig-compiler-fork.md) - Fork architecture
- [docs/artifact-flow.md](./artifact-flow.md) - Full artifact flow
- [docs/diagnostics.md](./diagnostics.md) - Invalidation diagnostics
- [docs/implementation-limits.md](./implementation-limits.md) - Fallback mode limitations and feature support matrix
