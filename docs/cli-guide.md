# CLI Guide

This document describes the currently implemented `chimera` explanation output.

## `chimera explain`

`chimera explain <file>` accepts either:

- Chimera diagnostic JSON
- Zig cache explanation JSON emitted from the comptime cache contract

## Cache Diagnostics

The current cache explanation JSON surface reports:

- `status`: `hit`, `miss`, or `rebuild`
- `reason`: cache hit, no matching entry, invalidated entry, changed dependency, or changed embed file
- `artifact_kind`: currently `comptime`
- `cache_key`: the exact cache key string used for lookup
- `key_components`: file, symbol name, line, column, argument hash, target, and builtins hash
- `reuse_checks`: cached-entry validity, dependency-graph hash, build-options hash, dependency fingerprints, and embed files

Example:

```json
{
  "artifact_kind": "comptime",
  "cache_key": "comptime_deadbeef",
  "status": "rebuild",
  "reason": {
    "kind": "dependency_changed",
    "dependency_kind": "Type",
    "dependency_id": "Point"
  },
  "key_components": {
    "file": "math.zig",
    "name": "compute_size",
    "line": 20,
    "column": 5,
    "args_hash": "type=Point",
    "target": "x86_64-linux-gnu",
    "builtins_hash": "builtin-hash-1"
  },
  "reuse_checks": {
    "cached_entry_valid": true,
    "dep_graph_hash": "graph-v2",
    "build_options_hash": "build-hash",
    "dependency_fingerprints": [
      { "kind": "Type", "id": "Point", "content_hash": "hash456" }
    ],
    "embed_files": ["assets/point.bin"]
  }
}
```

Use `chimera explain <file> --level verbose` to print all key components and reuse-check details.

## `chimera snapshot`

`chimera snapshot` reads and validates `.zsnap` binary snapshot files emitted by the patched Zig compiler.

### `chimera snapshot read <file>`

Reads a `.zsnap` file and displays its metadata:

```
chimera snapshot read build/.zigmera/x86_64-linux/Release/snapshot.zsnap
```

Output includes:
- Magic bytes and schema version
- Target triple, backend, and optimize mode
- Zig commit hash and timestamp
- Section counts (source files, decls, types, layouts, AIR bodies, exports)

Add `--verbose` for full details including build options, source file list, and export symbols.

### `chimera snapshot validate <file>`

Validates the integrity of a `.zsnap` file:

```
chimera snapshot validate build/.zigmera/x86_64-linux/Release/snapshot.zsnap
```

Exit code 0 means the file is valid. Exit code 1 means validation failed.

Use `--json` for machine-readable output:

```json
{
  "valid": true,
  "version": 1,
  "target": "x86_64-unknown-linux-gnu",
  "source_file_count": 5,
  "errors": []
}
```

### Binary Format

The `.zsnap` binary format has:
- 8 bytes magic (`ZSNAP001`)
- 4 bytes schema version (little-endian u32)
- 20 bytes Zig commit hash
- Length-prefixed strings for target, backend, optimize_mode
- 8 bytes timestamp_ns
- 4 bytes source_file_count
- 32 bytes BLAKE3 checksum
- JSON payload for sections

The adapter validates magic bytes, schema version compatibility, and data integrity before parsing.
