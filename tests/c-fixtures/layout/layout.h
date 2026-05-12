//! Layout fixture
//!
//! Task 149: Struct/layout fixture

#ifndef LAYOUT_H
#define LAYOUT_H

#include <stdint.h>

// Simple struct
struct Simple {
    int a;
    int b;
};

// Nested struct
struct Nested {
    struct Simple inner;
    int c;
};

// Packed struct (no padding)
struct __attribute__((packed)) PackedStruct {
    int8_t a;
    int32_t b;
    int16_t c;
};

// Explicitly aligned struct
struct __attribute__((aligned(16))) AlignedStruct {
    double d;
    int i;
};

// Struct with flexible array member
struct WithFlexibleArray {
    int count;
    int data[];
};

// Bitfield struct (separate fixture but related)
struct BitfieldStruct {
    unsigned int a : 3;
    unsigned int b : 5;
    unsigned int c : 8;
    unsigned int d : 16;
};

#endif // LAYOUT_H