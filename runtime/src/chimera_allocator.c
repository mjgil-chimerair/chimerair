/*!
 * @file chimera_allocator.c
 * @brief Chimera allocator ABI implementation
 *
 * Implements the allocator callback interface for the Chimera ABI runtime.
 */

#include <chimera_abi.h>
#include <stdlib.h>
#include <string.h>

/*============================================================================
 * Default System Allocator
 *============================================================================*/

static void* default_alloc(
    void* user_data,
    chimera_alloc_kind_t kind,
    void* ptr,
    size_t old_size,
    size_t new_size
) {
    (void)user_data;

    switch (kind) {
    case CHIMERA_ALLOC_NEW:
    case CHIMERA_ALLOC_MALLOC:
        if (new_size == 0) {
            return NULL;
        }
        return malloc(new_size);

    case CHIMERA_ALLOC_RESIZE:
        if (ptr == NULL) {
            return malloc(new_size);
        }
        if (new_size == 0) {
            free(ptr);
            return NULL;
        }
        return realloc(ptr, new_size);

    case CHIMERA_ALLOC_FREE:
        if (ptr != NULL) {
            free(ptr);
        }
        return NULL;

    default:
        return NULL;
    }
}

/*============================================================================
 * Global Allocator State
 *============================================================================*/

static chimera_allocator_t g_current_allocator = {
    .alloc = default_alloc,
    .user_data = NULL,
    .header_size = 0,
};

static chimera_allocator_t* g_default_allocator = &g_current_allocator;

/*============================================================================
 * Allocator Registration
 *============================================================================*/

CHIMERA_EXPORT chimera_status_t chimera_allocator_register(
    chimera_allocator_t* allocator
) {
    if (allocator == NULL) {
        g_current_allocator = *g_default_allocator;
        return CHIMERA_STATUS_OK;
    }

    if (allocator->alloc == NULL) {
        return CHIMERA_STATUS_INVALID_ARG;
    }

    g_current_allocator = *allocator;
    return CHIMERA_STATUS_OK;
}

CHIMERA_EXPORT chimera_allocator_t* chimera_allocator_get_current(void) {
    return &g_current_allocator;
}

/*============================================================================
 * Memory Allocation Helpers
 *============================================================================*/

CHIMERA_EXPORT void* chimera_alloc(size_t size) {
    return g_current_allocator.alloc(
        g_current_allocator.user_data,
        CHIMERA_ALLOC_MALLOC,
        NULL,
        0,
        size
    );
}

CHIMERA_EXPORT void chimera_dealloc(void* ptr, size_t size) {
    (void)size;
    g_current_allocator.alloc(
        g_current_allocator.user_data,
        CHIMERA_ALLOC_FREE,
        ptr,
        size,
        0
    );
}

/*============================================================================
 * Resize Helper
 *============================================================================*/

CHIMERA_EXPORT void* chimera_realloc(void* ptr, size_t old_size, size_t new_size) {
    return g_current_allocator.alloc(
        g_current_allocator.user_data,
        CHIMERA_ALLOC_RESIZE,
        ptr,
        old_size,
        new_size
    );
}

/*============================================================================
 * Drop Helpers
 *============================================================================*/

/* Drop callback registry - simple fixed-size array */
#define MAX_DROP_CALLBACKS 128

typedef struct {
    void* ptr;
    size_t size;
    chimera_drop_fn_t drop_fn;
} drop_entry_t;

static drop_entry_t g_drop_registry[MAX_DROP_CALLBACKS];
static size_t g_drop_count = 0;

CHIMERA_EXPORT void chimera_register_drop(
    void* ptr,
    size_t size,
    chimera_drop_fn_t drop_fn
) {
    if (ptr == NULL || g_drop_count >= MAX_DROP_CALLBACKS) {
        return;
    }

    g_drop_registry[g_drop_count].ptr = ptr;
    g_drop_registry[g_drop_count].size = size;
    g_drop_registry[g_drop_count].drop_fn = drop_fn;
    g_drop_count++;
}

CHIMERA_EXPORT void chimera_drop(void* ptr) {
    for (size_t i = 0; i < g_drop_count; i++) {
        if (g_drop_registry[i].ptr == ptr) {
            if (g_drop_registry[i].drop_fn != NULL) {
                g_drop_registry[i].drop_fn(ptr, g_drop_registry[i].size);
            } else {
                chimera_dealloc(ptr, g_drop_registry[i].size);
            }
            /* Remove from registry by swapping with last */
            if (i < g_drop_count - 1) {
                g_drop_registry[i] = g_drop_registry[g_drop_count - 1];
            }
            g_drop_count--;
            return;
        }
    }
}

CHIMERA_EXPORT void chimera_drop_bytes(void* ptr, size_t len) {
    if (ptr != NULL) {
        chimera_dealloc(ptr, len);
    }
}

/*============================================================================
 * Tests
 *============================================================================*/

#ifdef CHIMERA_ALLOCATOR_TEST

#include <assert.h>
#include <stdio.h>

static void test_allocator_register(void) {
    chimera_allocator_t alloc = {
        .alloc = default_alloc,
        .user_data = NULL,
        .header_size = 0,
    };

    chimera_status_t status = chimera_allocator_register(&alloc);
    assert(status == CHIMERA_STATUS_OK);

    chimera_allocator_t* current = chimera_allocator_get_current();
    assert(current->alloc == default_alloc);
}

static void test_alloc_reset(void) {
    chimera_status_t status = chimera_allocator_register(NULL);
    assert(status == CHIMERA_STATUS_OK);

    chimera_allocator_t* current = chimera_allocator_get_current();
    assert(current->alloc == default_alloc);
}

static void test_alloc_dealloc(void) {
    void* ptr = chimera_alloc(100);
    assert(ptr != NULL);

    chimera_dealloc(ptr, 100);
}

static void test_realloc(void) {
    void* ptr = chimera_alloc(100);
    assert(ptr != NULL);

    void* new_ptr = chimera_realloc(ptr, 100, 200);
    assert(new_ptr != NULL);

    chimera_dealloc(new_ptr, 200);
}

static void test_drop_bytes(void) {
    void* ptr = chimera_alloc(50);
    assert(ptr != NULL);

    chimera_drop_bytes(ptr, 50);
}

int main(void) {
    printf("Running allocator tests...\n");

    test_allocator_register();
    printf("  test_allocator_register: PASSED\n");

    test_alloc_reset();
    printf("  test_alloc_reset: PASSED\n");

    test_alloc_dealloc();
    printf("  test_alloc_dealloc: PASSED\n");

    test_realloc();
    printf("  test_realloc: PASSED\n");

    test_drop_bytes();
    printf("  test_drop_bytes: PASSED\n");

    printf("All allocator tests PASSED\n");
    return 0;
}

#endif /* CHIMERA_ALLOCATOR_TEST */
