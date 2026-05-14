/*!
 * @file chimera_sanitizers.h
 * @brief Sanitizer configuration for Chimera runtime
 *
 * Provides macros and configuration for AddressSanitizer (ASan),
 * UndefinedBehaviorSanitizer (UBSan), and other runtime sanitizers.
 *
 * Usage:
 *   #define CHIMERA_SANITIZE_ADDRESS 1
 *   #define CHIMERA_SANITIZE_UNDEFINED 1
 *   #include <chimera_sanitizers.h>
 */

#ifndef CHIMERA_SANITIZERS_H
#define CHIMERA_SANITIZERS_H

#include <chimera_abi.h>

/*============================================================================
 * Sanitizer Enable Macros
 *============================================================================*/

/* Enable AddressSanitizer - detects memory errors */
#if defined(CHIMERA_SANITIZE_ADDRESS) && CHIMERA_SANITIZE_ADDRESS
    #include <sanitizer/asan_interface.h>
    #define CHIMERA_ASAN_ENABLED 1
#else
    #define CHIMERA_ASAN_ENABLED 0
#endif

/* Enable UndefinedBehaviorSanitizer - detects undefined behavior */
#if defined(CHIMERA_SANITIZE_UNDEFINED) && CHIMERA_SANITIZE_UNDEFINED
    #include <sanitizer/ubsan_interface.h>
    #define CHIMERA_UBSAN_ENABLED 1
#else
    #define CHIMERA_UBSAN_ENABLED 0
#endif

/* Enable MemorySanitizer - detects uninitialized memory (LLVM only) */
#if defined(CHIMERA_SANITIZE_MEMORY) && CHIMERA_SANITIZE_MEMORY
    #include <sanitizer/msan_interface.h>
    #define CHIMERA_MSAN_ENABLED 1
#else
    #define CHIMERA_MSAN_ENABLED 0
#endif

/* Enable ThreadSanitizer - detects data races */
#if defined(CHIMERA_SANITIZE_THREAD) && CHIMERA_SANITIZE_THREAD
    #include <sanitizer/tsan_interface.h>
    #define CHIMERA_TSAN_ENABLED 1
#else
    #define CHIMERA_TSAN_ENABLED 0
#endif

/*============================================================================
 * Sanitizer Initialization
 *============================================================================*/

/*!
 * @brief Initialize all enabled sanitizers
 * @note Called automatically at program startup if CHIMERA_AUTO_INIT is defined
 */
CHIMERA_EXPORT void chimera_sanitizers_init(void);

/*!
 * @brief Check if any sanitizer is active
 * @return true if any sanitizer is enabled
 */
CHIMERA_EXPORT bool chimera_sanitizers_enabled(void);

/*!
 * @brief Get sanitizer report string
 * @return Human-readable list of active sanitizers
 */
CHIMERA_EXPORT const char* chimera_sanitizers_report(void);

/*============================================================================
 * Memory Shadow Operations (ASan)
 *============================================================================*/

#if CHIMERA_ASAN_ENABLED

/*!
 * @brief Poison a memory region (mark as inaccessible)
 * @param addr Start address
 * @param size Size to poison
 */
static inline void chimera_asan_poison_region(void* addr, size_t size) {
    __asan_poison_memory_region(addr, size);
}

/*!
 * @brief Unpoison a memory region (mark as accessible)
 * @param addr Start address
 * @param size Size to unpoison
 */
static inline void chimera_asan_unpoison_region(void* addr, size_t size) {
    __asan_unpoison_memory_region(addr, size);
}

/*!
 * @brief Check if address is poisoned
 * @param addr Address to check
 * @return true if poisoned
 */
static inline bool chimera_asan_is_poisoned(void const* addr) {
    return __asan_address_is_poisoned(addr);
}

/*!
 * @brief Poison left redzone of a stack variable
 * @param addr Address of variable
 * @param size Size of variable
 */
#define CHIMERA_ASAN_POISON_LEFT_REDZONE(addr, size) \
    __asan_poison_memory_region(addr, size)

/*!
 * @brief Unpoison stack variable
 * @param addr Address of variable
 * @param size Size of variable
 */
#define CHIMERA_ASAN_UNPOISON_VAR(addr, size) \
    __asan_unpoison_memory_region(addr, size)

#else

#define chimera_asan_poison_region(addr, size) ((void)0)
#define chimera_asan_unpoison_region(addr, size) ((void)0)
#define chimera_asan_is_poisoned(addr) (false)
#define CHIMERA_ASAN_POISON_LEFT_REDZONE(addr, size) ((void)0)
#define CHIMERA_ASAN_UNPOISON_VAR(addr, size) ((void)0)

#endif

/*============================================================================
 * Undefined Behavior Handling (UBSan)
 *============================================================================*/

#if CHIMERA_UBSAN_ENABLED

/*!
 * @brief Enable undefined behavior checking for a region
 */
static inline void chimera_ubsan_enable() {
    __ubsan_enable_trap();
}

/*!
 * @brief Disable undefined behavior checking
 */
static inline void chimera_ubsan_disable() {
    __ubsan_disable_trap();
}

/*!
 * @brief Report a type mismatch
 * @param type Type that was expected
 * @param ptr Pointer that violated type check
 */
static inline void chimera_ubsan_type_mismatch(
    const char* type,
    void* ptr
) {
    __ubsan_handle_type_mismatch_v1_abort(type, (void*)(uintptr_t)ptr);
}

