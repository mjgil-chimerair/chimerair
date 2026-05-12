//! Header-only fixture
//!
//! Task 147: C header-only fixture

#ifndef HEADER_ONLY_H
#define HEADER_ONLY_H

#include <stdint.h>

// Typedef for opaque handle
typedef void* Handle;

// Enum for status codes
typedef enum {
    OK = 0,
    ERROR_INVALID = -1,
    ERROR_OOM = -2,
    ERROR_BUSY = -3
} StatusCode;

// Struct with various field types
typedef struct {
    int32_t id;
    const char* name;
    uint64_t flags;
    double value;
} Config;

// Struct with nested struct
typedef struct {
    Config base;
    int priority;
} ExtendedConfig;

// Function declaration with const parameters
StatusCode init_handle(Handle h, const Config* cfg);

// Function returning struct by value
Config make_default_config(void);

// Function with pointer parameters
int process_config(const Config* input, Config* output);

// Static assertion to verify struct sizes at compile time
// This will fail to compile if sizes don't match expected values
#define STATIC_ASSERT_SIZE(s, expected) _Static_assert(sizeof(s) == expected, #s " size mismatch")
STATIC_ASSERT_SIZE(Config, 32);
STATIC_ASSERT_SIZE(ExtendedConfig, 40);

#endif // HEADER_ONLY_H