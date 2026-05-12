//! Build script fixture for testing build.rs tracking.
//!
//! This fixture demonstrates various cargo:rustc-* instructions
//! that chimera-rust-cargo must track for proper cache invalidation.

fn main() {
    // Tell rustc to rerun this build script if config.txt changes
    println!("cargo:rerun-if-changed=config.txt");

    // Add a cfg flag
    println!("cargo:rustc-cfg=feature=\"extended\"");

    // Link against a system library (for demonstration)
    // Note: This is commented out as it may not be available on all systems
    // println!("cargo:rustc-link-lib=ffi");

    // Print environment info that would affect compilation
    println!("cargo:rustc-env=BUILD_SCRIPT_VERSION=1.0");

    // Read config if it exists
    let config = std::fs::read_to_string("config.txt").unwrap_or_default();
    if config.contains("debug") {
        println!("cargo:rustc-cfg=debug_build");
    }
}