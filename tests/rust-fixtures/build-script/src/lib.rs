//! Library with build script for testing.
//!
//! This crate uses build.rs to emit cargo:rustc-* instructions
//! that chimera-rust-cargo must track.

/// Get the build script version.
#[no_mangle]
pub extern "C" fn get_version() -> i32 {
    1
}

/// Check if the extended feature is enabled.
#[cfg(feature = "extended")]
pub fn has_extended_feature() -> bool {
    true
}

#[cfg(not(feature = "extended"))]
pub fn has_extended_feature() -> bool {
    false
}