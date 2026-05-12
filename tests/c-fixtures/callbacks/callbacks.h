//! Callback fixture
//!
//! Task 153: Callback fixture

#ifndef CALLBACKS_H
#define CALLBACKS_H

#include <stdint.h>

// Opaque context handle
typedef struct Context Context;

// Callback function types
typedef int (*ProcessCallback)(const char* input, void* user_data);
typedef void (*CompletionCallback)(int result, void* user_data);
typedef int (*ValidationCallback)(int value, const char** error_msg);

// Struct containing callbacks
typedef struct {
    ProcessCallback process;
    CompletionCallback complete;
    ValidationCallback validate;
    void* user_data;
} CallbackBundle;

// Function taking simple callback
int process_with_callback(const char* data, ProcessCallback cb, void* user_data);

// Function taking completion callback
void async_operation(int input, CompletionCallback cb, void* user_data);

// Function returning callback
typedef CompletionCallback (*CallbackFactory)(void);

// Function taking multiple callbacks
int execute_with_callbacks(
    int value,
    ValidationCallback validate,
    ProcessCallback process,
    CompletionCallback complete,
    void* user_data
);

// Function pointer as struct field
typedef struct {
    const char* name;
    int (*fn)(int, int);
    void* next;
} Node;

// Function taking struct with function pointer
int traverse_list(Node* head, int (*visit)(int value, void* ctx), void* ctx);

#endif // CALLBACKS_H