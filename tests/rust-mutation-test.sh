#!/bin/bash
# Rust Mutation Test Suite (Task 171)
# Tests that changes to Rust fixtures properly invalidate cached artifacts

set -e

RUST_FIXTURES_DIR="tests/rust-fixtures"
RUST_CACHE_DIR="${RUST_FIXTURES_DIR}/.cache"
mkdir -p "$RUST_CACHE_DIR"

# Colors for output
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

# Backup a fixture file
backup_fixture() {
    local file="$1"
    if [ -f "$file" ]; then
        cp "$file" "${file}.backup"
    fi
}

# Restore a fixture file
restore_fixture() {
    local file="$1"
    if [ -f "${file}.backup" ]; then
        mv "${file}.backup" "$file"
    fi
}

# Run cargo test on a fixture
run_fixture_test() {
    local fixture="$1"
    local manifest="${RUST_FIXTURES_DIR}/${fixture}/Cargo.toml"
    if [ -f "$manifest" ]; then
        cargo test --manifest-path "$manifest" 2>&1 | tail -5
    fi
}

echo "=== Rust Mutation Test Suite (Task 171) ==="
echo ""

# Test 1: Basic fixture function body mutation
log_info "Test 1: Basic fixture function body mutation"
backup_fixture "${RUST_FIXTURES_DIR}/basic/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/basic/src/lib.rs" << 'MUTATION'
// MUTATION: Added function
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b * 2
}
MUTATION
if run_fixture_test "basic" > /dev/null 2>&1; then
    log_pass "Basic fixture body mutation compiles"
else
    log_fail "Basic fixture body mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/basic/src/lib.rs"

# Test 2: Layout struct field mutation
log_info "Test 2: Layout struct field addition"
backup_fixture "${RUST_FIXTURES_DIR}/layout/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/layout/src/lib.rs" << 'MUTATION'
// MUTATION: Added field to Rectangle
#[repr(C)]
pub struct MutatedRectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub mutation_marker: i32,
}
MUTATION
if run_fixture_test "layout" > /dev/null 2>&1; then
    log_pass "Layout struct field mutation compiles"
else
    log_fail "Layout struct field mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/layout/src/lib.rs"

# Test 3: FFI Result type mutation
log_info "Test 3: FFI error code convention change"
backup_fixture "${RUST_FIXTURES_DIR}/ffi/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/ffi/src/lib.rs" << 'MUTATION'
// MUTATION: Added new error code
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MutatedError {
    pub code: i32,
    pub message: [u8; 64],
    pub extra_field: u64,
}
MUTATION
if run_fixture_test "ffi" > /dev/null 2>&1; then
    log_pass "FFI error type mutation compiles"
else
    log_fail "FFI error type mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/ffi/src/lib.rs"

# Test 4: Slice function signature mutation
log_info "Test 4: Slice function parameter type change"
backup_fixture "${RUST_FIXTURES_DIR}/slices/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/slices/src/lib.rs" << 'MUTATION'
// MUTATION: New function with different slice type
#[no_mangle]
pub extern "C" fn sum_i16(data: *const i16, len: usize) -> i32 {
    if data.is_null() { return 0; }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    slice.iter().map(|&x| x as i32).sum()
}
MUTATION
if run_fixture_test "slices" > /dev/null 2>&1; then
    log_pass "Slice function signature mutation compiles"
else
    log_fail "Slice function signature mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/slices/src/lib.rs"

# Test 5: Panic function behavior mutation
log_info "Test 5: Panic function behavior change"
backup_fixture "${RUST_FIXTURES_DIR}/panic/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/panic/src/lib.rs" << 'MUTATION'
// MUTATION: Changed panic behavior
#[no_mangle]
pub extern "C" fn mutated_panic_function(value: i32) -> i32 {
    if value < 0 {
        panic!("mutated: negative value {}", value);
    }
    value * 4
}
MUTATION
if run_fixture_test "panic" > /dev/null 2>&1; then
    log_pass "Panic function behavior mutation compiles"
else
    log_fail "Panic function behavior mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/panic/src/lib.rs"

# Test 6: Unsafe function addition
log_info "Test 6: Unsafe function addition"
backup_fixture "${RUST_FIXTURES_DIR}/unsafe/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/unsafe/src/lib.rs" << 'MUTATION'
// MUTATION: New unsafe function
#[no_mangle]
pub unsafe extern "C" fn mutated_raw_u64_add(ptr: *mut u64, offset: usize, value: u64) -> bool {
    if ptr.is_null() { return false; }
    let target = ptr.wrapping_add(offset);
    if target.is_null() { return false; }
    *target = (*target).wrapping_add(value);
    true
}
MUTATION
if run_fixture_test "unsafe" > /dev/null 2>&1; then
    log_pass "Unsafe function addition compiles"
