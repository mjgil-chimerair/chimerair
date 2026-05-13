#!/bin/bash
# Basic shim validation test - verifies shim forwards correctly to system Zig

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZIGMERA_DIR="$(dirname "$(dirname "$SCRIPT_DIR")")"

echo "=== Shim Basic Validation ==="
echo "ZIGMERA_DIR: $ZIGMERA_DIR"

# Build the shim
echo ""
echo "Step 1: Building shim..."
cd "$ZIGMERA_DIR/tools" && cargo build -p zigmera-zig-shim --release 2>&1 | tail -3
SHIM_PATH="$ZIGMERA_DIR/tools/target/release/zig"
echo "Shim built: $SHIM_PATH"

echo ""
echo "Step 2: Testing shim with 'zig version'..."
export ZIGMERA_REAL_ZIG="$(which zig)"
export ZIGMERA_ENABLED="1"
export ZIGMERA_CACHE_DIR="/tmp/zigmera-test-cache"
rm -rf "$ZIGMERA_CACHE_DIR"
mkdir -p "$ZIGMERA_CACHE_DIR"

echo "Running: $SHIM_PATH version"
$SHIM_PATH version 2>&1

echo ""
echo "Step 3: Checking session.json was created..."
echo "Cache dir contents:"
ls -la "$ZIGMERA_CACHE_DIR/" 2>&1 || echo "Cache dir missing"
ls -la "$ZIGMERA_CACHE_DIR/sessions/" 2>&1 || echo "Sessions dir missing"

if [ -f "$ZIGMERA_CACHE_DIR/sessions/session.json" ]; then
    echo ""
    echo "Session file created:"
    cat "$ZIGMERA_CACHE_DIR/sessions/session.json" | head -30
    echo ""
    echo "=== VALIDATION PASSED ==="
else
    echo "ERROR: Session file not created at $ZIGMERA_CACHE_DIR/sessions/session.json"
    exit 1
fi