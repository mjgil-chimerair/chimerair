const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const mode = b.standardReleaseOptions();

    // Create a library that exports functions
    const lib = b.addLibrary(.{
        .name = "zig_lib",
        .root_module = b.createModule(.{
            .target = target,
            .optimize_mode = mode,
        }),
        .target = target,
    });

    // Add the source file
    lib.addCSourceFile(.{
        .file = b.path("src/main.zig"),
        .flags = &[_][]const u8{},
    });

    // This library exports functions that call into Rust via extern
    lib.linkLibC(); // For extern function calls

    b.installArtifact(lib);
}