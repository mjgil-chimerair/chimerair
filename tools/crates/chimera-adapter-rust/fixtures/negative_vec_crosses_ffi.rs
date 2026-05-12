// Negative test: Vec<T> crosses FFI boundary - should be rejected
// This file should fail validation in chimera-adapter-rust

#include <stdint.h>

// INVALID: Vec<u8> is not allowed across FFI boundaries
struct InvalidBuffer {
    uint32_t size;
    void* data;  // Should be validated as "Vec-like" type
};

// This function uses a forbidden type
extern void process_buffer(Vec<uint8_t> buffer);