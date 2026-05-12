#!/bin/bash
# C Mutation Test Suite (Task 158)
# Tests that changes to C fixtures properly invalidate cached artifacts

FIXTURES_DIR="tests/c-fixtures"
CACHE_DIR="${FIXTURES_DIR}/.cache"
mkdir -p "$CACHE_DIR"

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

echo "=== C Mutation Test Suite (Task 158) ==="
echo ""

# Test 1: Source body mutation - private function body change
log_info "Test 1: Source body mutation (private function body change)"
backup_fixture "$FIXTURES_DIR/source-body/source_body.h"
cat >> "$FIXTURES_DIR/source-body/source_body.h" << 'MUTATION'
// MUTATION: Added function
int multiply(int a, int b) { return a * b * 2; }
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/source-body/source_body.h" 2>/dev/null; then
    log_pass "Source body mutation compiles"
else
    log_fail "Source body mutation failed to compile"
fi
restore_fixture "$FIXTURES_DIR/source-body/source_body.h"

# Test 2: Struct layout mutation
log_info "Test 2: Struct layout mutation"
backup_fixture "$FIXTURES_DIR/layout/layout.h"
cat >> "$FIXTURES_DIR/layout/layout.h" << 'MUTATION'
// MUTATION: Added field
int mutation_marker;
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/layout/layout.h" 2>/dev/null; then
    log_pass "Struct layout mutation compiles"
else
    log_fail "Struct layout mutation failed to compile"
fi
restore_fixture "$FIXTURES_DIR/layout/layout.h"

# Test 3: Macro value mutation
log_info "Test 3: Macro value mutation (ABI-affecting)"
backup_fixture "$FIXTURES_DIR/preprocessor/preprocessor.h"
sed -i 's/#define POINTER_SIZE 8/#define POINTER_SIZE 4/' "$FIXTURES_DIR/preprocessor/preprocessor.h"
if clang -fsyntax-only -I. "$FIXTURES_DIR/preprocessor/preprocessor.h" 2>/dev/null; then
    log_pass "Macro mutation compiles (ABI changed)"
else
    log_fail "Macro mutation failed"
fi
restore_fixture "$FIXTURES_DIR/preprocessor/preprocessor.h"

# Test 4: Flexible array boundary change
log_info "Test 4: Flexible array size contract mutation"
backup_fixture "$FIXTURES_DIR/flexible-array/flexible_array.h"
sed -i 's/(cap)/(cap * 2)/' "$FIXTURES_DIR/flexible-array/flexible_array.h"
if clang -fsyntax-only -I. "$FIXTURES_DIR/flexible-array/flexible_array.h" 2>/dev/null; then
    log_pass "Flexible array contract mutation compiles"
else
    log_fail "Flexible array contract mutation failed"
fi
restore_fixture "$FIXTURES_DIR/flexible-array/flexible_array.h"

# Test 5: Bitfield width mutation
log_info "Test 5: Bitfield width mutation"
backup_fixture "$FIXTURES_DIR/bitfields/bitfield.h"
sed -i 's/: 8/: 16/' "$FIXTURES_DIR/bitfields/bitfield.h"
if clang -fsyntax-only -I. "$FIXTURES_DIR/bitfields/bitfield.h" 2>/dev/null; then
    log_pass "Bitfield width mutation compiles"
else
    log_fail "Bitfield width mutation failed"
fi
restore_fixture "$FIXTURES_DIR/bitfields/bitfield.h"

# Test 6: Callback signature mutation
log_info "Test 6: Callback function pointer signature change"
backup_fixture "$FIXTURES_DIR/callbacks/callbacks.h"
cat > "$FIXTURES_DIR/callbacks/callbacks.h" << 'MUTATION'
#ifndef CALLBACKS_H
#define CALLBACKS_H
/* Task 153: Callback fixture */

// Callback with 3 int params
typedef int (*callback_t)(int, int, int);
callback_t register_callback(callback_t cb);
int invoke_callback(callback_t cb, int a, int b, int c);
#endif
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/callbacks/callbacks.h" 2>/dev/null; then
    log_pass "Callback signature mutation compiles"
else
    log_fail "Callback signature mutation failed"