else
    log_fail "Unsafe function addition failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/unsafe/src/lib.rs"

# Test 7: Generic type instantiation mutation
log_info "Test 7: Generic function new type instantiation"
backup_fixture "${RUST_FIXTURES_DIR}/generics/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/generics/src/lib.rs" << 'MUTATION'
// MUTATION: New identity for i16
#[no_mangle]
pub extern "C" fn identity_i16(value: i16) -> IdentityResult {
    IdentityResult {
        value: value as u64,
        type_tag: TYPE_TAG_I64,
    }
}
MUTATION
if run_fixture_test "generics" > /dev/null 2>&1; then
    log_pass "Generic type instantiation mutation compiles"
else
    log_fail "Generic type instantiation mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/generics/src/lib.rs"

# Test 8: Ownership type mutation
log_info "Test 8: Ownership handle type field addition"
backup_fixture "${RUST_FIXTURES_DIR}/owned/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/owned/src/lib.rs" << 'MUTATION'
// MUTATION: Extended ChHandle with metadata
#[repr(C)]
pub struct MutatedHandle {
    pub ptr: *mut u8,
    pub size: usize,
    pub drop_trampoline: unsafe extern "C" fn(*mut u8),
    pub metadata: u64,
}
MUTATION
if run_fixture_test "owned" > /dev/null 2>&1; then
    log_pass "Ownership handle type mutation compiles"
else
    log_fail "Ownership handle type mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/owned/src/lib.rs"

# Test 9: Feature flag mutation
log_info "Test 9: Feature flag dependent compilation"
backup_fixture "${RUST_FIXTURES_DIR}/workspace/helper-crate/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/workspace/helper-crate/src/lib.rs" << 'MUTATION'
// MUTATION: New feature-dependent function
#[cfg(feature = "nightly")]
#[no_mangle]
pub extern "C" fn helper_nightly_only() -> i32 {
    42
}
MUTATION
(cd "${RUST_FIXTURES_DIR}/workspace" && cargo test -p helper-crate --features nightly 2>&1 | tail -5) || log_fail "Feature flag mutation failed"
restore_fixture "${RUST_FIXTURES_DIR}/workspace/helper-crate/src/lib.rs"

# Test 10: Const generic mutation
log_info "Test 10: Const generic value change"
backup_fixture "${RUST_FIXTURES_DIR}/generics/src/lib.rs"
cat >> "${RUST_FIXTURES_DIR}/generics/src/lib.rs" << 'MUTATION'
// MUTATION: New const generic function
fn const_generic_identity<const N: usize>(value: u64) -> u64 {
    value.wrapping_add(N as u64)
}

#[no_mangle]
pub extern "C" fn const_generic_twenty() -> u64 {
    const_generic_identity::<20>(0)
}
MUTATION
if run_fixture_test "generics" > /dev/null 2>&1; then
    log_pass "Const generic mutation compiles"
else
    log_fail "Const generic mutation failed"
fi
restore_fixture "${RUST_FIXTURES_DIR}/generics/src/lib.rs"

# Test 11: Build.rs environment change
log_info "Test 11: Build.rs environment variable mutation"
backup_fixture "${RUST_FIXTURES_DIR}/workspace/main-crate/build.rs"
echo 'println!("cargo:rustc-env=BUILD_SCRIPT_RAN=1")' >> "${RUST_FIXTURES_DIR}/workspace/main-crate/build.rs"
echo 'println!("cargo:rustc-env=MUTATION_MARKER=1")' >> "${RUST_FIXTURES_DIR}/workspace/main-crate/build.rs"
(cd "${RUST_FIXTURES_DIR}/workspace" && cargo build -p main-crate 2>&1 | tail -3) || log_fail "Build.rs mutation failed"
restore_fixture "${RUST_FIXTURES_DIR}/workspace/main-crate/build.rs"

# Summary
echo ""
echo "=== Mutation Test Summary ==="
echo -e "Passed: ${GREEN}${passed}${NC}"
echo -e "Failed: ${RED}${failed}${NC}"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All mutation tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some mutation tests failed!${NC}"
    exit 1
fi