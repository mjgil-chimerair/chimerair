#!/bin/bash
#!
# @file test_conformance.sh
# @brief Run runtime conformance tests
#
# Tests C, Rust, and Zig runtime conformance.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

PASSED=0
FAILED=0

pass() {
    echo -e "${GREEN}[PASS]${NC} $*"
    ((PASSED++)) || true
}

fail() {
    echo -e "${RED}[FAIL]${NC} $*"
    ((FAILED++)) || true
}

# Compile conformance test
compile_conformance() {
    echo "Compiling conformance suite..."
    mkdir -p "$BUILD_DIR"
    gcc -Wall -Wextra -O2 \
        -I"$SCRIPT_DIR/include" \
        "$SCRIPT_DIR/src/chimera_conformance.c" \
        -o "$BUILD_DIR/chimera_conformance" \
        2>&1 || return 1
}

# Run conformance tests
run_conformance() {
    echo "Running conformance suite..."
    if "$BUILD_DIR/chimera_conformance"; then
        pass "Conformance suite passed"
    else
        fail "Conformance suite failed"
    fi
}

echo "========================================="
echo "Chimera Runtime Conformance Tests"
echo "========================================="

if compile_conformance; then
    pass "Conformance suite compiled"
else
    fail "Conformance suite compilation failed"
fi

run_conformance

echo ""
echo "========================================="
echo "Results: $PASSED passed, $FAILED failed"
echo "========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
exit 0