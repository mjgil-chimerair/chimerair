// build.zig
// Build script for Chimera Zig checksum component

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Create chimera_abi module from the runtime
    const chimera_abi_mod = b.createModule(.{
        .root_source_file = b.path("../../../runtime/zig/chimera_abi.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Create chimera-checksum module that imports chimera
    const mod = b.addModule("chimera-checksum", .{
        .root_source_file = b.path("chimera_checksum.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "chimera", .module = chimera_abi_mod },
        },
    });

    // Create a test executable
    const tests = b.addTest(.{
        .root_module = mod,
    });

    // Add test step
    const test_step = b.step("test", "Run all tests");
    test_step.dependOn(&b.addRunArtifact(tests).step);
}