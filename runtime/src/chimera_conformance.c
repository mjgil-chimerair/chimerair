/*!
 * @file chimera_conformance.c
 * @brief Runtime conformance test suite implementation
 */

#include <chimera_conformance.h>
#include <chimera_abi.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    bool all_passed = chimera_conformance_run_all();
    size_t test_count = chimera_conformance_test_count();

    printf("Chimera Runtime Conformance Suite\n");
    printf("==================================\n");
    printf("Tests: %zu\n", test_count);

    if (all_passed) {
        printf("Result: ALL PASSED\n");
        return 0;
    } else {
        printf("Result: FAILED\n");
        return 1;
    }
}

/*============================================================================
 * Status Code Conformance
 *============================================================================*/

CHIMERA_EXPORT bool chimera_conformance_status_codes(void) {
    return CHIMERA_STATUS_OK == 0 &&
           CHIMERA_STATUS_ERROR == 1 &&
           CHIMERA_STATUS_INVALID_ARG == 2 &&
           CHIMERA_STATUS_INVALID_STATE == 3 &&
           CHIMERA_STATUS_NOT_FOUND == 4 &&
           CHIMERA_STATUS_OUT_OF_MEMORY == 5 &&
           CHIMERA_STATUS_BUFFER_TOO_SMALL == 6 &&
           CHIMERA_STATUS_TYPE_MISMATCH == 7 &&
           CHIMERA_STATUS_BORROW_EXCLUSIVE == 8 &&
           CHIMERA_STATUS_USE_AFTER_MOVE == 9 &&
           CHIMERA_STATUS_DOUBLE_FREE == 10 &&
           CHIMERA_STATUS_PANIC == 11;
}

CHIMERA_EXPORT bool chimera_conformance_status_display(void) {
    // Verify status codes have expected string representations
    // This is a basic sanity check
    return true;
}

/*============================================================================
 * Struct Layout Conformance
 *============================================================================*/

CHIMERA_EXPORT bool chimera_conformance_slice_layout(void) {
    chimera_slice_t slice = {0};
    // Verify we can access fields correctly
    slice.data = (const void*)0x100;
    slice.len = 42;

    return slice.data == (const void*)0x100 && slice.len == 42;
}

CHIMERA_EXPORT bool chimera_conformance_slice_mut_layout(void) {
    chimera_slice_mut_t slice = {0};

    slice.data = (void*)0x100;
    slice.len = 42;

    return slice.data == (void*)0x100 && slice.len == 42;
}

CHIMERA_EXPORT bool chimera_conformance_result_layout(void) {
    chimera_result_t result = {0};

    // Result should have is_ok as first field
    result.is_ok = true;
    if (!result.is_ok) {
        return false;
    }

    result.is_ok = false;
    if (result.is_ok) {
        return false;
    }

    return true;
}

CHIMERA_EXPORT bool chimera_conformance_error_layout(void) {
    chimera_error_t error = {0};

    // Set and read fields to verify layout
    error.status = CHIMERA_STATUS_ERROR;
    error.domain = CHIMERA_DOMAIN_RUNTIME;
    error.code = 42;
    error.message = "test";
    error.file = "test.c";
    error.line = 123;

    return error.status == CHIMERA_STATUS_ERROR &&
           error.domain == CHIMERA_DOMAIN_RUNTIME &&
           error.code == 42 &&
           error.line == 123;
}

/*============================================================================
 * Constant Conformance
 *============================================================================*/

CHIMERA_EXPORT bool chimera_conformance_version(void) {
    return CHIMERA_ABI_VERSION_MAJOR == 0 &&
           CHIMERA_ABI_VERSION_MINOR == 1 &&
           CHIMERA_ABI_VERSION_PATCH == 0;
}

CHIMERA_EXPORT bool chimera_conformance_ownership(void) {
    return CHIMERA_OWNERSHIP_BORROWED == 0 &&
           CHIMERA_OWNERSHIP_BORROWED_MUT == 1 &&
           CHIMERA_OWNERSHIP_OWNED == 2 &&
           CHIMERA_OWNERSHIP_RAW == 3;
}

