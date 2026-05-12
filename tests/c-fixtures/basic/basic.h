//! Basic C smoke fixture
//!
//! Task 146: Minimal C smoke fixture

#ifndef BASIC_H
#define BASIC_H

// Simple exported function
int add(int a, int b);

// Const function (no side effects)
int multiply(int a, int b);

// Struct for testing layout
struct Point {
    int x;
    int y;
};

// Function using the struct
int point_distance(struct Point p);

#endif // BASIC_H