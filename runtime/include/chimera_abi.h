/*!
 * @file chimera_abi.h
 * @brief Chimera polyglot IR canonical ABI header
 *
 * Defines the canonical C interface for the Chimera ABI, including:
 * - Status codes and error handling
 * - Memory allocation callbacks
 * - Primitive type definitions
 * - Calling convention macros
 * - Slice and string types
 * - Result and error propagation
 */

#ifndef CHIMERA_ABI_H
#define CHIMERA_ABI_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*============================================================================
 * Version
 *============================================================================*/

#define CHIMERA_ABI_VERSION_MAJOR 0
#define CHIMERA_ABI_VERSION_MINOR 1
#define CHIMERA_ABI_VERSION_PATCH 0

#define CHIMERA_ABI_VERSION_STRING "0.1.0"

/*============================================================================
 * Target Triple
 *============================================================================*/

/*! @brief Target architecture identifiers */
typedef enum chimera_target_arch {
    CHIMERA_ARCH_X86_64,
    CHIMERA_ARCH_AARCH64,
    CHIMERA_ARCH_WASM32,
    CHIMERA_ARCH_WASM64,
} chimera_target_arch_t;

/*! @brief Target OS identifiers */
typedef enum chimera_target_os {
    CHIMERA_OS_LINUX,
    CHIMERA_OS_MACOS,
    CHIMERA_OS_WINDOWS,
    CHIMERA_OS_WASI,
} chimera_target_os_t;

/*! @brief Target environment identifiers */
typedef enum chimera_target_env {
    CHIMERA_ENV_GNU,
    CHIMERA_ENV_MUSL,
    CHIMERA_ENV_MSVCRT,
    CHIMERA_ENV_WASI,
} chimera_target_env_t;

/*============================================================================
 * Status Codes
 *============================================================================*/

/*! @brief Canonical status codes for Chimera ABI operations */
typedef enum chimera_status {
    CHIMERA_STATUS_OK              = 0,
    CHIMERA_STATUS_ERROR           = 1,
    CHIMERA_STATUS_INVALID_ARG     = 2,
    CHIMERA_STATUS_INVALID_STATE   = 3,
    CHIMERA_STATUS_NOT_FOUND       = 4,
    CHIMERA_STATUS_OUT_OF_MEMORY    = 5,
    CHIMERA_STATUS_BUFFER_TOO_SMALL = 6,
    CHIMERA_STATUS_TYPE_MISMATCH    = 7,
    CHIMERA_STATUS_BORROW_EXCLUSIVE = 8,
    CHIMERA_STATUS_USE_AFTER_MOVE  = 9,
    CHIMERA_STATUS_DOUBLE_FREE      = 10,
    CHIMERA_STATUS_PANIC            = 11,
} chimera_status_t;

/*! @brief Check if a status indicates success */
static inline bool chimera_status_is_ok(chimera_status_t status) {
    return status == CHIMERA_STATUS_OK;
}

/*! @brief Check if a status indicates an error */
static inline bool chimera_status_is_error(chimera_status_t status) {
    return status != CHIMERA_STATUS_OK;
}

/*============================================================================
 * Error Domain
 *============================================================================*/

/*! @brief Error domain for rich error categorization */
typedef enum chimera_error_domain {
    CHIMERA_DOMAIN_NONE,
    CHIMERA_DOMAIN_IO,          /* I/O errors */
    CHIMERA_DOMAIN_MEMORY,      /* Memory allocation/free errors */
    CHIMERA_DOMAIN_TYPE,        /* Type system errors */
    CHIMERA_DOMAIN_OWNERSHIP,   /* Ownership/borrow errors */
    CHIMERA_DOMAIN_VALIDATION, /* Validation errors */
    CHIMERA_DOMAIN_RUNTIME,     /* Runtime errors */
} chimera_error_domain_t;

/*============================================================================
 * Canonical ABI Structs
 *============================================================================*/