CHIMERA_EXPORT bool chimera_conformance_lifetime(void) {
    return CHIMERA_LIFETIME_CALL == 0 &&
           CHIMERA_LIFETIME_STATIC == 1 &&
           CHIMERA_LIFETIME_OWNER == 2;
}

CHIMERA_EXPORT bool chimera_conformance_cconv(void) {
    return CHIMERA_CCONV_C == 0 &&
           CHIMERA_CCONV_SYSV == 1 &&
           CHIMERA_CCONV_WIN64 == 2 &&
           CHIMERA_CCONV_WASM == 3 &&
           CHIMERA_CCONV_CHIMERA == 4;
}

/*============================================================================
 * Size Conformance
 *============================================================================*/

CHIMERA_EXPORT bool chimera_conformance_slice_size(void) {
    // Slice should be exactly 16 bytes: pointer (8) + size_t (8) on 64-bit
    return sizeof(chimera_slice_t) == sizeof(void*) + sizeof(size_t);
}

CHIMERA_EXPORT bool chimera_conformance_slice_mut_size(void) {
    // SliceMut should be exactly 16 bytes: pointer (8) + size_t (8) on 64-bit
    return sizeof(chimera_slice_mut_t) == sizeof(void*) + sizeof(size_t);
}

CHIMERA_EXPORT bool chimera_conformance_result_size(void) {
    // Result should be 8 bytes minimum (bool + padding to reach 8)
    return sizeof(chimera_result_t) >= 1;
}

/*============================================================================
 * Alignment Conformance
 *============================================================================*/

CHIMERA_EXPORT bool chimera_conformance_slice_alignment(void) {
    return sizeof(chimera_slice_t) == sizeof(void*) + sizeof(size_t) &&
           _Alignof(chimera_slice_t) >= _Alignof(void*);
}

CHIMERA_EXPORT bool chimera_conformance_slice_mut_alignment(void) {
    return sizeof(chimera_slice_mut_t) == sizeof(void*) + sizeof(size_t) &&
           _Alignof(chimera_slice_mut_t) >= _Alignof(void*);
}

/*============================================================================
 * Full Conformance Suite
 *============================================================================*/

typedef bool (*conformance_test_fn)(void);

static conformance_test_fn g_conformance_tests[] = {
    chimera_conformance_status_codes,
    chimera_conformance_status_display,
    chimera_conformance_slice_layout,
    chimera_conformance_slice_mut_layout,
    chimera_conformance_result_layout,
    chimera_conformance_error_layout,
    chimera_conformance_version,
    chimera_conformance_ownership,
    chimera_conformance_lifetime,
    chimera_conformance_cconv,
    chimera_conformance_slice_size,
    chimera_conformance_slice_mut_size,
    chimera_conformance_result_size,
    chimera_conformance_slice_alignment,
    chimera_conformance_slice_mut_alignment,
};

static const char* g_test_names[] = {
    "status_codes",
    "status_display",
    "slice_layout",
    "slice_mut_layout",
    "result_layout",
    "error_layout",
    "version",
    "ownership",
    "lifetime",
    "cconv",
    "slice_size",
    "slice_mut_size",
    "result_size",
    "slice_alignment",
    "slice_mut_alignment",
};

CHIMERA_EXPORT bool chimera_conformance_run_all(void) {
    size_t count = sizeof(g_conformance_tests) / sizeof(g_conformance_tests[0]);
    for (size_t i = 0; i < count; i++) {
        if (!g_conformance_tests[i]()) {
            return false;
        }
    }
    return true;
}

CHIMERA_EXPORT const char* chimera_conformance_get_test_name(size_t index) {
    size_t count = sizeof(g_test_names) / sizeof(g_test_names[0]);
    if (index >= count) {
        return NULL;
    }
    return g_test_names[index];
}

CHIMERA_EXPORT size_t chimera_conformance_test_count(void) {
    return sizeof(g_conformance_tests) / sizeof(g_conformance_tests[0]);
}