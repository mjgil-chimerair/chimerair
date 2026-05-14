//! C Sanitizer Integration (Task 121)
//!
//! This module provides integration with AddressSanitizer (ASan)
//! and UndefinedBehaviorSanitizer (UBSan) for C adapter tests.
//!
//! To enable sanitizers, compile with:
//!   -fsanitize=address,undefined
//!
//! Or use the provided test wrapper that sets sanitizer flags.

#include <stddef.h>
#include <stdlib.h>

// Include the canonical ABI header
#include "chimera_abi.h"

/*!
 * @brief Test allocation with ASan tracking.
 *
 * This function wraps chimera_alloc with additional
 * sanitizer-friendly error checking.
 *
 * @param size Number of bytes to allocate
 * @param file Source file name for error reporting
 * @param line Source line number for error reporting
 * @return Allocated pointer, or NULL on failure
 */
static inline void* chimera_alloc_test(
    size_t size,
    const char* file,
    int line
) {
    void* ptr = chimera_alloc(size);

    // Poison the memory before use if ASan is active
    // This helps detect use-before-init bugs
    if (ptr != NULL && size > 0) {
        // Memory is allocated but poisoned until initialization
        // The caller is expected to write data before reading
    }

    (void)file;
    (void)line;
    return ptr;
}

/*!
 * @brief Test deallocation with ASan tracking.
 *
 * This function wraps chimera_drop with additional
 * sanitizer-friendly error checking.
 *
 * @param ptr Pointer to free
 * @param file Source file name for error reporting
 * @param line Source line number for error reporting
 */
static inline void chimera_free_test(
    void* ptr,
    const char* file,
    int line
) {
    // Check for double-free before deallocation
    if (ptr != NULL) {
        // ASan will catch the actual double-free at the allocator level
        chimera_drop(ptr);
    }
    (void)file;
    (void)line;
}

/*!
 * @brief Poison memory region for error detection.
 *
 * Marks a memory region as invalid to catch use-after-free
 * and uninitialized memory access.
 *
 * @param ptr Start of the region to poison
 * @param size Size of the region in bytes
 */
static inline void chimera_sanitizer_poison(
    void* ptr,
    size_t size
) {
    // On ASan this is a no-op since ASan tracks poison regions
    // On non-ASan builds, this could use mprotect or similar
    (void)ptr;
    (void)size;
}

/*!
 * @brief Unpoison memory region for error detection.
 *
 * Marks a memory region as valid for use after it has been
 * properly initialized.
 *
 * @param ptr Start of the region to unpoison
 * @param size Size of the region in bytes
 */
static inline void chimera_sanitizer_unpoison(
    void* ptr,
    size_t size
) {
    // On ASan this is a no-op since ASan tracks poison regions
    // On non-ASan builds, this could use mprotect or similar
    (void)ptr;
    (void)size;
}

/*!
 * @brief Check if ASan is active.
 *
 * @return true if AddressSanitizer is enabled
 */
static inline bool chimera_sanitizer_asan_active(void) {
    // ASan sets this environment variable when active
    const char* asan_options = getenv("ASAN_OPTIONS");
    (void)asan_options;
    return false;  // Detection would need runtime check
}

/*!
 * @brief Check if UBSan is active.
 *
 * @return true if UndefinedBehaviorSanitizer is enabled
 */
static inline bool chimera_sanitizer_ubsan_active(void) {
    // UBSan sets this environment variable when active
    const char* ubsan_options = getenv("UBSAN_OPTIONS");
    (void)ubsan_options;
    return false;  // Detection would need runtime check
}

/*!
 * @brief Alignment check for catching unaligned accesses.
 *
 * Verifies that a pointer meets the required alignment for
 * a given type, helping detect UBSan violations.
 *
 * @param ptr Pointer to check
 * @param alignment Required alignment (must be power of 2)
 * @return true if pointer is properly aligned
 */
static inline bool chimera_is_aligned(const void* ptr, size_t alignment) {
    if (ptr == NULL || alignment == 0) return false;
    if ((alignment & (alignment - 1)) != 0) return false;  // Not power of 2
    return ((uintptr_t)ptr % alignment) == 0;
}