fi
restore_fixture "$FIXTURES_DIR/callbacks/callbacks.h"

# Test 7: Error code mutation
log_info "Test 7: Error code convention change"
backup_fixture "$FIXTURES_DIR/errors/errors.h"
cat > "$FIXTURES_DIR/errors/errors.h" << 'MUTATION'
#ifndef ERRORS_H
#define ERRORS_H
/* Task 152: Errno/status fixture */

enum error_code {
    ERROR_NONE = 0,
    ERROR_INVALID = -1,
    ERROR_MUTATED = -2
};

int lookup_error(void);
const char* error_to_string(int code);
#endif
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/errors/errors.h" 2>/dev/null; then
    log_pass "Error code mutation compiles"
else
    log_fail "Error code mutation failed"
fi
restore_fixture "$FIXTURES_DIR/errors/errors.h"

# Test 8: Allocator function signature mutation
log_info "Test 8: Allocator function signature change"
backup_fixture "$FIXTURES_DIR/allocator/allocator.h"
cat > "$FIXTURES_DIR/allocator/allocator.h" << 'MUTATION'
#ifndef ALLOCATOR_H
#define ALLOCATOR_H
/* Task 154: Allocator fixture */

#include <stddef.h>

// Mutation: Added alignment parameter
void* chimera_alloc(size_t size, size_t alignment);
void chimera_free(void* ptr, size_t alignment);

// Owned memory handle
struct Handle {
    void* data;
    size_t size;
    size_t alignment;
};

struct Handle* chimera_handle_create(size_t size, size_t alignment);
void chimera_handle_destroy(struct Handle* h);

// Drop trampoline for foreign callers
void chimera_drop_owned(void* ptr);
#endif
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/allocator/allocator.h" 2>/dev/null; then
    log_pass "Allocator signature mutation compiles"
else
    log_fail "Allocator signature mutation failed"
fi
restore_fixture "$FIXTURES_DIR/allocator/allocator.h"

# Test 9: Varargs ABI change
log_info "Test 9: Varargs function signature change"
backup_fixture "$FIXTURES_DIR/varargs/varargs.h"
cat > "$FIXTURES_DIR/varargs/varargs.h" << 'MUTATION'
#ifndef VARARGS_H
#define VARARGS_H
/* Task 156: Varargs fixture */

#include <stdarg.h>

// Mutation: Changed parameter types
int sum(int count, double first, ...);
void log_message(int level, const char* fmt, ...);
int vprintf_wrapper(const char* fmt, va_list args);

#ifdef __GNUC__
#define VA_START(ap, last) __builtin_va_start(ap, last)
#define VA_END(ap) __builtin_va_end(ap)
#else
#include <stdarg.h>
#define VA_START(ap, last) va_start(ap, last)
#define VA_END(ap) va_end(ap)
#endif
#endif
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/varargs/varargs.h" 2>/dev/null; then
    log_pass "Varargs signature mutation compiles"
else
    log_fail "Varargs signature mutation failed"
fi
restore_fixture "$FIXTURES_DIR/varargs/varargs.h"

# Test 10: Header-only fixture extension
log_info "Test 10: Header-only fixture extension"
backup_fixture "$FIXTURES_DIR/header-only/header.h"
cat >> "$FIXTURES_DIR/header-only/header.h" << 'MUTATION'
// MUTATION: Added enum
enum mutation_enum { MUT_A = 0, MUT_B = 1 };
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/header-only/header.h" 2>/dev/null; then
    log_pass "Header-only extension compiles"
else
    log_fail "Header-only extension failed"
fi
restore_fixture "$FIXTURES_DIR/header-only/header.h"

# Test 11: Basic fixture extension
log_info "Test 11: Basic fixture extension"
backup_fixture "$FIXTURES_DIR/basic/basic.h"
cat >> "$FIXTURES_DIR/basic/basic.h" << 'MUTATION'
// MUTATION: Added function
int multiply(int a, int b);
MUTATION
if clang -fsyntax-only -I. "$FIXTURES_DIR/basic/basic.h" 2>/dev/null; then
    log_pass "Basic fixture extension compiles"
else
    log_fail "Basic fixture extension failed"
fi
restore_fixture "$FIXTURES_DIR/basic/basic.h"

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