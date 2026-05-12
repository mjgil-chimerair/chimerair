#!/bin/bash
# Bun Incremental Build Benchmark Script
# Clones Bun repo and runs timing comparisons

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZIGMERA_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ZIGMERA_DIR"

echo "=== Bun Incremental Build Benchmark ==="
echo ""

# Check if we have a Bun repo
BUN_REPO="${1:-}"
if [ -z "$BUN_REPO" ]; then
    if [ -d "$HOME/git/others/bun" ]; then
        BUN_REPO="$HOME/git/others/bun"
        echo "Using existing Bun repo: $BUN_REPO"
    elif [ -d "$HOME/bun" ]; then
        BUN_REPO="$HOME/bun"
        echo "Using existing Bun repo: $BUN_REPO"
    else
        echo "Usage: $0 <bun-repo-path>"
        echo "   or place Bun repo at ~/bun or ~/git/others/bun"
        echo ""
        echo "No Bun repo found. To get started:"
        echo "  git clone https://github.com/oven-sh/bun.git ~/bun"
        exit 1
    fi
fi

cd "$BUN_REPO"

echo ""
echo "=== Step 1: Build zigmera-zig-shim ==="
cd "$ZIGMERA_DIR"
cargo build --release -p zigmera-zig-shim
SHIM_PATH="$ZIGMERA_DIR/target/release/zig"

if [ ! -f "$SHIM_PATH" ]; then
    # Try debug build
    cargo build -p zigmera-zig-shim
    SHIM_PATH="$ZIGMERA_DIR/target/debug/zig"
fi

echo "Shim built at: $SHIM_PATH"

echo ""
echo "=== Step 2: Find Bun's pinned Zig ==="
# Bun uses a pinned Zig in vendor/zig or .bun/install/cache/
if [ -f "$BUN_REPO/vendor/zig/zig" ]; then
    REAL_ZIG="$BUN_REPO/vendor/zig/zig"
elif [ -d "$BUN_REPO/.bun/install/cache" ]; then
    # Find zig binary in bun's cache
    REAL_ZIG=$(find "$BUN_REPO/.bun/install/cache" -name "zig" -type f 2>/dev/null | head -1)
fi

if [ -z "$REAL_ZIG" ] || [ ! -f "$REAL_ZIG" ]; then
    echo "Warning: Bun's pinned Zig not found. Options:"
    echo "  1. Bun hasn't downloaded zig yet (normal for fresh clones)"
    echo "  2. Bun uses a different zig location"
    echo ""
    echo "Bun downloads its pinned Zig on first build via 'bun scripts/build.ts'"
    echo "Checking for system zig as fallback..."
    if command -v zig &> /dev/null; then
        REAL_ZIG=$(which zig)
        echo "Using system zig: $REAL_ZIG"
    else
        echo "Error: Could not find Bun's pinned Zig"
        echo "Expected locations:"
        echo "  - $BUN_REPO/vendor/zig/zig (downloaded during first build)"
        echo "  - $BUN_REPO/.bun/install/cache/*/zig"
        echo ""
        echo "To get started:"
        echo "  cd $BUN_REPO && bun scripts/build.ts --help"
        exit 1
    fi
fi

echo "Real Zig: $REAL_ZIG"
ZIG_VERSION=$("$REAL_ZIG" version 2>/dev/null || echo "unknown")
echo "Zig version: $ZIG_VERSION"

echo ""
echo "=== Step 3: Detect Bun repo ==="
cd "$BUN_REPO"
if [ ! -f "build.zig" ]; then
    echo "Error: Not a Bun repo (no build.zig)"
    exit 1
fi

echo "Bun repo detected: $BUN_REPO"

echo ""
echo "=== Step 3b: Run configure (ensures vendor/zig is downloaded) ==="
# Use debug-no-asan profile to avoid ASAN OOM issues with high parallelism
bun scripts/build.ts --profile=debug-no-asan --quiet --configure-only 2>&1 | tail -10

echo ""
echo "=== Step 4: Run baseline (normal) build ==="
echo "--- Baseline build (no shim) ---"
# Use debug-no-asan profile to avoid ASAN OOM, -j4 limits ninja parallelism
BUN_DEBUG_QUIET_LOGS=1 bun scripts/build.ts --profile=debug-no-asan --quiet -j4 2>&1 || true

echo ""
echo "=== Step 5: Run shimmed build (first run - records session) ==="
echo "--- Shimmed build (first run) ---"
export ZIGMERA_REAL_ZIG="$REAL_ZIG"
export ZIGMERA_ENABLED=1
export ZIGMERA_CACHE_DIR="$BUN_REPO/.zigmera/cache"
mkdir -p "$ZIGMERA_CACHE_DIR"

# Put shim first in PATH
export PATH="$SHIM_PATH:$PATH"

# Debug: verify shim is in PATH
echo "DEBUG: which zig = $(which zig 2>&1)"
echo "DEBUG: ZIGMERA_REAL_ZIG=$ZIGMERA_REAL_ZIG"

BUN_DEBUG_QUIET_LOGS=1 bun scripts/build.ts --profile=debug-no-asan --quiet -j4 2>&1 || true

echo ""
echo "=== Step 6: Run shimmed build (second run - should skip compile) ==="
echo "--- Shimmed build (second run - no-op reuse) ---"
BUN_DEBUG_QUIET_LOGS=1 bun scripts/build.ts --profile=debug-no-asan --quiet -j4 2>&1 || true

echo ""
echo "=== Session data captured ==="
if [ -d "$ZIGMERA_CACHE_DIR/sessions" ]; then
    echo "Sessions:"
    ls -la "$ZIGMERA_CACHE_DIR/sessions/" 2>/dev/null || true
fi

echo ""
echo "=== Done ==="
echo "To see session details:"
echo "  cat $ZIGMERA_CACHE_DIR/sessions/session.json | jq ."