const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});

    const main_module = b.createModule(.{
        .target = target,
        .optimize = .ReleaseFast,
        .root_source_file = b.path("src/main.zig"),
    });

    const util_module = b.createModule(.{
        .target = target,
        .optimize = .ReleaseFast,
        .root_source_file = b.path("src/util.zig"),
    });

    const main_obj = b.addObject(.{
        .name = "main.o",
        .root_module = main_module,
    });

    const util_obj = b.addObject(.{
        .name = "util.o",
        .root_module = util_module,
    });

    // Don't install directly - the user can run 'zig build-obj' on each file
    // This build.zig is just for testing the shim's manifest generation
    _ = main_obj;
    _ = util_obj;
}