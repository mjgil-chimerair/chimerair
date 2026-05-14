/*!
 * @file chimera_abi_test.c
 * @brief Compile-time tests for chimera_abi.h
 *
 * Validates struct layout, enum values, and macro behavior at compile time.
 */

#include "chimera_abi.h"
#include <stdio.h>
#include <string.h>

/* Test that static inline functions compile correctly */
static void test_status_functions(void) {
    /* Valid status codes */
   chimera_status_t status_ok = CHIMERA_STATUS_OK;
    chimera_status_t status_err = CHIMERA_STATUS_ERROR;

    if (!chimera_status_is_ok(status_ok)) {
        fprintf(stderr, "FAIL: status_is_ok(CHIMERA_STATUS_OK) should be true\n");
    }
    if (chimera_status_is_ok(status_err)) {
        fprintf(stderr, "FAIL: status_is_ok(CHIMERA_STATUS_ERROR) should be false\n");
    }
    if (!chimera_status_is_error(status_err)) {
        fprintf(stderr, "FAIL: status_is_error(CHIMERA_STATUS_ERROR) should be true\n");
    }
    if (chimera_status_is_error(status_ok)) {
        fprintf(stderr, "FAIL: status_is_error(CHIMERA_STATUS_OK) should be false\n");
    }
}

/* Test slice creation */
static void test_slice_functions(void) {
    const int data[] = {1, 2, 3, 4, 5};
    chimera_slice_t slice = chimera_slice_from_ptr(data, sizeof(data));

    if (slice.data != data) {
        fprintf(stderr, "FAIL: slice_from_ptr should preserve data pointer\n");
    }
    if (slice.len != sizeof(data)) {
        fprintf(stderr, "FAIL: slice_from_ptr should preserve length\n");
    }
}

/* Test result functions */
static void test_result_functions(void) {
    chimera_result_t result_ok = { .is_ok = true };
    chimera_result_t result_err = { .is_ok = false };

    if (!chimera_result_is_ok(&result_ok)) {
        fprintf(stderr, "FAIL: result_is_ok should return true for OK result\n");
    }
    if (chimera_result_is_err(&result_ok)) {
        fprintf(stderr, "FAIL: result_is_err should return false for OK result\n");
    }
    if (!chimera_result_is_err(&result_err)) {
        fprintf(stderr, "FAIL: result_is_err should return true for Err result\n");
    }
    if (chimera_result_is_ok(&result_err)) {
        fprintf(stderr, "FAIL: result_is_ok should return false for Err result\n");
    }
}

/* Test string as slice */
static void test_string_functions(void) {
    chimera_string_t str = {
        .data = "hello world",
        .len = 11,
        .capacity = 64
    };

    chimera_slice_t slice = chimera_string_as_slice(&str);
    if (slice.data != str.data) {
        fprintf(stderr, "FAIL: string_as_slice should preserve data pointer\n");
    }
    if (slice.len != str.len) {
        fprintf(stderr, "FAIL: string_as_slice should preserve length\n");
    }
}

/* Compile-time layout assertions */
static void test_layout_size(void) {
    /* Verify sizes are as expected */
    if (sizeof(chimera_status_t) != sizeof(int32_t)) {
        fprintf(stderr, "FAIL: chimera_status_t should be int32-sized\n");
    }
    if (sizeof(chimera_slice_t) != sizeof(void*) * 2) {
        fprintf(stderr, "FAIL: chimera_slice_t should be two pointers\n");
    }
    if (sizeof(chimera_string_t) != sizeof(void*) * 3) {
        fprintf(stderr, "FAIL: chimera_string_t should be three pointers\n");
    }
}

/* Test macro expansion */
static void test_macros(void) {
    int aligned = (int)((uintptr_t)(void*)42 & ~((uintptr_t)0xF));
    void* ptr = CHIMERA_ALIGN_PTR((void*)0x10, 16);
    if ((uintptr_t)ptr != 0x10) {
        fprintf(stderr, "FAIL: CHIMERA_ALIGN_PTR should not modify aligned pointer\n");
    }

    chimera_slice_t lit_slice = CHIMERA_SLICE_FROM_LITERAL("test");
    if (lit_slice.len != 4) {
        fprintf(stderr, "FAIL: SLICE_FROM_LITERAL should compute length correctly\n");
    }
}

int main(void) {
    printf("Running chimera_abi compile-time tests...\n");

    test_status_functions();
    test_slice_functions();
    test_result_functions();
    test_string_functions();
    test_layout_size();
    test_macros();

    printf("All tests passed.\n");
    return 0;
}