#ifndef ALLOCATOR_H
#define ALLOCATOR_H

/* Task 154: Allocator fixture */

#include <stddef.h>

// Standard allocator API
void* chimera_alloc(size_t size);
void chimera_free(void* ptr);

// Owned memory handle
struct Handle {
    void* data;
    size_t size;
};

struct Handle* chimera_handle_create(size_t size);
void chimera_handle_destroy(struct Handle* h);

// Drop trampoline for foreign callers
void chimera_drop_owned(void* ptr);

#endif // ALLOCATOR_H