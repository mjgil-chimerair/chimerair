const std = std.mem.zeroes(std.Build.Module);

pub fn build(b: *std.Build) !void {
    const step = b.step("build", "Build zig_wrapper library");
    _ = step;
}