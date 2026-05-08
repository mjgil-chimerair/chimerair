# One-Binary Example

This fixture is the current end-to-end demo surface for Chimera’s mixed-language build path.

## Build Modes

This example demonstrates the **Cargo/C ABI baseline path** - the original mixed-language build approach where each language compiles to a native artifact (C to object files, Rust to library, Zig to object files) and then links them together.

**Important**: This example uses the legacy `[[sources]]` format and the Cargo/C ABI build path. It serves as the **correctness baseline** to ensure unified lowering doesn’t silently replace the working path.

## Components

- `c-reader/`: C-side reader and ABI fixture
- `rust-config/`: Rust configuration parser
- `zig-checksum/`: Zig checksum helper
- `Chimera.toml`: example project manifest (sources format, not components)
- `build.sh` and `test.sh`: local build and validation entrypoints

## Relationship to Unified Lowering

| Build Mode | Description | This Example |
|-------------|-------------|---------------|
| `CargoCAbi` | Legacy multi-language path via sources + native linking | This example |
| `ArchiveBridge` | Intermediate path via separate archives | Not this example |
| `UnifiedLowering` | New path via ChimeraIR merge + LLVM emission | Uses `[[components]]` |

This example verifies that even when `UnifiedLowering` is the default, the Cargo/C ABI path remains functional and produces correct results.

## Validation

Run the local example checks with:

```bash
bash examples/one-binary/test.sh
```

Or build specifically with Cargo/C ABI mode:

```bash
# Force Cargo/C ABI mode to verify baseline still works
chimerac build --manifest examples/one-binary/Chimera.toml --build-mode cargo-c-abi
```
