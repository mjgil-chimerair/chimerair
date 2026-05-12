//! Bitfield fixture
//!
//! Task 150: Bitfield fixture

#ifndef BITFIELD_H
#define BITFIELD_H

#include <stdint.h>

// Simple bitfield struct
struct SimpleBitfield {
    unsigned int flags : 3;
    unsigned int mode : 5;
};

// Mixed bitfield and regular fields
struct MixedBitfield {
    int regular;
    unsigned int bits : 4;
    double padding;
    unsigned int more_bits : 12;
};

// Packed bitfield struct
struct __attribute__((packed)) PackedBitfield {
    unsigned int a : 8;
    unsigned int b : 8;
    unsigned int c : 8;
    unsigned int d : 8;
};

// Bitfield with signed type
struct SignedBitfield {
    signed int value : 8;
    signed int sign_ext : 7;
    unsigned int unsigned_part : 1;
};

// Large bitfield struct
struct LargeBitfield {
    uint64_t field1 : 16;
    uint64_t field2 : 16;
    uint64_t field3 : 16;
    uint64_t field4 : 16;
};

#endif // BITFIELD_H