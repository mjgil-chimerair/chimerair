#!/usr/bin/env bash
# Test: repository structure validation
# Validates that all required directories and files exist

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$REPO_ROOT"

FAILED=0

echo "=== Testing Repository Structure ==="

# Test 1: Required directories exist
echo "Checking required directories..."
REQUIRED_DIRS=(
    "compiler-core"
    "tools"
    "runtime"
    "examples"
    "ChimeraProof"
    "docs"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "  [PASS] $dir/ exists"
    else
        echo "  [FAIL] $dir/ missing"
        FAILED=1
    fi
done

# Test 2: Version definition exists
echo "Checking version definitions..."
if [ -f "docs/version-definitions.toml" ]; then
    echo "  [PASS] docs/version-definitions.toml exists"
else
    echo "  [FAIL] docs/version-definitions.toml missing"
    FAILED=1
fi

# Test 3: Toolchain pin exists
echo "Checking toolchain pin..."
if [ -f "toolchain.toml" ]; then
    echo "  [PASS] toolchain.toml exists"
else
    echo "  [FAIL] toolchain.toml missing"
    FAILED=1
fi

# Test 4: Build orchestration scripts
echo "Checking build scripts..."
for script in build.sh test.sh; do
    if [ -x "$script" ]; then
        echo "  [PASS] $script is executable"
    elif [ -f "$script" ]; then
        echo "  [WARN] $script exists but not executable"
    else
        echo "  [FAIL] $script missing"
        FAILED=1
    fi
done

# Test 5: Required documentation
echo "Checking required documentation..."
REQUIRED_DOCS=(
    "docs/versioning.md"
    "docs/artifact-flow.md"
    "docs/build.md"
    "docs/testing.md"
)

for doc in "${REQUIRED_DOCS[@]}"; do
    if [ -f "$doc" ]; then
        echo "  [PASS] $doc exists"
    else
        echo "  [FAIL] $doc missing"
        FAILED=1
    fi
done

# Test 6: compiler-core structure
echo "Checking compiler-core structure..."
if [ -f "compiler-core/README.md" ]; then
    echo "  [PASS] compiler-core/README.md exists"
else
    echo "  [FAIL] compiler-core/README.md missing"
    FAILED=1
fi

# Test 7: tools structure
echo "Checking tools structure..."
if [ -f "tools/README.md" ]; then
    echo "  [PASS] tools/README.md exists"
else
    echo "  [FAIL] tools/README.md missing"
    FAILED=1
fi

# Test 8: runtime structure
echo "Checking runtime structure..."
if [ -f "runtime/README.md" ]; then
    echo "  [PASS] runtime/README.md exists"
else
    echo "  [FAIL] runtime/README.md missing"
    FAILED=1
fi

# Test 9: examples structure
echo "Checking examples structure..."
if [ -f "examples/README.md" ]; then
    echo "  [PASS] examples/README.md exists"
else
    echo "  [FAIL] examples/README.md missing"
    FAILED=1
fi

# Test 10: CI configuration
echo "Checking CI configuration..."
if [ -f ".github/workflows/ci.yml" ]; then
    echo "  [PASS] .github/workflows/ci.yml exists"
else
    echo "  [FAIL] .github/workflows/ci.yml missing"
    FAILED=1
fi

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "=== All repository structure tests passed ==="
    exit 0
else
    echo ""
    echo "=== Repository structure tests FAILED ==="
    exit 1
fi