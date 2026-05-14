/*!
 * @file chimera_error.c
 * @brief Chimera error ABI helpers implementation
 *
 * Implements error constructors, domain mapping, message ownership,
 * and payload drop for the Chimera ABI runtime.
 */

#include <chimera_abi.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

/*============================================================================
 * Error Domain Names
 *============================================================================*/

CHIMERA_EXPORT const char* chimera_error_domain_name(chimera_error_domain_t domain) {
    switch (domain) {
    case CHIMERA_DOMAIN_NONE:
        return "none";
    case CHIMERA_DOMAIN_IO:
        return "io";
    case CHIMERA_DOMAIN_MEMORY:
        return "memory";
    case CHIMERA_DOMAIN_TYPE:
        return "type";
    case CHIMERA_DOMAIN_OWNERSHIP:
        return "ownership";
    case CHIMERA_DOMAIN_VALIDATION:
        return "validation";
    case CHIMERA_DOMAIN_RUNTIME:
        return "runtime";
    default:
        return "unknown";
    }
}

/*============================================================================
 * Error Message Registry
 *============================================================================*/

#define MAX_ERROR_MESSAGES 64

typedef struct {
    char* message;
    size_t len;
} error_message_t;

static error_message_t g_error_messages[MAX_ERROR_MESSAGES];
static size_t g_error_message_count = 0;

/*============================================================================
 * Error Creation and Destruction
 *============================================================================*/

CHIMERA_EXPORT chimera_error_t* chimera_error_create(
    chimera_status_t status,
    chimera_error_domain_t domain,
    int32_t code,
    const char* message
) {
    if (g_error_message_count >= MAX_ERROR_MESSAGES) {
        return NULL;
    }

    chimera_error_t* error = (chimera_error_t*)malloc(sizeof(chimera_error_t));
    if (error == NULL) {
        return NULL;
    }

    error->status = status;
    error->domain = domain;
    error->code = code;
    error->line = 0;
    error->file = NULL;

    if (message != NULL) {
        size_t msg_len = strlen(message);
        char* stored_msg = (char*)malloc(msg_len + 1);
        if (stored_msg == NULL) {
            free(error);
            return NULL;
        }
        memcpy(stored_msg, message, msg_len + 1);
        error->message = stored_msg;

        g_error_messages[g_error_message_count].message = stored_msg;
        g_error_messages[g_error_message_count].len = msg_len;
        g_error_message_count++;
    } else {
        error->message = NULL;
    }

    return error;
}

CHIMERA_EXPORT void chimera_error_destroy(chimera_error_t* error) {
    if (error == NULL) {
        return;
    }

    /* Free message if present */
    if (error->message != NULL) {
        /* Find and remove from registry */
        for (size_t i = 0; i < g_error_message_count; i++) {
            if (g_error_messages[i].message == error->message) {
                /* Remove by swapping with last */
                if (i < g_error_message_count - 1) {
                    g_error_messages[i] = g_error_messages[g_error_message_count - 1];
                }
                g_error_message_count--;
                break;
            }
        }
        free((char*)error->message);
    }

    free(error);
}

CHIMERA_EXPORT void chimera_error_set_location(
    chimera_error_t* error,
    const char* file,
    int32_t line
) {
    if (error == NULL) {
        return;
    }
    error->file = file;
    error->line = line;
}

/*============================================================================
 * Error Domain Mapping
 *============================================================================*/

CHIMERA_EXPORT chimera_error_domain_t chimera_status_to_domain(chimera_status_t status) {
    switch (status) {
    case CHIMERA_STATUS_OK:
        return CHIMERA_DOMAIN_NONE;
    case CHIMERA_STATUS_ERROR:
        return CHIMERA_DOMAIN_RUNTIME;
    case CHIMERA_STATUS_INVALID_ARG:
        return CHIMERA_DOMAIN_VALIDATION;
    case CHIMERA_STATUS_INVALID_STATE:
        return CHIMERA_DOMAIN_RUNTIME;
    case CHIMERA_STATUS_NOT_FOUND:
        return CHIMERA_DOMAIN_IO;
    case CHIMERA_STATUS_OUT_OF_MEMORY:
        return CHIMERA_DOMAIN_MEMORY;
    case CHIMERA_STATUS_BUFFER_TOO_SMALL:
        return CHIMERA_DOMAIN_IO;
    case CHIMERA_STATUS_TYPE_MISMATCH:
        return CHIMERA_DOMAIN_TYPE;
    case CHIMERA_STATUS_BORROW_EXCLUSIVE:
        return CHIMERA_DOMAIN_OWNERSHIP;
    case CHIMERA_STATUS_USE_AFTER_MOVE:
        return CHIMERA_DOMAIN_OWNERSHIP;
    case CHIMERA_STATUS_DOUBLE_FREE:
        return CHIMERA_DOMAIN_MEMORY;
    case CHIMERA_STATUS_PANIC:
        return CHIMERA_DOMAIN_RUNTIME;
    default:
        return CHIMERA_DOMAIN_RUNTIME;
    }
}

