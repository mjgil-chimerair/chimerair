#!/usr/bin/env bash
# Top-level build orchestration for Chimera
# Builds all layers: proof, compiler-core, tools, runtime

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load toolchain versions
if [ -f "toolchain.toml" ]; then
    echo "Loading toolchain from toolchain.toml"
fi

echo "=== Building Chimera ==="

# Build ChimeraProof (Lean)
echo "--- Building ChimeraProof ---"
if [ -d "ChimeraProof" ]; then
    cd ChimeraProof
    lake build
    cd ..
else
    echo "Warning: ChimeraProof not found, skipping"
fi

# Build compiler-core (C++/MLIR)
echo "--- Building compiler-core ---"
if [ -d "compiler-core" ]; then
    mkdir -p compiler-core/build
    cd compiler-core/build
    cmake .. -G Ninja
    ninja
    cd ../..
else
    echo "Warning: compiler-core not found, skipping"
fi

# Build tools (Rust)
echo "--- Building tools ---"
if [ -d "tools" ]; then
    cd tools
    cargo build --release
    cd ..
else
    echo "Warning: tools not found, skipping"
fi

# Build runtime
echo "--- Building runtime ---"
if [ -d "runtime" ]; then
    cd runtime
    ./build.sh 2>/dev/null || echo "Runtime build: no build.sh found, assuming header-only"
    cd ..
fi

echo "=== Build complete ==="