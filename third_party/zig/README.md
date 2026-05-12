# Patched Zig Submodule

This directory is a placeholder for the patched Zig compiler fork.

## Fork Location

The actual fork is at: https://github.com/mjgil/zigmera-zig
Branch: `zigmera/snapshot-v1`

## Setup

To initialize the patched Zig submodule:

```bash
git submodule add https://github.com/mjgil/zigmera-zig.git third_party/zig
cd third_party/zig
git checkout zigmera/snapshot-v1
```

## Building

```bash
cd third_party/zig
mkdir build && cd build
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release
ninja
```

## Integration Testing

After building, run the integration smoke test:

```bash
../../scripts/test-zigmera-integration.sh
```

## CI Usage

The submodule is only required for:
- Integration test mode (`--integration-tests` flag)
- Full release gate testing

Normal development (`cargo build`, `ninja`) does NOT require the submodule.

## Documentation

See `docs/zig-compiler-fork.md` for fork architecture details.
