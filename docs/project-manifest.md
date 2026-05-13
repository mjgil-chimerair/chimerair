# Project Manifest

## Overview

The Chimera project manifest (`Chimera.toml`) describes a project's sources, imports, targets, toolchains, runtime mode, and output kind for the Chimera build system.

## Manifest Structure

```toml
version = "0.1.0"
name = "my-project"
description = "A test project"
chimera_version = "0.1.0"

# Source files
[[sources]]
path = "src/lib.rs"
language = "rust"

[[sources]]
path = "src/main.c"
language = "c"

# External imports
[[imports]]
name = "my_func"
path = "libmyfunc.so"
cconv = "c"

# Build targets
[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = []

# Runtime configuration
[runtime]
mode = "std"
output = "executable"

# Toolchain overrides
[toolchains]
rust = "nightly"
c = "gcc"
zig = "0.14.0"
```

## Fields

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | Yes | Manifest version (must be "0.1.0") |
| `name` | string | Yes | Project name (max 256 chars) |
| `description` | string | No | Project description |
| `chimera_version` | string | No | Required Chimera ABI version |

### Source Entries

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | Yes | Path to source file |
| `language` | string | Yes | Source language: "c", "rust", or "zig" |

### Import Entries

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Imported symbol name |
| `path` | string | Yes | Path to library/binary |
| `cconv` | string | No | Calling convention: "c", "sysv", "fastcall", "thiscall" |

### Target Entries

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `triple` | string | Yes | Target triple (e.g., "x86_64-unknown-linux-gnu") |
| `features` | array | No | Target-specific features |

### Runtime Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mode` | string | "nostd" | Runtime mode: "core", "std", "nostd" |
| `output` | string | "staticlib" | Output kind: "staticlib", "sharedlib", "executable" |

### Toolchain Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `c` | string | No | C toolchain identifier |
| `rust` | string | No | Rust toolchain identifier |
| `zig` | string | No | Zig toolchain identifier |

## Runtime Modes

- **core**: Core runtime with minimal types, no standard library
- **std**: Full standard library support
- **nostd**: No standard library (embedded/targets without stdlib)

## Output Kinds

- **staticlib**: Static library (.a/.lib)
- **sharedlib**: Shared library (.so/.dll)
- **executable**: Executable binary

## Example: One-Binary Demo

```toml
version = "0.1.0"
name = "one-binary-demo"
description = "C + Rust + Zig one-binary demo"

[[sources]]
path = "c-reader/chimera_reader.c"
language = "c"

[[sources]]
path = "rust-config/src/lib.rs"
language = "rust"

[[targets]]
triple = "x86_64-unknown-linux-gnu"

[runtime]
mode = "std"
output = "executable"
```

## Versioning

The manifest format follows semantic versioning of the Chimera toolchain. The current supported version is `0.1.0`.

## See Also

- [ChimeraIR Final Design](chimerair-final-design.md) — **Normative** design doc with v0.2 component schema
- [Build Documentation](build.md)
- [Artifact Flow](artifact-flow.md)
- [Runtime README](../runtime/README.md)