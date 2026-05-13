#!/bin/bash
#!
# @file setup-toolchains.sh
# @brief Toolchain validation for Chimera
#
# Validates that installed toolchain versions match the pinned versions.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST="$SCRIPT_DIR/versions.toml"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

ERRORS=0
WARNINGS=0

check() {
    local name="$1"
    local version="$2"
    local min_version="$3"

    # Simple version comparison (would use proper semver in production)
    if [ "$version" = "$min_version" ]; then
        echo -e "${GREEN}[OK]${NC} $name $version"
    else
        # Just check major.minor
        local v_major=$(echo $version | cut -d. -f1)
        local v_minor=$(echo $version | cut -d. -f2)
        local m_major=$(echo $min_version | cut -d. -f1)
        local m_minor=$(echo $min_version | cut -d. -f2)
        if [ "$v_major" = "$m_major" ] && [ "$v_minor" = "$m_minor" ]; then
            echo -e "${YELLOW}[WARN]${NC} $name $version (expected $min_version)"
            ((WARNINGS++)) || true
        else
            echo -e "${RED}[FAIL]${NC} $name $version (expected $min_version)"
            ((ERRORS++)) || true
        fi
    fi
}

# Check Lean
if command -v lean &> /dev/null; then
    version=$(lean --version 2>&1 | head -1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
    check "Lean" "$version" "4.29.1"
else
    echo -e "${RED}[SKIP]${NC} Lean (not installed)"
fi

# Check Rust
if command -v rustc &> /dev/null; then
    version=$(rustc --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
    check "Rust" "$version" "1.85.0"
else
    echo -e "${RED}[SKIP]${NC} Rust (not installed)"
fi

# Check GCC
if command -v gcc &> /dev/null; then
    version=$(gcc --version | head -1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
    check "GCC" "$version" "11.4.0"
else
    echo -e "${RED}[SKIP]${NC} GCC (not installed)"
fi

# Check CMake
if command -v cmake &> /dev/null; then
    version=$(cmake --version | head -1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
    check "CMake" "$version" "3.28.1"
else
    echo -e "${RED}[SKIP]${NC} CMake (not installed)"
fi

# Check Ninja
if command -v ninja &> /dev/null; then
    version=$(ninja --version 2>&1)
    check "Ninja" "$version" "1.11.1"
else
    echo -e "${RED}[SKIP]${NC} Ninja (not installed)"
fi

# Check Zig
if command -v zig &> /dev/null; then
    version=$(zig version 2>&1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' | head -1)
    check "Zig" "$version" "0.13.0"
else
    echo -e "${YELLOW}[SKIP]${NC} Zig (not installed - optional)"
fi

echo ""
echo "Toolchain check complete: $ERRORS errors, $WARNINGS warnings"

if [ $ERRORS -gt 0 ]; then
    exit 1
fi
exit 0