/*! @brief Canonical status type used by lowered Result bridges */
typedef int32_t ch_status;

/*! @brief Canonical rich error payload */
typedef struct ch_error {
    uint32_t domain;
    uint32_t code;
    uint32_t flags;
    const void* message_ptr;
    uint64_t message_len;
    void* payload_ptr;
    void (*payload_drop_fn)(void* ptr);
    void* payload_drop_ctx;
} ch_error_t;

/*! @brief Extended error information */
typedef struct chimera_error {
    chimera_status_t      status;
    chimera_error_domain_t domain;
    int32_t               code;
    const char*           message;
    const char*           file;
    int32_t               line;
} chimera_error_t;

/*============================================================================
 * Calling Conventions
 *============================================================================*/

/*! @brief Calling convention identifiers */
typedef enum chimera_cconv {
    CHIMERA_CCONV_C       = 0,  /* C calling convention */
    CHIMERA_CCONV_SYSV    = 1,  /* System V (Linux/macOS x86_64) */
    CHIMERA_CCONV_WIN64    = 2,  /* Windows x64 */
    CHIMERA_CCONV_WASM    = 3,  /* WebAssembly */
    CHIMERA_CCONV_CHIMERA  = 4,  /* Chimera ABI */
} chimera_cconv_t;

/*! @brief Calling convention macros for function declarations */
#define CHIMERA_EXPORT __attribute__((visibility("default")))
#define CHIMERA_IMPORT __attribute__((visibility("default")))
#define CHIMERA_CALLSYS __attribute__((sysv_abi))
#define CHIMERA_CALLWIN64 __attribute__((ms_abi))

/*============================================================================
 * Ownership and Lifetime
 *============================================================================*/

/*! @brief Ownership kind for borrowed/owned values */
typedef enum chimera_ownership {
    CHIMERA_OWNERSHIP_BORROWED,     /* Borrowed reference */
    CHIMERA_OWNERSHIP_BORROWED_MUT,  /* Mutable borrowed reference */
    CHIMERA_OWNERSHIP_OWNED,        /* Owned value */
    CHIMERA_OWNERSHIP_RAW,          /* Raw pointer, no ownership tracking */
} chimera_ownership_t;

/*! @brief Lifetime kind for borrowed values */
typedef enum chimera_lifetime {
    CHIMERA_LIFETIME_CALL,    /* Lifetime ends at call boundary */
    CHIMERA_LIFETIME_STATIC, /* Lifetime is static/global */
    CHIMERA_LIFETIME_OWNER,  /* Owned value with dynamic lifetime */
} chimera_lifetime_t;

/*============================================================================
 * Allocator Interface
 *============================================================================*/

/*! @brief Allocator allocation kind */
typedef enum chimera_alloc_kind {
    CHIMERA_ALLOC_NEW,        /* Allocate new memory */
    CHIMERA_ALLOC_MALLOC,     /* Standard malloc semantics */
    CHIMERA_ALLOC_RESIZE,      /* Resize existing allocation */
    CHIMERA_ALLOC_FREE,        /* Free memory */
} chimera_alloc_kind_t;

/*! @brief Allocator callback function type */
typedef void* (*chimera_alloc_fn)(
    void*       user_data,
    chimera_alloc_kind_t kind,
    void*       ptr,
    size_t      old_size,
    size_t      new_size
);

/*! @brief Canonical allocator descriptor */
typedef struct ch_allocator {
    uint64_t id;
    uint32_t kind;
    void* ptr;
} ch_allocator_t;

/*! @brief Allocator configuration */
typedef struct chimera_allocator {
    chimera_alloc_fn  alloc;
    void*            user_data;
    size_t           header_size;
} chimera_allocator_t;

/*! @brief Default system allocator */
extern chimera_allocator_t chimera_default_allocator;

/*============================================================================
 * Allocator Helpers
 *============================================================================*/

/*!
 * @brief Register a custom allocator for use by Chimera runtime
 * @param allocator Allocator to register (NULL resets to default)
 * @return CHIMERA_STATUS_OK on success
 */
