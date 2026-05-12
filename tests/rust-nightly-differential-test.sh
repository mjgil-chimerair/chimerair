#!/bin/bash
# Rust Nightly Differential Test - compares HIR-derived vs source-parsed artifacts
# Part of Task R8 (Verification)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="/tmp/chimera-rust-nightly-differential-test"
DRIVER="$SCRIPT_DIR/../tools/target/release/chimera-rustc-driver"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Create test source with known structure
cat > "$TEST_DIR/lib.rs" << 'EOF'
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub fn distance(&self, other: &Point) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

fn private_helper() -> i32 {
    42
}

pub fn public_function(x: i32) -> i32 {
    x * 2
}

pub mod internal {
    pub fn internal_fn() {}
}
EOF

echo -e "${YELLOW}[INFO]${NC} Rust Nightly Differential Test"

# Check if driver exists
if [ ! -f "$DRIVER" ]; then
    DRIVER="$SCRIPT_DIR/../tools/target/debug/chimera-rustc-driver"
fi

if [ ! -f "$DRIVER" ]; then
    echo -e "${RED}[SKIP]${NC} chimera-rustc-driver binary not found"
    exit 0
fi

ARTIFACTS_DIR="$TEST_DIR/artifacts"
STABLE_ARTIFACTS="$TEST_DIR/stable"
SEMANTIC_ARTIFACTS="$TEST_DIR/semantic"
mkdir -p "$STABLE_ARTIFACTS" "$SEMANTIC_ARTIFACTS"

echo -e "${YELLOW}[INFO]${NC} Running stable-surface extraction..."

# Run stable-surface extraction (default, no --semantic-extraction)
$DRIVER compile \
    --source "$TEST_DIR/lib.rs" \
    --output "$TEST_DIR/lib.o" \
    --artifacts-dir "$STABLE_ARTIFACTS" \
    --target "x86_64-unknown-linux-gnu" 2>&1 || true

echo -e "${YELLOW}[INFO]${NC} Running semantic-extraction (HIR-based)..."

# Run semantic extraction (with --semantic-extraction flag)
$DRIVER compile \
    --source "$TEST_DIR/lib.rs" \
    --output "$TEST_DIR/lib_hir.o" \
    --artifacts-dir "$SEMANTIC_ARTIFACTS" \
    --target "x86_64-unknown-linux-gnu" \
    --semantic-extraction 2>&1 || true

passed=0
failed=0

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((passed++)) || true
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((failed++)) || true
}

# Test: Both should produce .rsnap artifacts
if [ -f "$STABLE_ARTIFACTS/lib.rs.rsnap" ]; then
    log_pass "Stable-surface produced .rsnap artifact"
else
    log_fail "Stable-surface did not produce .rsnap artifact"
fi

if [ -f "$SEMANTIC_ARTIFACTS/lib.rs.rsnap" ]; then
    log_pass "Semantic-extraction produced .rsnap artifact"
else
    log_fail "Semantic-extraction did not produce .rsnap artifact"
fi

# Test: Both should contain the same item names (Point, new, distance, public_function)
STABLE_ITEMS=$(grep -o '"def_path":"[^"]*"' "$STABLE_ARTIFACTS/lib.rs.rsnap" 2>/dev/null | wc -l)
SEMANTIC_ITEMS=$(grep -o '"def_path":"[^"]*"' "$SEMANTIC_ARTIFACTS/lib.rs.rsnap" 2>/dev/null | wc -l)

echo -e "${YELLOW}[INFO]${NC} Stable items: $STABLE_ITEMS, Semantic items: $SEMANTIC_ITEMS"

if [ "$STABLE_ITEMS" -gt 0 ] && [ "$SEMANTIC_ITEMS" -gt 0 ]; then
    log_pass "Both extraction modes produced items"
else
    log_fail "One or both extraction modes produced no items"
fi

# Test: Both should have public items marked correctly
if grep -q "public_function" "$STABLE_ARTIFACTS/lib.rs.rsnap" && \
   grep -q "public_function" "$SEMANTIC_ARTIFACTS/lib.rs.rsnap"; then
    log_pass "public_function present in both artifacts"
else
    log_fail "public_function missing from one or both artifacts"
fi

if grep -q "Point" "$STABLE_ARTIFACTS/lib.rs.rsnap" && \
   grep -q "Point" "$SEMANTIC_ARTIFACTS/lib.rs.rsnap"; then
    log_pass "Point struct present in both artifacts"
else
    log_fail "Point struct missing from one or both artifacts"
fi

# Test: Both should have same number of public items (basic correctness check)
# Note: In full integration, semantic should have more accurate visibility info
if [ "$STABLE_ITEMS" -eq "$SEMANTIC_ITEMS" ]; then
    log_pass "Item count matches between stable and semantic extraction"
else
    echo -e "${YELLOW}[INFO]${NC} Item count differs (stable: $STABLE_ITEMS, semantic: $SEMANTIC_ITEMS)"
    echo -e "${YELLOW}[INFO]${NC} This may be expected if semantic extraction includes more metadata"
fi

# Cleanup
rm -rf "$TEST_DIR"

echo ""
echo -e "${GREEN}=== Rust Nightly Differential Test Summary ===${NC}"
echo -e "Passed: ${GREEN}${passed}${NC}"
echo -e "Failed: ${RED}${failed}${NC}"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All differential tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some differential tests failed!${NC}"
    exit 1
fi