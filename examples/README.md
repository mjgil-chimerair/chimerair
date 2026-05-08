# Chimera One-Binary Demo

Multi-language demo project showing C + Rust + Zig interoperability.

## Status

**Tasks 60-63 Complete**: All component implementations done.

## Structure

```
one-binary/
├── c-reader/          # C file reader component
│   ├── chimera_reader.h
│   ├── chimera_reader.c
│   └── chimera_reader_test.c
├── rust-config/       # Rust config parser
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── main.rs
├── zig-checksum/      # Zig checksum module
│   └── chimera_checksum.zig
├── generated/         # Generated wrappers (future)
└── tests/           # Integration tests (future)
```

## Components

### C File Reader (`c-reader/`)

Bounded file reader using Chimera ABI types:

```c
chimera_status_t chimera_read_file(
    const char* path,
    chimera_reader_config_t* config,
    chimera_reader_result_t* result
);
```

### Rust Config Parser (`rust-config/`)

Config parser with key=value format:

```
# example.config
host=localhost
port=8080
```

Usage:
```bash
cargo run --release -- <config-file>
```

### Zig Checksum (`zig-checksum/`)

Checksum calculation with multiple algorithms:

- CRC32
- Fletcher-16
- Fletcher-32

## Building

### All Components

The project uses the Chimera toolchain to build all components into one binary.

### Individual Components

```bash
# C reader
gcc -I../runtime/include -c c-reader/chimera_reader.c

# Rust config parser
cd rust-config && cargo build --release

# Zig checksum
zig build
```