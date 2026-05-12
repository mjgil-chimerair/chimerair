//! Error handling fixture
//!
//! Task 152: errno/status fixture

#ifndef ERRORS_H
#define ERRORS_H

#include <errno.h>
#include <stddef.h>
#include <stdint.h>

// Status code enum (ch_status style)
typedef enum {
    CH_STATUS_OK = 0,
    CH_STATUS_ERROR = -1,
    CH_STATUS_INVALID_ARG = -2,
    CH_STATUS_NOT_FOUND = -3,
    CH_STATUS_OOM = -4
} MyStatus;

// Function returning error code directly
int process_data(int* output);

// Function using errno
double compute_ratio(int a, int b);

// Function returning negative on error, positive on success
int64_t compute_hash(const void* data, size_t len);

// Struct with error field
typedef struct {
    int code;
    const char* message;
} ErrorInfo;

// Function that fills ErrorInfo
int get_error_info(int code, ErrorInfo* info);

// Callback type for error handling
typedef void (*ErrorHandler)(int error_code, const char* msg);

// Function taking error handler callback
void set_error_handler(ErrorHandler handler);

// Function using ch_status convention (status as return, data via out param)
typedef int (*ProcessFn)(void* input, void* output, int* status);

#endif // ERRORS_H