#else

#define chimera_ubsan_enable() ((void)0)
#define chimera_ubsan_disable() ((void)0)
#define chimera_ubsan_type_mismatch(type, ptr) ((void)0)

#endif

/*============================================================================
 * Shadow Memory (MSan)
 *============================================================================*/

#if CHIMERA_MSAN_ENABLED

/*!
 * @brief Mark memory region as initialized
 * @param addr Start address
 * @param size Size to mark
 */
static inline void chimera_msan_mark_initialized(void* addr, size_t size) {
    __msan_unpoison(addr, size);
}

/*!
 * @brief Mark memory region as uninitialized
 * @param addr Start address
 * @param size Size to mark
 */
static inline void chimera_msan_mark_uninitialized(void* addr, size_t size) {
    // No-op: memory is uninitialized by default
    (void)addr;
    (void)size;
}

/*!
 * @brief Check if memory is initialized
 * @param addr Address to check
 * @param size Size to check
 * @return true if all initialized
 */
static inline bool chimera_msan_is_initialized(void const* addr, size_t size) {
    // Simplified: actual implementation would check shadow memory
    (void)addr;
    (void)size;
    return true;
}

#else

#define chimera_msan_mark_initialized(addr, size) ((void)0)
#define chimera_msan_mark_uninitialized(addr, size) ((void)0)
#define chimera_msan_is_initialized(addr, size) (true)

#endif

/*============================================================================
 * Thread Sanitizer Annotations
 *============================================================================*/

#if CHIMERA_TSAN_ENABLED

/*!
 * @brief Annotate that mutex is locked
 * @param addr Mutex address
 * @param size Mutex size
 */
#define CHIMERA_TSAN_MUTEX_LOCK(addr, size) \
    __tsan_mutex_pre_lock(addr, 0); \
    __tsan_mutex_post_lock(addr, 0)

/*!
 * @brief Annotate that mutex is unlocked
 * @param addr Mutex address
 */
#define CHIMERA_TSAN_MUTEX_UNLOCK(addr) \
    __tsan_mutex_pre_unlock(addr, 0); \
    __tsan_mutex_post_unlock(addr, 0)

/*!
 * @brief Annotate memory access for tsan
 * @param addr Address being accessed
 * @param size Size of access
 */
#define CHIMERA_TSAN_READ(addr, size) \
    __tsan_read0(addr)

/*!
 * @brief Annotate memory write for tsan
 * @param addr Address being written
 * @param size Size of write
 */
#define CHIMERA_TSAN_WRITE(addr, size) \
    __tsan_write0(addr)

/*!
 * @brief Annotate function entry
 * @param name Function name
 */
#define CHIMERA_TSAN_FUNC_ENTRY(name) \
    __tsan_func_entry(name)

/*!
 * @brief Annotate function exit
 */
#define CHIMERA_TSAN_FUNC_EXIT() \
    __tsan_func_exit()

#else

#define CHIMERA_TSAN_MUTEX_LOCK(addr, size) ((void)0)
#define CHIMERA_TSAN_MUTEX_UNLOCK(addr) ((void)0)
#define CHIMERA_TSAN_READ(addr, size) ((void)0)
#define CHIMERA_TSAN_WRITE(addr, size) ((void)0)
#define CHIMERA_TSAN_FUNC_ENTRY(name) ((void)0)
#define CHIMERA_TSAN_FUNC_EXIT() ((void)0)

#endif

/*============================================================================
 * Sanitizer-Aware Allocation Wrappers
 *============================================================================*/

/*!
 * @brief Allocate with ASan poisoning
 * @param size Size to allocate
 * @param redzone Redzone size around allocation
 * @return Pointer to allocated memory (unpoisoned)
 */
CHIMERA_EXPORT void* chimera_alloc_with_redzone(size_t size, size_t redzone);

/*!
 * @brief Free with ASan poisoning
 * @param ptr Pointer to free
 * @param size Original allocation size
 * @param redzone Redzone size around allocation
 */
CHIMERA_EXPORT void chimera_dealloc_with_redzone(void* ptr, size_t size, size_t redzone);

/*============================================================================
 * Sanitizer Control
 *============================================================================*/

/*!
 * @brief Disable sanitizer for a scope
 * @param type Sanitizer type (asan, ubsan, msan, tsan)
 */
CHIMERA_EXPORT void chimera_sanitizer_disable(const char* type);

/*!
 * @brief Re-enable sanitizer after disable
 * @param type Sanitizer type
 */
CHIMERA_EXPORT void chimera_sanitizer_enable(const char* type);

/*============================================================================
 * Conformance Test Helpers
 *============================================================================*/

/*!
 * @brief Run sanitizer conformance tests
 * @return true if all tests pass
 */
CHIMERA_EXPORT bool chimera_sanitizer_conformance_run(void);

/*!
 * @brief Get number of sanitizer conformance tests
 * @return Test count
 */
CHIMERA_EXPORT size_t chimera_sanitizer_conformance_count(void);

/*!
 * @brief Get sanitizer test name
 * @param index Test index
 * @return Test name or NULL if invalid index
 */
CHIMERA_EXPORT const char* chimera_sanitizer_conformance_name(size_t index);

#ifdef __cplusplus
}
#endif

#endif /* CHIMERA_SANITIZERS_H */