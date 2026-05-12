#!/bin/bash
#!
# @file test-all.sh
# @brief Top-level test orchestration for Chimera
#
# Provides one root test command that runs Lean tests, C++ tests,
# Rust tests, and runtime fixture checks.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_TYPE="${BUILD_TYPE:-release}"
CARGO_PROFILE_FLAG=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0
SKIPPED=0

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $*"; }
pass() { echo -e "${GREEN}[PASS]${NC} $*"; ((PASSED++)) || true; }
fail() { echo -e "${RED}[FAIL]${NC} $*"; ((FAILED++)) || true; }
skip() { echo -e "${YELLOW}[SKIP]${NC} $*"; ((SKIPPED++)) || true; }

compiler_test_count() {
    local build_dir="$1"
    local count

    count="$(cd "$build_dir" && ctest -N | sed -n 's/^Total Tests: //p' | tail -n 1)"
    if [ -z "$count" ]; then
        echo "0"
        return
    fi
    echo "$count"
}

show_help() {
    cat << EOF
Chimera Test System

Usage: ./scripts/test-all.sh [OPTIONS]

Options:
    --build-type TYPE     Build type: release (default) or debug
    --skip-proof          Skip proof system tests
    --skip-runtime        Skip runtime tests
    --skip-tools          Skip tools tests
    --skip-compiler       Skip compiler-core tests
    --test TYPE           Run specific test suite (proof, runtime, tools, compiler)
    -j N                  Parallel jobs
    -h, --help            Show this help

Examples:
    ./scripts/test-all.sh                    # All tests
    ./scripts/test-all.sh --skip-proof       # Skip proof tests
    ./scripts/test-all.sh --test runtime     # Runtime tests only

EOF
}

# Parse arguments
SKIP_PROOF=false
SKIP_RUNTIME=false
SKIP_TOOLS=false
SKIP_COMPILER=false
TEST_TYPE=""
JOBS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        --skip-proof)
            SKIP_PROOF=true
            shift
            ;;
        --skip-runtime)
            SKIP_RUNTIME=true
            shift
            ;;
        --skip-tools)
            SKIP_TOOLS=true
            shift
            ;;
        --skip-compiler)
            SKIP_COMPILER=true
            shift
            ;;
        --test)
            TEST_TYPE="$2"
            shift 2
            ;;
        -j)
            JOBS="-j $2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done
[ -z "$JOBS" ] && JOBS="-j$(nproc 2>/dev/null || echo 4)"

case "$BUILD_TYPE" in
    release) CARGO_PROFILE_FLAG="--release" ;;
    debug) CARGO_PROFILE_FLAG="" ;;
    *)
        log_error "Unsupported build type: $BUILD_TYPE"
        exit 1
        ;;
esac

log_info "Chimera Test System"
log_info "===================="
echo ""

# Test proof system
if [[ "$TEST_TYPE" == "" || "$TEST_TYPE" == "proof" ]]; then
    if ! $SKIP_PROOF; then
        log_step "Testing proof system (ChimeraProof)..."
        PROOF_DIR="$PROJECT_ROOT/ChimeraProof"
        if [ -d "$PROOF_DIR" ] && [ -f "$PROOF_DIR/lakefile.toml" ]; then
            if (cd "$PROOF_DIR" && ./test.sh 2>&1) && \
               (cd "$PROJECT_ROOT" && bash scripts/check-placeholders.sh 2>&1); then
                pass "Proof system tests"
            else
                fail "Proof system tests"
            fi
        else
            skip "Proof system not configured"
        fi
    fi
fi

# Test runtime
if [[ "$TEST_TYPE" == "" || "$TEST_TYPE" == "runtime" ]]; then
    if ! $SKIP_RUNTIME; then
        log_step "Testing runtime..."
        RUNTIME_DIR="$PROJECT_ROOT/runtime"
        if [ -d "$RUNTIME_DIR/rust" ]; then
            if (cd "$RUNTIME_DIR/rust" && cargo test $CARGO_PROFILE_FLAG 2>&1); then
                pass "Runtime Rust tests"
            else
                fail "Runtime Rust tests"
            fi
        fi
        # Run C conformance tests
        if [ -f "$RUNTIME_DIR/test_conformance.sh" ]; then
            if (cd "$RUNTIME_DIR" && bash test_conformance.sh 2>&1); then
                pass "Runtime conformance tests"
            else
                fail "Runtime conformance tests"
            fi
        fi
        if [ -f "$RUNTIME_DIR/test_sanitizers.sh" ]; then
            if (cd "$RUNTIME_DIR" && bash test_sanitizers.sh 2>&1); then
                pass "Runtime sanitizer tests"
            else
                fail "Runtime sanitizer tests"
            fi
        fi
    fi
fi

# Test tools
if [[ "$TEST_TYPE" == "" || "$TEST_TYPE" == "tools" ]]; then
    if ! $SKIP_TOOLS; then
        log_step "Testing tools..."
        TOOLS_DIR="$PROJECT_ROOT/tools"
        if [ -d "$TOOLS_DIR" ]; then
            if (cd "$TOOLS_DIR" && cargo test $CARGO_PROFILE_FLAG $JOBS 2>&1); then
                pass "Tools tests"
            else
                fail "Tools tests"
            fi
        fi
    fi
fi

# Test compiler-core
if [[ "$TEST_TYPE" == "" || "$TEST_TYPE" == "compiler" ]]; then
    if ! $SKIP_COMPILER; then
        log_step "Testing compiler-core..."
        COMPILER_DIR="$PROJECT_ROOT/compiler-core"
        if [ -d "$COMPILER_DIR" ]; then
            BUILD_DIR="$COMPILER_DIR/build"
            if [ -d "$BUILD_DIR" ]; then
                if ! (cd "$BUILD_DIR" && cmake --build . 2>&1); then
                    fail "Compiler-core build"
                else
                    TEST_COUNT="$(compiler_test_count "$BUILD_DIR")"
                    if [ "$TEST_COUNT" -eq 0 ]; then
                        fail "Compiler-core tests (zero tests discovered)"
                    elif (cd "$BUILD_DIR" && ctest --output-on-failure $JOBS 2>&1); then
                        pass "Compiler-core tests"
                    else
                        fail "Compiler-core tests"
                    fi
                fi
            else
                skip "Compiler-core not configured"
            fi
        fi
    fi
fi

echo ""
log_info "===================="
log_info "Test Results: $PASSED passed, $FAILED failed, $SKIPPED skipped"
echo ""

if [ $FAILED -gt 0 ]; then
    exit 1
fi
exit 0
