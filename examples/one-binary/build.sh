#!/bin/bash
#!
# @file build.sh
# @brief One-binary demo build script
#
# Builds all three language components (C, Rust, Zig) and links them
# into a single demo binary using the Chimera toolchain.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
CHIMERA_TOOLS="$PROJECT_ROOT/tools/target/release"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Ensure build directory
mkdir -p "$BUILD_DIR"

log_info "Building Chimera CLI..."
(cd "$PROJECT_ROOT/tools" && cargo build --release -p chimera-cli 2>&1) > /dev/null

CHIMERA="$CHIMERA_TOOLS/chimera"

log_info "Building Chimera one-binary demo via chimera CLI..."
log_info "Build directory: $BUILD_DIR"

# ============================================================
# Build using chimera CLI which handles:
# - Metadata extraction from source files
# - Wrapper generation via chimera-wrappergen
# - Compilation of C, Rust, Zig sources
# - Final linking into one binary
# ============================================================
log_info "Running chimera build..."
if ! "$CHIMERA" build --manifest "$SCRIPT_DIR/Chimera.toml" --output "$BUILD_DIR" --skip-proof 2>&1; then
    log_error "Build failed - chimera CLI returned error"
    exit 1
fi

log_info "Build completed successfully"

cat > "$BUILD_DIR/demo.config" << 'EOF'
# Demo configuration
app_name=ChimeraDemo
version=0.1.0
mode=production
EOF

if [ ! -f "$BUILD_DIR/chimera_binary" ]; then
    log_error "No final binary produced"
    exit 1
fi

log_info "Final binary: $BUILD_DIR/chimera_binary"
chmod +x "$BUILD_DIR/chimera_binary"

if "$BUILD_DIR/chimera_binary" "$BUILD_DIR/demo.config"; then
    log_info "Binary runs successfully with demo config"
else
    log_error "Built binary failed to run"
    exit 1
fi

# ============================================================
# Summary
# ============================================================
echo ""
log_info "Build complete!"
echo ""
echo "Components available:"
[ -f "$BUILD_DIR/chimera_binary" ] && echo "  - Final binary:    $BUILD_DIR/chimera_binary"
[ -f "$BUILD_DIR/demo.config" ] && echo "  - Config file:     $BUILD_DIR/demo.config"
echo ""
