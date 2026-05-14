const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const mode = b.standardReleaseOptions();

    const lib = b.addLibrary(.{
        .name = "zig-chimera-fixture",
        .target = target,
        .optimize = mode,
    });

    lib.addRootSourceFile("src/main.zig");
    lib.setExportManifest(.{
        .root_src_file = lib.root_source_file.?,
    });

    b.installArtifact(lib);
}