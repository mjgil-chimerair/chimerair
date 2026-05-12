#!/usr/bin/env bash
# Top-level test orchestration for Chimera
# Runs all tests: proof, compiler-core, tools, runtime

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Testing Chimera ==="

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

# Test ChimeraProof (Lean)
echo "--- Testing ChimeraProof ---"
if [ -d "ChimeraProof" ]; then
    echo "Running release contract checks..."
    bash scripts/release-gate.sh --contracts-only
    ./ChimeraProof/test.sh
    echo "Checking for placeholders..."
    bash scripts/check-placeholders.sh
else
    echo "Warning: ChimeraProof not found, skipping"
fi

# Test compiler-core (C++/MLIR)
echo "--- Testing compiler-core ---"
if [ -d "compiler-core" ]; then
    if [ -d "compiler-core/build" ]; then
        echo "Building compiler-core test targets..."
        cmake --build compiler-core/build
        TEST_COUNT="$(compiler_test_count "compiler-core/build")"
        if [ "$TEST_COUNT" -eq 0 ]; then
            echo "Error: compiler-core/build has zero registered tests"
            exit 1
        fi
        cd compiler-core/build
        ctest --output-on-failure
        cd ../..
    else
        echo "compiler-core not built yet, skipping"
    fi
else
    echo "Warning: compiler-core not found, skipping"
fi

# Test tools (Rust)
echo "--- Testing tools ---"
if [ -d "tools" ]; then
    cd tools
    cargo test --workspace
    cd ..
else
    echo "Warning: tools not found, skipping"
fi

# Test runtime conformance
echo "--- Testing runtime conformance ---"
if [ -f "runtime/test_conformance.sh" ]; then
    cd runtime
    bash test_conformance.sh
    cd ..
fi

echo "--- Testing runtime sanitizers ---"
if [ -f "runtime/test_sanitizers.sh" ]; then
    cd runtime
    bash test_sanitizers.sh
    cd ..
fi

echo "=== All tests passed ==="
