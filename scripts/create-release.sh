#!/bin/bash
#!
# @file create-release.sh
# @brief Create a Chimera release package
#
# Produces the release package layout documented in docs/release-package-layout.md

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="${1:-0.1.0}"
OUTPUT_DIR="$SCRIPT_DIR/release"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }

# Clean output directory
log_info "Creating release package v$VERSION..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/chimera-$VERSION"

PKG_DIR="$OUTPUT_DIR/chimera-$VERSION"

# ============================================================
# bin/
# ============================================================
log_info "Packaging binaries..."
mkdir -p "$PKG_DIR/bin"

# Build chimerac if not exists
if [ -f "$PROJECT_ROOT/target/release/chimera" ]; then
    cp "$PROJECT_ROOT/target/release/chimera" "$PKG_DIR/bin/chimerac"
    log_info "  chimerac: OK"
else
    log_info "  chimerac: not built (skip)"
fi

# Build weaver if exists
if [ -f "$PROJECT_ROOT/target/release/weaver" ]; then
    cp "$PROJECT_ROOT/target/release/weaver" "$PKG_DIR/bin/weaver"
    log_info "  weaver: OK"
else
    log_info "  weaver: not built (skip)"
fi

# ============================================================
# lib/cmake/Chimera/
# ============================================================
log_info "Packaging CMake config..."
mkdir -p "$PKG_DIR/lib/cmake/Chimera"

cat > "$PKG_DIR/lib/cmake/Chimera/ChimeraConfig.cmake" << 'EOF'
@PACKAGE_INIT@
include("${CMAKE_CURRENT_LIST_DIR}/ChimeraTargets.cmake")
check_required_components(Chimera)
EOF

cat > "$PKG_DIR/lib/cmake/Chimera/ChimeraTargets.cmake" << 'EOF'
if(NOT TARGET Chimera::Chimera)
  add_library(Chimera::Chimera INTERFACE IMPORTED)
  set_target_properties(Chimera::Chimera PROPERTIES
    INTERFACE_INCLUDE_DIRECTORIES "${PACKAGE_PREFIX_DIR}/include"
  )
endif()
EOF

cat > "$PKG_DIR/lib/cmake/Chimera/ChimeraVersion.cmake" << 'EOF'
set(PACKAGE_VERSION "@VERSION@")
EOF

sed -i "s/@VERSION@/$VERSION/g" "$PKG_DIR/lib/cmake/Chimera/ChimeraVersion.cmake"

log_info "  CMake config: OK"

# ============================================================
# include/chimera/
# ============================================================
log_info "Packaging public headers..."
mkdir -p "$PKG_DIR/include/chimera"

if [ -f "$PROJECT_ROOT/runtime/include/chimera_abi.h" ]; then
    cp "$PROJECT_ROOT/runtime/include/chimera_abi.h" "$PKG_DIR/include/chimera/"
    cp "$PROJECT_ROOT/runtime/include/chimera_conformance.h" "$PKG_DIR/include/chimera/" 2>/dev/null || true
    log_info "  chimera_abi.h: OK"
else
    log_info "  chimera_abi.h: not found (skip)"
fi

# ============================================================
# runtime/
# ============================================================
log_info "Packaging runtime..."
mkdir -p "$PKG_DIR/runtime/include" "$PKG_DIR/runtime/rust/src" "$PKG_DIR/runtime/zig"

# C runtime
if [ -d "$PROJECT_ROOT/runtime/include" ]; then
    cp -r "$PROJECT_ROOT/runtime/include/"* "$PKG_DIR/runtime/include/" 2>/dev/null || true
    log_info "  C runtime headers: OK"
fi

# Rust runtime
if [ -f "$PROJECT_ROOT/runtime/rust/Cargo.toml" ]; then
    cp "$PROJECT_ROOT/runtime/rust/Cargo.toml" "$PKG_DIR/runtime/rust/"
    if [ -d "$PROJECT_ROOT/runtime/rust/src" ]; then
        cp -r "$PROJECT_ROOT/runtime/rust/src" "$PKG_DIR/runtime/rust/"
    fi
    log_info "  Rust runtime: OK"
fi

# Zig runtime
if [ -f "$PROJECT_ROOT/runtime/zig/chimera_abi.zig" ]; then
    cp "$PROJECT_ROOT/runtime/zig/chimera_abi.zig" "$PKG_DIR/runtime/zig/"
    log_info "  Zig runtime: OK"
fi

# ============================================================
# examples/
# ============================================================
log_info "Packaging examples..."
mkdir -p "$PKG_DIR/examples"

if [ -d "$PROJECT_ROOT/examples/one-binary" ]; then
    cp -r "$PROJECT_ROOT/examples/one-binary" "$PKG_DIR/examples/"
    log_info "  one-binary: OK"
fi

# ============================================================
# share/doc/
# ============================================================
log_info "Packaging documentation..."
mkdir -p "$PKG_DIR/share/doc/chimera"

DOC_FILES=(
    "docs/build.md"
    "docs/testing.md"
    "docs/ci.md"
    "docs/repo-layout.md"
    "docs/abi.md"
    "docs/versioning.md"
    "docs/artifact-flow.md"
)

for doc in "${DOC_FILES[@]}"; do
    if [ -f "$PROJECT_ROOT/$doc" ]; then
        dest="$PKG_DIR/share/doc/chimera/$(basename $doc)"
        cp "$PROJECT_ROOT/$doc" "$dest"
        log_info "  $doc: OK"
    fi
done

# ============================================================
# Create archive
# ============================================================
log_info "Creating archive..."
cd "$OUTPUT_DIR"
tar -czf "chimera-$VERSION-linux-x86_64.tar.gz" "chimera-$VERSION"
zip -rq "chimera-$VERSION-linux-x86_64.zip" "chimera-$VERSION"
log_info "  chimera-$VERSION-linux-x86_64.tar.gz: OK"
log_info "  chimera-$VERSION-linux-x86_64.zip: OK"

# ============================================================
# Summary
# ============================================================
echo ""
log_info "Release package created successfully!"
echo ""
echo "Package contents:"
find "chimera-$VERSION" -type f | head -30
echo ""
echo "Archives:"
ls -lh *.tar.gz *.zip 2>/dev/null || true