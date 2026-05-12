// Negative test: slice ([]T) crosses FFI boundary - should be rejected
// This file should fail validation in chimera-adapter-zig

const std = @import("std");

// INVALID: []const u8 is a slice, not allowed across FFI boundaries
pub fn process_data(data: []const u8) void {
    _ = data;
}

// Zig error union crosses FFI boundary - should be rejected
pub fn read_file(path: []const u8) !std.fs.File {
    _ = path;
    return error.FileNotFound;
}