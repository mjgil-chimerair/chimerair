// chimera_abi.zig
// Chimera ABI support for Zig
//
// Provides extern structs and helpers for interfacing with Chimera ABI.

const std = @import("std");

/// Version information
pub const CHIMERA_ABI_VERSION_MAJOR: u32 = 0;
pub const CHIMERA_ABI_VERSION_MINOR: u32 = 1;
pub const CHIMERA_ABI_VERSION_PATCH: u32 = 0;
pub const CHIMERA_ABI_VERSION_STRING: []const u8 = "0.1.0";

/// Status codes for Chimera ABI operations
pub const Status = enum(i32) {
    ok = 0,
    err = 1,
    invalid_arg = 2,
    invalid_state = 3,
    not_found = 4,
    out_of_memory = 5,
    buffer_too_small = 6,
    type_mismatch = 7,
    borrow_exclusive = 8,
    use_after_move = 9,
    double_free = 10,
    panic = 11,
};

/// Check if status indicates success
pub fn statusIsOk(status: Status) bool {
    return status == .ok;
}

/// Check if status indicates an error
pub fn statusIsError(status: Status) bool {
    return status != .ok;
}

/// Error domains for rich error categorization
pub const ErrorDomain = enum(i32) {
    none = 0,
    io = 1,
    memory = 2,
    type_enum = 3,
    ownership = 4,
    validation = 5,
    runtime = 6,
};

/// Ownership kinds for borrowed/owned values
pub const Ownership = enum(i32) {
    borrowed = 0,
    borrowed_mut = 1,
    owned = 2,
    raw = 3,
};

/// Lifetime kinds for borrowed values
pub const Lifetime = enum(i32) {
    call = 0,
    static_enum = 1,
    owner = 2,
};

/// Calling convention identifiers
pub const CConv = enum(i32) {
    c = 0,
    sysv = 1,
    win64 = 2,
    wasm = 3,
    chimera = 4,
};

/// Target architecture identifiers
pub const TargetArch = enum(i32) {
    x86_64 = 0,
    aarch64 = 1,
    wasm32 = 2,
    wasm64 = 3,
};

/// Target OS identifiers
pub const TargetOs = enum(i32) {
    linux = 0,
    macos = 1,
    windows = 2,
    wasi = 3,
};

/// Allocator allocation kinds
pub const AllocKind = enum(i32) {
    new = 0,
    malloc = 1,
    resize = 2,
    free = 3,
};

/// Canonical lowered allocator descriptor
pub const ChAllocator = extern struct {
    id: u64,
    kind: u32,
    ptr: ?*anyopaque,
};

/// Canonical lowered error payload
pub const ChError = extern struct {
    domain: u32,
    code: u32,
    flags: u32,
    message_ptr: ?*const anyopaque,
    message_len: u64,
    payload_ptr: ?*anyopaque,
    payload_drop_fn: ?*const fn (?*anyopaque) callconv(.c) void,
    payload_drop_ctx: ?*anyopaque,
};

/// Slice type for borrowed sequences
pub const Slice = extern struct {
    data: [*]const u8,
    len: usize,

    /// Check if slice is empty
    pub fn isEmpty(self: Slice) bool {
        return self.len == 0;
    }

    /// Get the length of the slice
    pub fn getLen(self: Slice) usize {
        return self.len;
    }

    /// Create an empty slice
    pub fn empty() Slice {
        return Slice{
            .data = undefined,
            .len = 0,
        };
    }
};

/// Canonical lowered byte slice
pub const ChSlice = extern struct {
    ptr: ?*const anyopaque,
    len: u64,
};

/// Mutable slice type
pub const SliceMut = extern struct {
    data: [*]u8,
    len: usize,

    pub fn isEmpty(self: SliceMut) bool {
        return self.len == 0;
    }

    pub fn getLen(self: SliceMut) usize {
        return self.len;
    }
};

/// String type (always UTF-8)
pub const String = extern struct {
    data: [*]const u8,
    len: usize,
    capacity: usize,

    pub fn getLen(self: String) usize {
        return self.len;
    }

    pub fn isEmpty(self: String) bool {
        return self.len == 0;
    }

    pub fn getCapacity(self: String) usize {
        return self.capacity;
    }
};

/// Canonical borrowed UTF-8 string
pub const ChBorrowStr = extern struct {
    ptr: [*]const u8,
    len: u64,
    lifetime: u32,
};

/// Canonical owned byte buffer
pub const ChOwnedBytes = extern struct {
    ptr: [*]u8,
    len: u64,
    capacity: u64,
    allocator_id: u64,
};

/// Canonical opaque owned handle
pub const ChHandle = extern struct {
    ptr: ?*anyopaque,
    drop_fn: ?*const fn (?*anyopaque) callconv(.c) void,
    size: u64,
};

/// Result type
pub const Result = extern struct {
    is_ok: bool,
    _padding: [7]u8 = undefined,

    pub fn isOk(self: Result) bool {
        return self.is_ok;
    }

    pub fn isErr(self: Result) bool {
        return !self.is_ok;
    }
};

