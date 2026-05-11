#!/bin/bash
#!
# @file test.sh
# @brief Integration tests for one-binary demo
#
# Tests all three language components work correctly.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

# Test 1: Demo config exists
test_demo_config() {
    echo "Test: Demo config exists"
    if [ -f "$BUILD_DIR/demo.config" ]; then
        pass "Demo config file exists"
    else
        fail "Demo config file not found"
    fi
}

# Test 2: Final binary exists
test_final_binary_exists() {
    echo "Test: Final binary exists"
    if [ -x "$BUILD_DIR/chimera_binary" ]; then
        pass "Final binary exists and is executable"
    else
        fail "Final binary missing or not executable"
    fi
}

# Test 3: Final binary runs
test_final_binary_runs() {
    echo "Test: Final binary runs"
    if [ ! -x "$BUILD_DIR/chimera_binary" ]; then
        fail "Cannot run binary - it does not exist"
        return
    fi

    local output
    if output=$("$BUILD_DIR/chimera_binary" "$BUILD_DIR/demo.config" 2>&1); then
        if echo "$output" | grep -q "entries=3" && echo "$output" | grep -q "checksum="; then
            pass "Final binary executed successfully"
        else
            fail "Final binary output unexpected"
        fi
    else
        fail "Final binary execution failed"
    fi
}

# Test 4: C reader unit fixture still passes
test_c_reader_fixture() {
    echo "Test: C reader fixture"
    local test_binary="$SCRIPT_DIR/c-reader/chimera_reader_test"
    gcc -Wall -Wextra -I"$SCRIPT_DIR/../../runtime/include" \
        "$SCRIPT_DIR/c-reader/chimera_reader.c" \
        "$SCRIPT_DIR/c-reader/chimera_reader_test.c" \
        -o "$test_binary" 2>/dev/null && \
        "$test_binary" > /dev/null 2>&1 && \
        pass "C reader fixture passed" || \
        fail "C reader fixture failed"
}

# Run all tests
echo "========================================="
echo "Chimera One-Binary Demo Tests"
echo "========================================="
echo ""

test_demo_config
test_final_binary_exists
test_final_binary_runs
test_c_reader_fixture

echo ""
echo "========================================="
echo "Results: $PASSED passed, $FAILED failed"
echo "========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
exit 0
