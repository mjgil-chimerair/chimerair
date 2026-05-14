const Builder = @import("std").build.Builder;

pub fn build(b: *Builder) void {
    b.setOption("target", "x86_64-unknown-linux-gnu");
    b.setOption("optimize", "ReleaseFast");
    b.setOption("ver", "0.1.0");

    const exe = b.addExecutable("bun-test", "src/main.zig");
    exe.linkLibC();
    b.default_step.dependOn(&exe.step);
}