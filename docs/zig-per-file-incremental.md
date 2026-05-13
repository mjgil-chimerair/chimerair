# ZigMera Per-File Incremental Build Plan

## Problem Statement

Current Bun builds with `zig build obj` compile ALL Zig source files into a single monolithic `bun-zig.0.o` through `bun-zig.19.o`. This means even a small change to one source file triggers a full rebuild of all 20 object files (~48s for an incremental change).

**Goal**: Achieve sub-second incremental builds for small source changes by only recompiling changed files.

## Current State

| Build Type | Time | Mechanism |
|------------|------|-----------|
| Clean fresh | ~5 min | Full compilation |
| Small change | ~48s | Zig's internal cache (via `ZIG_LOCAL_CACHE_DIR`) |
| No-op | ~0.4s | Session reuse (shim detects no work) |

The ~48s is Zig's internal incremental, but it still recompiles the entire `bun-zig.*.o` set when any source changes because `zig build obj` produces multiple object files as a single target.

## Solution: Per-File Incremental Build

### Key Insight

Zig supports `zig build-obj <file.zig>` to compile a single .zig file to a .o file with proper dependency tracking. We can:

1. Parse Bun's `build.zig` to extract per-file compilation units
2. Use `zig build-obj` for individual source files
3. Track file dependencies and only rebuild changed files
4. Use content hashing to detect changes

### Architecture

```
zigmera-zig-shim/
├── src/
│   ├── main.rs           # Shim intercepts zig calls
│   ├── per_file.rs        # Per-file build orchestration (NEW)
│   ├── dep_parser.rs      # Parse build.zig for file targets (NEW)
│   └── cache.rs           # File-level cache with content hashing
├── build.zig              # Zig build definition for shim itself
└── Cargo.toml
```

### Implementation Steps

#### Step 1: Parse build.zig to Extract File Targets

Create `dep_parser.rs` that:
- Parses Bun's `build.zig` using a simple regex/AST approach
- Extracts all `.addObject()` calls and their source files
- Maps output `.o` files to input `.zig` files

```rust
pub struct FileTarget {
    pub source_files: Vec<String>,  // e.g., ["src/main.zig", "src/util.zig"]
    pub output_file: String,        // e.g., "bun-zig.0.o"
    pub dependencies: Vec<String>,  // other targets this depends on
}
```

**Difficulty**: Medium - requires understanding build.zig AST

#### Step 2: Implement Per-File Build

Create `per_file.rs` that:
- Tracks content hashes of all source files
- On each build invocation:
  1. Check if any source file changed (compare content hashes)
  2. If changed files affect a target, rebuild only that target with `zig build-obj`
  3. Copy output .o files to the build directory

```rust
pub fn needs_rebuild(&self, target: &FileTarget) -> bool {
    for source in &target.source_files {
        if self.hash(source) != self.cache.get(source) {
            return true;
        }
    }
    false
}
```

**Difficulty**: Medium - content hashing and file tracking

#### Step 3: Integrate with Shim

Modify `main.rs`:
- Intercept `zig build obj` calls
- Instead of forwarding directly, check if per-file mode is enabled
- If enabled, use `per_file.rs` to handle the build
- Otherwise, fall back to current behavior

**Difficulty**: Low - mostly routing logic

#### Step 4: Add Emitter Options for Dependency Tracking

Use zig's `--emit-zigmera-dep` (existing custom flag) or create new:
- When zig outputs a .o file, also output a `.dep` file listing all imports
- Parse these to build accurate dependency graphs
- Enable downstream target invalidation

**Difficulty**: Medium - requires coordination with zig compiler

### Data Flow

```
1. Fresh Build:
   zig build obj --cache-dir X --global-cache-dir Y ...
   -> PerFileBuilder::build_all(targets)
   -> For each target:
      -> zig build-obj source.zig --cache-dir X -of=output.o -fno-incremental
      -> Store content hashes in cache
   -> Total time: ~5 min (same as baseline)

2. Incremental (no change):
   Same command
   -> PerFileBuilder::check_cached(targets)
   -> All hashes match, skip compilation
   -> Copy cached .o files to build dir
   -> Total time: <0.1s

3. Incremental (small change to src/main.zig):
   Same command
   -> PerFileBuilder::check_cached(targets)
   -> Hash mismatch for src/main.zig
   -> Find affected targets (bun-zig.0.o depends on main.zig)
   -> zig build-obj src/main.zig --cache-dir X -of=bun-zig.0.o -fno-incremental
   -> Update hash cache
   -> Total time: ~2-5s (only one file recompiled)
```