CHIMERA_EXPORT chimera_status_t chimera_allocator_register(
    chimera_allocator_t* allocator
);

/*!
 * @brief Get the currently registered allocator
 * @return Pointer to current allocator
 */
CHIMERA_EXPORT chimera_allocator_t* chimera_allocator_get_current(void);

/*!
 * @brief Allocate memory using the current allocator
 * @param size Size to allocate
 * @return Pointer to allocated memory or NULL on failure
 */
CHIMERA_EXPORT void* chimera_alloc(size_t size);

/*!
 * @brief Free memory using the current allocator
 * @param ptr Pointer to free
 * @param size Original allocation size
 */
CHIMERA_EXPORT void chimera_dealloc(void* ptr, size_t size);

/*============================================================================
 * Drop Helpers
 *============================================================================*/

/*!
 * @brief Drop callback for opaque payloads
 * @param ptr Pointer to the payload
 * @param size Size of the payload
 * @param drop_fn Optional drop function (if NULL, uses dealloc)
 */
typedef void (*chimera_drop_fn_t)(void* ptr, size_t size);

/*!
 * @brief Register a drop callback for an owned payload
 * @param ptr Pointer to the payload
 * @param size Size of the payload
 * @param drop_fn Drop function to call (NULL = default dealloc)
 */
CHIMERA_EXPORT void chimera_register_drop(
    void* ptr,
    size_t size,
    chimera_drop_fn_t drop_fn
);

/*!
 * @brief Drop a registered payload and clean up
 * @param ptr Pointer to the payload
 */
CHIMERA_EXPORT void chimera_drop(void* ptr);

/*!
 * @brief Owned-byte cleanup helper
 * @param ptr Pointer to owned bytes
 * @param len Length of owned bytes
 * @note Frees the memory via current allocator
 */
CHIMERA_EXPORT void chimera_drop_bytes(void* ptr, size_t len);

/*============================================================================
 * Slice Types
 *============================================================================*/

/*! @brief Canonical byte slice layout */
typedef struct ch_slice {
    const void* ptr;
    uint64_t    len;
} ch_slice_t;

/*! @brief Chimera slice for borrowed sequences */
typedef struct chimera_slice {
    const void*  data;   /* Pointer to slice data */
    size_t       len;    /* Length in bytes */
} chimera_slice_t;

/*! @brief Mutable slice */
typedef struct chimera_slice_mut {
    void*  data;
    size_t len;
} chimera_slice_mut_t;

/*! @brief Create a slice from a pointer and length */
static inline chimera_slice_t chimera_slice_from_ptr(const void* data, size_t len) {
    chimera_slice_t slice = { data, len };
    return slice;
}

/*============================================================================
 * String Types
 *============================================================================*/

/*! @brief Canonical borrowed UTF-8 string layout */
typedef struct ch_borrow_str {
    const uint8_t* ptr;
    uint64_t       len;
    uint32_t       lifetime;
} ch_borrow_str_t;

/*! @brief Canonical owned byte buffer layout */
typedef struct ch_owned_bytes {
    uint8_t* ptr;
    uint64_t len;
    uint64_t capacity;
    uint64_t allocator_id;
} ch_owned_bytes_t;

/*! @brief Canonical opaque owned handle layout */
typedef struct ch_handle {
    void* ptr;
    void (*drop_fn)(void* ptr);
    uint64_t size;
} ch_handle_t;

/*! @brief Chimera string (always UTF-8, null-terminated) */
typedef struct chimera_string {
    const char*  data;   /* Pointer to UTF-8 data */
    size_t       len;    /* Length NOT including null terminator */
    size_t       capacity;
} chimera_string_t;

/*! @brief Mutable string */
typedef struct chimera_string_mut {
    char*   data;
    size_t  len;
    size_t  capacity;
} chimera_string_mut_t;

