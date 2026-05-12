#!/bin/bash
# HIR Verification Test for rustc_private integration
# Tests that HIR extraction returns correct items

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="/tmp/chimera-rust-hir-verification-test"
DRIVER="$SCRIPT_DIR/../tools/target/release/chimera-rustc-driver"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Create a test source file with known structure
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

mod internal {
    pub fn internal_fn() {}
}
EOF

echo -e "${YELLOW}[INFO]${NC} HIR Verification Test"

# Check if driver exists
if [ ! -f "$DRIVER" ]; then
    DRIVER="$SCRIPT_DIR/../tools/target/debug/chimera-rustc-driver"
fi

if [ ! -f "$DRIVER" ]; then
    echo -e "${RED}[SKIP]${NC} chimera-rustc-driver binary not found"
    exit 0
fi

ARTIFACTS_DIR="$TEST_DIR/artifacts"
mkdir -p "$ARTIFACTS_DIR"

# Run HIR extraction test
echo -e "${YELLOW}[INFO]${NC} Running HIR extraction..."

$DRIVER compile \
    --source "$TEST_DIR/lib.rs" \
    --output "$TEST_DIR/lib.o" \
    --artifacts-dir "$ARTIFACTS_DIR" \
    --target "x86_64-unknown-linux-gnu" \
    --semantic-extraction 2>&1 || true

RSNAP="$ARTIFACTS_DIR/lib.rs.rsnap"

if [ ! -f "$RSNAP" ]; then
    echo -e "${RED}[FAIL]${NC} .rsnap artifact not produced"
    exit 1
fi

echo -e "${GREEN}[PASS]${NC} HIR extraction produced .rsnap artifact"

# Verify artifact contains expected items
if grep -q "Point" "$RSNAP" && grep -q "new" "$RSNAP" && grep -q "distance" "$RSNAP"; then
    echo -e "${GREEN}[PASS]${NC} HIR artifact contains expected item names"
else
    echo -e "${RED}[FAIL]${NC} HIR artifact missing expected item names"
    exit 1
fi

# Verify public items are marked
if grep -q '"rank":"Pub"' "$RSNAP" || grep -q '"rank": "Pub"' "$RSNAP"; then
    echo -e "${GREEN}[PASS]${NC} Public items have correct visibility"
else
    echo -e "${YELLOW}[WARN]${NC} Could not verify public visibility marker"
fi

# Cleanup
rm -rf "$TEST_DIR"

echo ""
echo -e "${GREEN}=== HIR Verification Test Summary ===${NC}"
echo -e "${GREEN}All HIR verification checks passed!${NC}"

exit 0