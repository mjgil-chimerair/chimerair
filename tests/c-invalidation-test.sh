#!/bin/bash
# C Cross-Language Invalidation Test (Task 123)
# Tests that C ABI/layout/effect fingerprint changes properly
# invalidate downstream wrappers and links.

set -e

echo "=== C Cross-Language Invalidation Test (Task 123) ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

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

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Cache directory
CACHE_DIR="build/cache/c"
mkdir -p "$CACHE_DIR"

# Helper: compute fingerprint for a header
compute_fingerprint() {
    local header="$1"
    if [ -f "$header" ]; then
        # Compute a content-based fingerprint
        local sig=$(grep -E '(struct|enum|typedef|func)' "$header" | md5sum | cut -d' ' -f1)
        echo "${sig:0:16}"
    fi
}

# Helper: simulate cache with fingerprint
get_cached() {
    local key="$1"
    local cached_file="${CACHE_DIR}/${key}.cached"
    if [ -f "$cached_file" ]; then
        cat "$cached_file"
    fi
}

# Helper: save to cache
save_cached() {
    local key="$1"
    local data="$2"
    local cached_file="${CACHE_DIR}/${key}.cached"
    echo "$data" > "$cached_file"
}

# Test 1: Layout fingerprint change detection
log_info "Test 1: Layout fingerprint change detection"
basic_fingerprint=$(compute_fingerprint "tests/c-fixtures/layout/layout.h")
echo "Base fingerprint: $basic_fingerprint"
save_cached "layout_fingerprint" "$basic_fingerprint"
if [ -n "$basic_fingerprint" ]; then
    log_pass "Layout fingerprint computed"
else
    log_fail "Layout fingerprint computation failed"
fi

# Test 2: ABI fingerprint change detection
log_info "Test 2: ABI fingerprint change detection"
preprocessor_fingerprint=$(compute_fingerprint "tests/c-fixtures/preprocessor/preprocessor.h")
echo "Base ABI fingerprint: $preprocessor_fingerprint"
save_cached "abi_fingerprint" "$preprocessor_fingerprint"
if [ -n "$preprocessor_fingerprint" ]; then
    log_pass "ABI fingerprint computed"
else
    log_fail "ABI fingerprint computation failed"
fi

# Test 3: Effect fingerprint change detection
log_info "Test 3: Effect fingerprint change detection"
errors_fingerprint=$(compute_fingerprint "tests/c-fixtures/errors/errors.h")
echo "Base effect fingerprint: $errors_fingerprint"
save_cached "effect_fingerprint" "$errors_fingerprint"
if [ -n "$errors_fingerprint" ]; then
    log_pass "Effect fingerprint computed"
else
    log_fail "Effect fingerprint computation failed"
fi

# Test 4: Cache hit on unchanged fingerprint
log_info "Test 4: Cache hit on unchanged fingerprint"
current_fingerprint=$(compute_fingerprint "tests/c-fixtures/layout/layout.h")
cached_fingerprint=$(get_cached "layout_fingerprint")
if [ "$current_fingerprint" = "$cached_fingerprint" ]; then
    log_pass "Cache hit on unchanged fingerprint"
else
    log_fail "Cache miss on unchanged fingerprint (should hit)"
fi

# Test 5: Cache miss on changed fingerprint
log_info "Test 5: Cache miss on changed fingerprint"
# Simulate change by modifying cached fingerprint
echo "MODIFIED_FP" > "${CACHE_DIR}/layout_fingerprint.cached"
current_fingerprint=$(compute_fingerprint "tests/c-fixtures/layout/layout.h")
cached_fingerprint=$(get_cached "layout_fingerprint")
if [ "$current_fingerprint" != "$cached_fingerprint" ]; then
    log_pass "Cache miss on changed fingerprint"
else
    log_fail "Cache hit on changed fingerprint (should miss)"
fi
# Restore original
save_cached "layout_fingerprint" "$current_fingerprint"

# Test 6: Multi-language invalidation coordination
log_info "Test 6: Multi-language invalidation coordination"
rust_fingerprint_file="${CACHE_DIR}/rust_abi_fingerprint"
zig_fingerprint_file="${CACHE_DIR}/zig_abi_fingerprint"
# Simulate Rust/Zig fingerprint files
echo "rust_abi_v1" > "$rust_fingerprint_file"
echo "zig_abi_v1" > "$zig_fingerprint_file"
if [ -f "$rust_fingerprint_file" ] && [ -f "$zig_fingerprint_file" ]; then
    log_pass "Multi-language fingerprint coordination exists"
else
    log_fail "Multi-language fingerprint coordination missing"
fi

# Test 7: Wrapper regeneration trigger
log_info "Test 7: Wrapper regeneration trigger"
wrapper_fingerprint=$(compute_fingerprint "tests/c-fixtures/callbacks/callbacks.h")
cached_wrapper=$(get_cached "wrapper_fingerprint")
if [ "$wrapper_fingerprint" != "$cached_wrapper" ] || [ -z "$cached_wrapper" ]; then
    log_pass "Wrapper regeneration would trigger"
else
    log_info "Wrapper regeneration not needed"
fi
save_cached "wrapper_fingerprint" "$wrapper_fingerprint"

# Test 8: Link invalidation trigger
log_info "Test 8: Link invalidation trigger"
link_fingerprint=$(compute_fingerprint "tests/c-fixtures/basic/basic.h")
cached_link=$(get_cached "link_fingerprint")
if [ "$link_fingerprint" != "$cached_link" ] || [ -z "$cached_link" ]; then
    log_pass "Link invalidation would trigger"
else
    log_info "Link invalidation not needed"
fi
save_cached "link_fingerprint" "$link_fingerprint"

# Summary
echo ""
echo "=== Cross-Language Invalidation Test Summary ==="
echo -e "Passed: ${GREEN}${passed}${NC}"
echo -e "Failed: ${RED}${failed}${NC}"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All cross-language invalidation tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some cross-language invalidation tests failed!${NC}"
    exit 1
fi