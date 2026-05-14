//! C Allocator/Drop Helper APIs (Task 120)
//!
//! This module provides canonical helper hooks for allocator IDs,
//! owned bytes, handles, and drop trampolines as defined in the
//! Chimera ABI specification.

#include <stddef.h>
#include <stdint.h>

// Include the canonical ABI header
#include "chimera_abi.h"

/*!
 * @brief Opaque handle for owned memory.
 *
 * This structure represents a Chimera-owned memory handle
 * that can be passed across the ABI boundary.
 */
struct chimera_owned_handle {
    void* data;          // Pointer to allocated data
    size_t size;         // Size of the allocated data
    uint32_t allocator_id;  // ID of the allocator used
};

/*!
 * @brief Create an owned memory handle.
 *
 * Allocates memory using the registered Chimera allocator
 * and wraps it in an opaque handle for safe crossing of
 * the ABI boundary.
 *
 * @param size Number of bytes to allocate
 * @param allocator_id The allocator to use (0 = default)
 * @return A new owned handle, or a handle with NULL data on failure
 */
static inline struct chimera_owned_handle chimera_owned_create(
    size_t size,
    uint32_t allocator_id
) {
    struct chimera_owned_handle handle = { NULL, 0, allocator_id };

    void* data = chimera_alloc(size);
    if (data != NULL) {
        handle.data = data;
        handle.size = size;
    }

    return handle;
}

/*!
 * @brief Destroy an owned memory handle.
 *
 * Frees the memory associated with the handle using the
 * corresponding allocator. After this call, the handle
 * should not be used.
 *
 * @param handle The owned handle to destroy
 */
static inline void chimera_owned_destroy(struct chimera_owned_handle handle) {
    if (handle.data != NULL && handle.allocator_id == 0) {
        chimera_drop(handle.data);
    }
    handle.data = NULL;
    handle.size = 0;
}

/*!
 * @brief Get the data pointer from an owned handle.
 *
 * @param handle The owned handle
 * @return The data pointer, or NULL if the handle is invalid
 */
static inline void* chimera_owned_data(struct chimera_owned_handle handle) {
    return handle.data;
}

/*!
 * @brief Get the size from an owned handle.
 *
 * @param handle The owned handle
 * @return The size of the allocated data
 */
static inline size_t chimera_owned_size(struct chimera_owned_handle handle) {
    return handle.size;
}

/*!
 * @brief Drop trampoline for foreign callers.
 *
 * This function is called when a foreign language runtime
 * needs to release Chimera-owned memory. It provides a
 * safe跨 language boundary for deallocation.
 *
 * @param ptr The pointer to free (must have been allocated by Chimera)
 */
static inline void chimera_drop_owned(void* ptr) {
    if (ptr != NULL) {
        chimera_drop(ptr);
    }
}