/*============================================================================
 * Convenience Error Helpers
 *============================================================================*/

CHIMERA_EXPORT chimera_error_t* chimera_error_from_errno(int errno_value) {
    return chimera_error_create(
        CHIMERA_STATUS_ERROR,
        CHIMERA_DOMAIN_IO,
        errno_value,
        strerror(errno_value)
    );
}

CHIMERA_EXPORT chimera_error_t* chimera_error_bad_alloc(void) {
    return chimera_error_create(
        CHIMERA_STATUS_OUT_OF_MEMORY,
        CHIMERA_DOMAIN_MEMORY,
        0,
        "out of memory"
    );
}

CHIMERA_EXPORT chimera_error_t* chimera_error_invalid_arg(const char* message) {
    return chimera_error_create(
        CHIMERA_STATUS_INVALID_ARG,
        CHIMERA_DOMAIN_VALIDATION,
        0,
        message
    );
}

/*============================================================================
 * Tests
 *============================================================================*/

#ifdef CHIMERA_ERROR_TEST

#include <assert.h>
#include <stdio.h>

static void test_error_domain_name(void) {
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_NONE), "none") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_IO), "io") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_MEMORY), "memory") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_TYPE), "type") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_OWNERSHIP), "ownership") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_VALIDATION), "validation") == 0);
    assert(strcmp(chimera_error_domain_name(CHIMERA_DOMAIN_RUNTIME), "runtime") == 0);
}

static void test_error_create_destroy(void) {
    chimera_error_t* error = chimera_error_create(
        CHIMERA_STATUS_ERROR,
        CHIMERA_DOMAIN_RUNTIME,
        42,
        "test error"
    );
    assert(error != NULL);
    assert(error->status == CHIMERA_STATUS_ERROR);
    assert(error->domain == CHIMERA_DOMAIN_RUNTIME);
    assert(error->code == 42);
    assert(error->message != NULL);
    assert(strcmp(error->message, "test error") == 0);

    chimera_error_destroy(error);
}

static void test_error_set_location(void) {
    chimera_error_t* error = chimera_error_create(
        CHIMERA_STATUS_INVALID_ARG,
        CHIMERA_DOMAIN_VALIDATION,
        0,
        "invalid argument"
    );
    assert(error != NULL);
    assert(error->file == NULL);
    assert(error->line == 0);

    chimera_error_set_location(error, "test.c", 42);
    assert(error->file == "test.c");
    assert(error->line == 42);

    chimera_error_destroy(error);
}

static void test_status_to_domain(void) {
    assert(chimera_status_to_domain(CHIMERA_STATUS_OK) == CHIMERA_DOMAIN_NONE);
    assert(chimera_status_to_domain(CHIMERA_STATUS_OUT_OF_MEMORY) == CHIMERA_DOMAIN_MEMORY);
    assert(chimera_status_to_domain(CHIMERA_STATUS_TYPE_MISMATCH) == CHIMERA_DOMAIN_TYPE);
    assert(chimera_status_to_domain(CHIMERA_STATUS_BORROW_EXCLUSIVE) == CHIMERA_DOMAIN_OWNERSHIP);
    assert(chimera_status_to_domain(CHIMERA_STATUS_DOUBLE_FREE) == CHIMERA_DOMAIN_MEMORY);
}

static void test_error_bad_alloc(void) {
    chimera_error_t* error = chimera_error_bad_alloc();
    assert(error != NULL);
    assert(error->status == CHIMERA_STATUS_OUT_OF_MEMORY);
    assert(error->domain == CHIMERA_DOMAIN_MEMORY);
    chimera_error_destroy(error);
}

static void test_error_invalid_arg(void) {
    chimera_error_t* error = chimera_error_invalid_arg("size must be positive");
    assert(error != NULL);
    assert(error->status == CHIMERA_STATUS_INVALID_ARG);
    assert(error->domain == CHIMERA_DOMAIN_VALIDATION);
    assert(error->message != NULL);
    assert(strcmp(error->message, "size must be positive") == 0);
    chimera_error_destroy(error);
}

int main(void) {
    printf("Running error ABI tests...\n");

    test_error_domain_name();
    printf("  test_error_domain_name: PASSED\n");

    test_error_create_destroy();
    printf("  test_error_create_destroy: PASSED\n");

    test_error_set_location();
    printf("  test_error_set_location: PASSED\n");

    test_status_to_domain();
    printf("  test_status_to_domain: PASSED\n");

    test_error_bad_alloc();
    printf("  test_error_bad_alloc: PASSED\n");

    test_error_invalid_arg();
    printf("  test_error_invalid_arg: PASSED\n");

    printf("All error ABI tests PASSED\n");
    return 0;
}

#endif /* CHIMERA_ERROR_TEST */
