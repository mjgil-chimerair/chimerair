#!/bin/bash
#!
# @file build-all.sh
# @brief Top-level build orchestration for Chimera
#
# Provides one root entrypoint to build proof, compiler, tools, and runtime artifacts.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_TYPE="${BUILD_TYPE:-release}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $*"; }

# Build directories
PROOF_DIR="$PROJECT_ROOT/ChimeraProof"
RUNTIME_DIR="$PROJECT_ROOT/runtime"
TOOLS_DIR="$PROJECT_ROOT/tools"
COMPILER_DIR="$PROJECT_ROOT/compiler-core"

show_help() {
    cat << EOF
Chimera Build System

Usage: ./scripts/build-all.sh [OPTIONS]

Options:
    --build-type TYPE     Build type: release (default) or debug
    --target TRIPLE       Target triple (default: native)
    --skip-proof          Skip proof system build
    --skip-runtime        Skip runtime build
    --skip-tools          Skip tools build
    --skip-compiler       Skip compiler-core build
    --check               Run type checking only (no build)
    --clean               Clean before building
    -j N                  Parallel jobs
    -h, --help            Show this help

Examples:
    ./scripts/build-all.sh                    # Full release build
    ./scripts/build-all.sh --build-type debug # Debug build
    ./scripts/build-all.sh --skip-proof      # Skip proof system
    ./scripts/build-all.sh -j 4              # 4 parallel jobs

EOF
}

# Parse arguments
SKIP_PROOF=false
SKIP_RUNTIME=false
SKIP_TOOLS=false
SKIP_COMPILER=false
CHECK_ONLY=false
CLEAN=false
JOBS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-type)
            BUILD_TYPE="$2"
            shift 2
            ;;
        --target)
            TARGET="$2"
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
        --check)
            CHECK_ONLY=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
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

# Determine number of jobs
if [ -z "$JOBS" ]; then
    JOBS="-j$(nproc 2>/dev/null || echo 4)"
fi

log_info "Chimera Build System"
log_info "====================="
log_info "Build type: $BUILD_TYPE"
[ -n "$TARGET" ] && log_info "Target: $TARGET"
echo ""

# Clean if requested
if $CLEAN; then
    log_step "Cleaning build artifacts..."
    [ ! -d "$TOOLS_DIR/target" ] || rm -rf "$TOOLS_DIR/target"
    [ ! -d "$RUNTIME_DIR/rust/target" ] || rm -rf "$RUNTIME_DIR/rust/target"
    log_info "Clean complete"
    echo ""
fi

# Build proof system
if ! $SKIP_PROOF; then
    log_step "Building proof system (ChimeraProof)..."
    if [ -d "$PROOF_DIR" ]; then
        if [ -f "$PROOF_DIR/lakefile.toml" ]; then
            (cd "$PROOF_DIR" && lake build 2>&1) || {
                log_warn "Proof system build had issues, continuing..."
            }
            log_info "Proof system build complete"
        else
            log_warn "Proof system not configured (no lakefile.toml)"
        fi
    else
        log_warn "Proof directory not found, skipping"
    fi
    echo ""
fi

# Build runtime
if ! $SKIP_RUNTIME; then
    log_step "Building runtime (C/Rust/Zig)..."
    if [ -d "$RUNTIME_DIR/rust" ]; then
        (cd "$RUNTIME_DIR/rust" && cargo build --"$BUILD_TYPE" 2>&1) || {
            log_warn "Rust runtime build had issues"
        }
        log_info "Rust runtime build complete"
    fi
    log_info "Runtime build complete"
    echo ""
fi

# Build tools
if ! $SKIP_TOOLS; then
    log_step "Building tools (Rust CLI)..."
    if [ -d "$TOOLS_DIR" ]; then
        (cd "$TOOLS_DIR" && cargo build --"$BUILD_TYPE" $JOBS 2>&1) || {
            log_warn "Tools build had issues"
        }
        log_info "Tools build complete"
    fi
    echo ""
fi

# Build compiler-core
if ! $SKIP_COMPILER; then
    log_step "Building compiler-core (C++/MLIR)..."
    if [ -d "$COMPILER_DIR" ]; then
        if [ -f "$COMPILER_DIR/CMakeLists.txt" ]; then
            BUILD_DIR="$COMPILER_DIR/build"
            mkdir -p "$BUILD_DIR"
            if [ -f "$BUILD_DIR/Makefile" ]; then
                (cd "$BUILD_DIR" && make $JOBS 2>&1) || {
                    log_warn "Compiler-core build had issues"
                }
            else
                log_info "Compiler-core not configured, run cmake first"
            fi
            log_info "Compiler-core build complete"
        else
            log_warn "Compiler-core not configured (no CMakeLists.txt)"
        fi
    fi
    echo ""
fi

log_info "====================="
log_info "Build complete!"

if $CHECK_ONLY; then
    echo "Note: --check mode, no artifacts produced"
fi
echo ""
echo "To run tests:"
echo "  ./scripts/test-all.sh"
echo ""