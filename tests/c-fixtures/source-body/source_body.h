#ifndef SOURCE_BODY_H
#define SOURCE_BODY_H

/* Task 148: C source-body fixture */

// Simple function with implementation
int add(int a, int b) {
    return a + b;
}

// Function with conditional
int max(int a, int b) {
    if (a > b) {
        return a;
    }
    return b;
}

// Function with pointer arithmetic
int sum_array(int* arr, int len) {
    int total = 0;
    for (int i = 0; i < len; i++) {
        total += arr[i];
    }
    return total;
}

// Global variable
extern int global_counter;

static int get_next(void) {
    return ++global_counter;
}

// Struct with inline access
struct Point {
    int x;
    int y;
};

int point_distance(struct Point* p) {
    return p->x * p->x + p->y * p->y;
}

#endif // SOURCE_BODY_H