const std = @import("std");

/// Add two u32 values.
export fn add(a: u32, b: u32) u32 {
    return a + b;
}

/// Subtract two u32 values.
export fn subtract(a: u32, b: u32) u32 {
    return a - b;
}

/// Multiply two u32 values.
export fn multiply(a: u32, b: u32) u32 {
    return a * b;
}

/// Divide two u32 values.
export fn divide(a: u32, b: u32) u32 {
    return a / b;
}

/// Return the maximum of two u32 values.
export fn max(a: u32, b: u32) u32 {
    return if (a > b) a else b;
}

/// Return the minimum of two u32 values.
export fn min(a: u32, b: u32) u32 {
    return if (a < b) a else b;
}

/// Negate a u32 value.
export fn negate(a: u32) u32 {
    return 0 - a;
}

/// Check if a u32 value is zero.
export fn is_zero(a: u32) bool {
    return a == 0;
}

/// Constant zero.
export const ZERO: u32 = 0;

/// Constant one.
export const ONE: u32 = 1;

const Point2D = extern struct {
    x: i32,
    y: i32,
};

/// Calculate distance from origin (sqrt(x*x + y*y)).
export fn point_distance(p: Point2D) f32 {
    const dx: f32 = @floatFromInt(p.x * p.x);
    const dy: f32 = @floatFromInt(p.y * p.y);
    return @sqrt(dx + dy);
}

/// Create a point at origin.
export fn point_origin() Point2D {
    return Point2D{ .x = 0, .y = 0 };
}

test "add" {
    try std.testing.expect(add(1, 2) == 3);
    try std.testing.expect(add(0, 0) == 0);
}

test "subtract" {
    try std.testing.expect(subtract(5, 3) == 2);
    try std.testing.expect(subtract(1, 1) == 0);
}

test "multiply" {
    try std.testing.expect(multiply(3, 4) == 12);
    try std.testing.expect(multiply(0, 100) == 0);
}

test "divide" {
    try std.testing.expect(divide(10, 2) == 5);
    try std.testing.expect(divide(7, 2) == 3);
}

test "max" {
    try std.testing.expect(max(1, 2) == 2);
    try std.testing.expect(max(100, 50) == 100);
}

test "min" {
    try std.testing.expect(min(1, 2) == 1);
    try std.testing.expect(min(100, 50) == 50);
}

test "negate" {
    try std.testing.expect(negate(5) == 0 - 5);
    try std.testing.expect(negate(0) == 0);
}

test "is_zero" {
    try std.testing.expect(is_zero(0) == true);
    try std.testing.expect(is_zero(1) == false);
}

test "point_distance" {
    const p = Point2D{ .x = 3, .y = 4 };
    try std.testing.expect(point_distance(p) == 5.0);
}

test "point_origin" {
    const p = point_origin();
    try std.testing.expect(p.x == 0);
    try std.testing.expect(p.y == 0);
}