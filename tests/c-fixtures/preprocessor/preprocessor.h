#ifndef PREPROCESSOR_H
#define PREPROCESSOR_H

/* Task 155: Macro/include fixture - ABI controlled by macros */

// ABI control macros
#ifdef __x86_64__
#define POINTER_SIZE 8
#define ALIGNMENT 8
#elif defined(__aarch64__)
#define POINTER_SIZE 8
#define ALIGNMENT 8
#else
#define POINTER_SIZE 4
#define ALIGNMENT 4
#endif

// Conditional compilation for platform-specific types
struct PlatformData {
    char pointer[POINTER_SIZE];
    char data[16];
};

// Feature detection macros
#ifdef __STDC_VERSION__
#if __STDC_VERSION__ >= 201112L
#define HAVE_C11 1
#endif
#endif

// Macro-controlled ABI contract
#ifdef DEBUG
#define ASSERT(expr) do { } while(0)
#else
#define ASSERT(expr) ((void)0)
#endif

// Conditional function export
#ifdef EXPORT_API
#define API_FUNC extern
#else
#define API_FUNC static
#endif

API_FUNC int platform_pointer_size(void);

#endif // PREPROCESSOR_H