### Risk Mitigation

**Risk**: Parsing build.zig is complex
**Mitigation**: Start with a simple manifest file approach where Bun's build system exports the file mapping explicitly

**Risk**: `zig build-obj` might have different compiler flags
**Mitigation**: Extract exact flags from ninja command and pass them through

**Risk**: Cache invalidation might be wrong
**Mitigation**: Use content hashing, verify via `zig build --scanresources`

### Success Criteria

| Build Type | Baseline | Target | Method |
|------------|----------|--------|--------|
| Clean fresh | ~5 min | ~5 min | Full compile |
| Small change | ~48s | <2s | Per-file rebuild |
| No-op | ~0.4s | <0.1s | Copy cached .o files |

### Alternative Approaches Considered

1. **Use zig's `-fincremental` flag**: Causes hangs, likely needs specific cache setup
2. **Parse .d files**: Zig doesn't emit Clang-style dependency files
3. **Use zig's `--watch` mode**: Continuous build, not suitable for single invocations
4. **Modify Bun's build.zig**: Too invasive, requires Bun maintainer changes

### Implementation Order

1. **Phase 1**: Manifest-based approach - have Bun's build export a `file_targets.json` manifest ✓
2. **Phase 2**: Implement per-file builder that reads manifest and uses `zig build-obj` ✓
3. **Phase 3**: Add content hashing and caching ✓
4. **Phase 4**: Wire into shim with automatic detection ✓

### Files Created/Modified

```
tools/crates/zigmera-zig-shim/
├── src/
│   ├── main.rs            # MODIFY: Route to per_file builder ✓
│   ├── per_file.rs         # NEW: Per-file build orchestration ✓
│   ├── manifest.rs         # NEW: Parse file_targets.json + HashCache ✓
│   ├── hash_cache.rs       # (integrated into manifest.rs) ✓
│   └── dep_parser.rs       # NEW: Parse ninja for manifest generation ✓
├── build.zig
└── Cargo.toml
```

### Implementation Status (2026-05-08)

- [x] `manifest.rs`: FileTarget, Manifest, ContentHash, HashCache structs
- [x] `per_file.rs`: PerFileBuilder with `build()`, `build_target()`, `copy_cached_object()`
- [x] `main.rs`: Integration with `run_per_file_build()` and `is_zig_build_obj()`
- [x] `gen.rs`: Manifest generator with `parse_imports()`, `generate_from_sources()`
- [x] Borrow checker issues resolved (clone targets before iteration)
- [x] All 29 unit tests passing
- [x] Ninja manifest parsing (`generate_manifest_from_ninja`, `parse_zig_build_line`)
- [x] Transitive dependency rebuild (recursive graph traversal)
- [x] Integration test for actual `zig build-obj` invocation
- [x] Test fixture project created at `tests/rust-fixtures/zig-incremental-test/`
- [x] `zigmera-cli` crate for general Zig project support
  - `init` command: Initialize project with manifest generation
  - `build` command: Per-file incremental build
  - `status` command: Show cache status
  - `clean` command: Clean build artifacts and cache
  - `gen-manifest` command: Generate manifest without building

### Testing Plan

1. [x] Create a simple test project (`tests/rust-fixtures/zig-incremental-test/`) ✓
2. [x] Verify fresh build produces same output as baseline ✓
3. [x] Modify one file, verify only that file recompiles ✓
4. [x] Verify no-op builds are truly no-op (cache hit) ✓
5. [x] Run zigmera-cli on general Zig project ✓

## Conclusion

This plan achieves the goal by leveraging zig's native `build-obj` capability rather than trying to make the monolithic `build obj` incremental. The key is extracting the per-file mapping and using content hashing to determine what needs rebuilding.

The shim becomes the orchestration layer that:
1. Knows the file mapping (via manifest)
2. Tracks content hashes
3. Only invokes `zig build-obj` for changed files
4. Copies cached .o files for unchanged files