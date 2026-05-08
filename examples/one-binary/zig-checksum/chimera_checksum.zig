// chimera_checksum.zig
// Zig checksum component using Chimera ABI
//
// Provides checksum calculation with explicit error handling.

const std = @import("std");

/// Checksum algorithm selection
pub const ChecksumAlgorithm = enum(u32) {
    crc32 = 0,
    fletcher16 = 1,
    fletcher32 = 2,
};

/// Checksum result
pub const ChecksumResult = struct {
    value: u32,
    algorithm: ChecksumAlgorithm,
};

/// Calculate CRC32 checksum
pub fn crc32(data: []const u8) u32 {
    var crc: u32 = 0xFFFFFFFF;
    for (data) |byte| {
        crc ^= @as(u32, byte);
        var i: usize = 0;
        while (i < 8) : (i += 1) {
            crc = if ((crc & 1) != 0) (crc >> 1) ^ 0xEDB88320 else crc >> 1;
        }
    }
    return ~crc;
}

/// Calculate Fletcher-16 checksum
pub fn fletcher16(data: []const u8) u16 {
    var sum1: u16 = 0;
    var sum2: u16 = 0;
    for (data) |byte| {
        sum1 = sum1 +% byte;
        sum2 = sum2 +% sum1;
    }
    return sum1 | (sum2 << 8);
}

/// Calculate Fletcher-32 checksum
pub fn fletcher32(data: []const u8) u32 {
    var sum1: u32 = 0;
    var sum2: u32 = 0;
    for (data) |byte| {
        sum1 = sum1 +% @as(u32, byte);
        sum2 = sum2 +% sum1;
    }
    return sum1 | (sum2 << 16);
}

/// Calculate checksum using specified algorithm
pub fn calculate(data: []const u8, algorithm: ChecksumAlgorithm) ChecksumResult {
    const value: u32 = switch (algorithm) {
        .crc32 => crc32(data),
        .fletcher16 => @as(u32, fletcher16(data)),
        .fletcher32 => @as(u32, fletcher32(data)),
    };
    return ChecksumResult{
        .value = value,
        .algorithm = algorithm,
    };
}

export fn chimera_zig_crc32(data: [*]const u8, len: usize) u32 {
    return crc32(data[0..len]);
}

/// Format checksum result as hex string
pub fn formatHex(result: ChecksumResult, buffer: []u8) []u8 {
    const hex_chars = "0123456789abcdef";
    var value = result.value;
    var i: usize = 0;
    while (i < 8 and i < buffer.len) : (i += 1) {
        const nibble = @as(u8, @truncate(value & 0xF));
        buffer[7 - i] = hex_chars[nibble];
        value >>= 4;
    }
    return buffer[0..@min(8, buffer.len)];
}

test "crc32" {
    const data = "hello world";
    const crc = crc32(data);
    try std.testing.expect(crc == 0x0D4A1185);
}

test "fletcher16" {
    const data = "test";
    const result = fletcher16(data);
    try std.testing.expect(result != 0);
}

test "calculate with algorithm" {
    const data = "hello";
    const result = calculate(data, .crc32);
    try std.testing.expect(result.algorithm == .crc32);
    try std.testing.expect(result.value != 0);
}

test "chimera_zig_crc32 export" {
    const data = "hello";
    try std.testing.expect(chimera_zig_crc32(data.ptr, data.len) == crc32(data));
}

test "format hex" {
    const result = ChecksumResult{ .value = 0xDEADBEEF, .algorithm = .crc32 };
    var buffer: [16]u8 = undefined;
    const formatted = formatHex(result, &buffer);
    try std.testing.expect(formatted.len > 0);
}