/// Panic behavior on panic boundary
pub const PanicPolicy = enum(i32) {
    abort = 0,
    unwind = 1,
    rust = 2,
};

/// Panic info passed across boundary
pub const PanicInfo = extern struct {
    message: [*]const u8,
    message_len: usize,
    file: [*]const u8,
    line: i32,
    reason: [*]const u8,
};

/// Create a result that indicates OK
pub fn resultOk() Result {
    return Result{ .is_ok = true };
}

/// Create a result that indicates error
pub fn resultErr() Result {
    return Result{ .is_ok = false };
}

// Test module
test "status functions" {
    try std.testing.expect(statusIsOk(.ok));
    try std.testing.expect(!statusIsOk(.err));
    try std.testing.expect(statusIsError(.err));
    try std.testing.expect(!statusIsError(.ok));
}

test "slice functions" {
    const empty = Slice.empty();
    try std.testing.expect(empty.isEmpty());
    try std.testing.expect(empty.getLen() == 0);
}

test "result functions" {
    const ok = resultOk();
    const err = resultErr();
    try std.testing.expect(ok.isOk());
    try std.testing.expect(!ok.isErr());
    try std.testing.expect(err.isErr());
    try std.testing.expect(!err.isOk());
}

test "enum values" {
    try std.testing.expect(@intFromEnum(Ownership.borrowed) == 0);
    try std.testing.expect(@intFromEnum(Ownership.borrowed_mut) == 1);
    try std.testing.expect(@intFromEnum(Ownership.owned) == 2);
    try std.testing.expect(@intFromEnum(Ownership.raw) == 3);

    try std.testing.expect(@intFromEnum(Lifetime.call) == 0);
    try std.testing.expect(@intFromEnum(Lifetime.static_enum) == 1);
    try std.testing.expect(@intFromEnum(Lifetime.owner) == 2);

    try std.testing.expect(@intFromEnum(CConv.c) == 0);
    try std.testing.expect(@intFromEnum(CConv.sysv) == 1);
    try std.testing.expect(@intFromEnum(CConv.win64) == 2);
    try std.testing.expect(@intFromEnum(CConv.wasm) == 3);
    try std.testing.expect(@intFromEnum(CConv.chimera) == 4);

    try std.testing.expect(@intFromEnum(TargetArch.x86_64) == 0);
    try std.testing.expect(@intFromEnum(TargetArch.aarch64) == 1);
    try std.testing.expect(@intFromEnum(TargetArch.wasm32) == 2);

    try std.testing.expect(@intFromEnum(TargetOs.linux) == 0);
    try std.testing.expect(@intFromEnum(TargetOs.macos) == 1);
    try std.testing.expect(@intFromEnum(TargetOs.windows) == 2);
    try std.testing.expect(@intFromEnum(TargetOs.wasi) == 3);

    try std.testing.expect(@intFromEnum(AllocKind.new) == 0);
    try std.testing.expect(@intFromEnum(AllocKind.malloc) == 1);
    try std.testing.expect(@intFromEnum(AllocKind.resize) == 2);
    try std.testing.expect(@intFromEnum(AllocKind.free) == 3);
}

test "canonical struct sizes and alignments match Lean" {
    try std.testing.expect(@sizeOf(ChError) == 56);
    try std.testing.expect(@alignOf(ChError) == 8);
    try std.testing.expect(@sizeOf(ChAllocator) == 24);
    try std.testing.expect(@alignOf(ChAllocator) == 8);
    try std.testing.expect(@sizeOf(ChSlice) == 16);
    try std.testing.expect(@alignOf(ChSlice) == 8);
    try std.testing.expect(@sizeOf(ChBorrowStr) == 24);
    try std.testing.expect(@alignOf(ChBorrowStr) == 8);
    try std.testing.expect(@sizeOf(ChOwnedBytes) == 32);
    try std.testing.expect(@alignOf(ChOwnedBytes) == 8);
    try std.testing.expect(@sizeOf(ChHandle) == 24);
    try std.testing.expect(@alignOf(ChHandle) == 8);
}

test "canonical struct field offsets match Lean" {
    try std.testing.expect(@offsetOf(ChError, "domain") == 0);
    try std.testing.expect(@offsetOf(ChError, "code") == 4);
    try std.testing.expect(@offsetOf(ChError, "flags") == 8);
    try std.testing.expect(@offsetOf(ChError, "message_ptr") == 16);
    try std.testing.expect(@offsetOf(ChError, "message_len") == 24);
    try std.testing.expect(@offsetOf(ChError, "payload_ptr") == 32);
    try std.testing.expect(@offsetOf(ChError, "payload_drop_fn") == 40);
    try std.testing.expect(@offsetOf(ChError, "payload_drop_ctx") == 48);
    try std.testing.expect(@offsetOf(ChHandle, "ptr") == 0);
    try std.testing.expect(@offsetOf(ChHandle, "drop_fn") == 8);
    try std.testing.expect(@offsetOf(ChHandle, "size") == 16);
}
