// Negative test: Result<T, E> crosses FFI boundary - should be rejected
// This file should fail validation in chimera-adapter-rust

#include <stdint.h>

// INVALID: Result<String, Error> crosses FFI boundary
// Native Rust Result types are not FFI-safe

extern int32_t get_config(
    const char* key,
    Result<uint64_t, ConfigError>* out_value  // INVALID: Result not allowed
);