/*! @brief Create a string slice from a C string */
static inline chimera_slice_t chimera_string_as_slice(const chimera_string_t* str) {
    chimera_slice_t slice = { str->data, str->len };
    return slice;
}

/*============================================================================
 * Result Type
 *============================================================================*/

/*! @brief Result T, E - Ok variant */
typedef struct chimera_result_ok {
    void*  value_ptr;
    size_t value_size;
} chimera_result_ok_t;

/*! @brief Result T, E - Err variant */
typedef struct chimera_result_err {
    void*  error_ptr;
    size_t error_size;
    int32_t error_code;
} chimera_result_err_t;

/*! @brief Result storage (discriminated union) */
typedef struct chimera_result {
    bool                is_ok;
    union {
        chimera_result_ok_t ok;
        chimera_result_err_t err;
    } data;
} chimera_result_t;

/*! @brief Check if result is OK */
static inline bool chimera_result_is_ok(const chimera_result_t* result) {
    return result->is_ok;
}

/*! @brief Check if result is Err */
static inline bool chimera_result_is_err(const chimera_result_t* result) {
    return !result->is_ok;
}

/*============================================================================
 * Panic Handling
 *============================================================================*/

/*! @brief Panic behavior on panic boundary */
typedef enum chimera_panic_policy {
    CHIMERA_PANIC_ABORT,    /* Abort immediately */
    CHIMERA_PANIC_UNWIND,   /* Unwind stack if possible */
    CHIMERA_PANIC_RUST,     /* Use Rust panic handler */
} chimera_panic_policy_t;

/*! @brief Panic info passed across boundary */
typedef struct chimera_panic_info {
    const char* message;
    size_t      message_len;
    const char* file;
    int32_t     line;
    const char* reason;
} chimera_panic_info_t;

/*! @brief Panic callback type */
typedef void (*chimera_panic_fn)(const chimera_panic_info_t* info);

/*! @brief Register a panic handler */
CHIMERA_EXPORT void chimera_set_panic_handler(chimera_panic_fn handler);

/*============================================================================
 * Function Pointer Types
 *============================================================================*/

/*! @brief Opaque function handle */
typedef void (*chimera_fn_ptr_t)(void);

typedef struct chimera_import {
    const char* name;
    chimera_fn_ptr_t ptr;
    chimera_cconv_t cconv;
} chimera_import_t;

/*============================================================================
 * Module Interface
 *============================================================================*/

/*! @brief Module handle */
typedef struct chimera_module* chimera_module_t;

/*! @brief Module creation options */
typedef struct chimera_module_options {
    chimera_target_arch_t    target_arch;
    chimera_target_os_t      target_os;
    chimera_target_env_t     target_env;
    chimera_allocator_t*     allocator;
    chimera_panic_policy_t   panic_policy;
} chimera_module_options_t;

/*! @brief Get a function pointer from a module */
CHIMERA_EXPORT chimera_fn_ptr_t chimera_module_get_fn(
    chimera_module_t module,
    const char* name
);

/*! @brief Get a function pointer with calling convention check */
CHIMERA_EXPORT chimera_fn_ptr_t chimera_module_get_fn_with_cconv(
    chimera_module_t module,
    const char* name,
    chimera_cconv_t expected_cconv
);

/*============================================================================
 * Utility Macros
 *============================================================================*/

/*! @brief Result type builder helpers */
#define CHIMERA_OK(result_ptr) ((result_ptr)->is_ok = true)
#define CHIMERA_ERR(result_ptr) ((result_ptr)->is_ok = false)

/*! @brief String literal to slice */
#define CHIMERA_SLICE_FROM_LITERAL(lit) \
    { lit, sizeof(lit) - 1 }

/*! @brief Align pointer to given alignment */
#define CHIMERA_ALIGN_PTR(ptr, align) \
    ((void*)(((uintptr_t)(ptr) + ((align) - 1)) & ~((align) - 1)))

#ifdef __cplusplus
}
#endif

#endif /* CHIMERA_ABI_H */
