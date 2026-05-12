#ifndef FLEXIBLE_ARRAY_H
#define FLEXIBLE_ARRAY_H

/* Task 151: Flexible array fixture */

#include <stddef.h>

// Flexible array member (must be last member of struct)
struct Buffer {
    size_t capacity;
    size_t length;
    char data[];  // Flexible array member
};

// Associated length contract
#define BUFFER_SIZE(b) ((b)->capacity * sizeof(char))
#define BUFFER_LENGTH(b) ((b)->length)

// Safe initialization macro
#define INIT_BUFFER(p, cap) do { \
    (p)->capacity = (cap); \
    (p)->length = 0; \
} while(0)

#endif // FLEXIBLE_ARRAY_H