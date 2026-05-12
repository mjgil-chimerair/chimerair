#ifndef VARARGS_H
#define VARARGS_H

/* Task 156: Varargs fixture */

#include <stdarg.h>

// Simple varargs function
int sum(int count, ...);

// Formatted varargs
void log_message(int level, const char* fmt, ...);

// Va_list passthrough
int vprintf_wrapper(const char* fmt, va_list args);

// Macro for initializing va_list
#ifdef __GNUC__
#define VA_START(ap, last) __builtin_va_start(ap, last)
#define VA_END(ap) __builtin_va_end(ap)
#else
#include <stdarg.h>
#define VA_START(ap, last) va_start(ap, last)
#define VA_END(ap) va_end(ap)
#endif

#endif // VARARGS_H