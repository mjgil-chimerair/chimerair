//! Library fixture for testing repr(C) struct layout.
//!
//! This fixture provides repr(C) structs for layout verification
//! in the chimera-rust-layout and chimera-rust-abi crates.

/// A simple 2D point with explicit layout.
#[repr(C)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// A 3D point with explicit layout.
#[repr(C)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A color value with RGBA components.
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A rectangle defined by two corners.
#[repr(C)]
pub struct Rectangle {
    pub top_left: Point2D,
    pub bottom_right: Point2D,
}

/// A tagged union for testing C-like enum behavior.
#[repr(C)]
pub union IntOrFloat {
    pub as_int: i64,
    pub as_float: f64,
}

/// A C-like enum with explicit discriminants.
#[repr(u32)]
pub enum Status {
    Ok = 0,
    Error = 1,
    Pending = 2,
}

/// A transparent wrapper around a primitive.
#[repr(transparent)]
pub struct Wrapper(i32);

/// A struct with padding for layout testing.
#[repr(C)]
pub struct StructWithPadding {
    pub a: u8,   // 1 byte
    // 7 bytes padding here
    pub b: u64,  // 8 bytes at offset 8
}

/// A struct with alignment requirements.
#[repr(C, align(16))]
pub struct AlignedStruct {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_size() {
        assert_eq!(std::mem::size_of::<Point2D>(), 16);
        assert_eq!(std::mem::align_of::<Point2D>(), 8);
    }

    #[test]
    fn test_point3d_size() {
        assert_eq!(std::mem::size_of::<Point3D>(), 24);
        assert_eq!(std::mem::align_of::<Point3D>(), 8);
    }

    #[test]
    fn test_color_size() {
        assert_eq!(std::mem::size_of::<Color>(), 4);
        assert_eq!(std::mem::align_of::<Color>(), 1);
    }

    #[test]
    fn test_rectangle_size() {
        assert_eq!(std::mem::size_of::<Rectangle>(), 32);
        assert_eq!(std::mem::align_of::<Rectangle>(), 8);
    }

    #[test]
    fn test_int_or_float_size() {
        assert_eq!(std::mem::size_of::<IntOrFloat>(), 8);
        assert_eq!(std::mem::align_of::<IntOrFloat>(), 8);
    }

    #[test]
    fn test_status_size() {
        assert_eq!(std::mem::size_of::<Status>(), 4);
        assert_eq!(std::mem::align_of::<Status>(), 4);
    }

    #[test]
    fn test_wrapper_size() {
        assert_eq!(std::mem::size_of::<Wrapper>(), 4);
        assert_eq!(std::mem::align_of::<Wrapper>(), 4);
    }

    #[test]
    fn test_struct_with_padding_size() {
        assert_eq!(std::mem::size_of::<StructWithPadding>(), 16);
        assert_eq!(std::mem::align_of::<StructWithPadding>(), 8);
    }

    #[test]
    fn test_aligned_struct_size() {
        assert_eq!(std::mem::size_of::<AlignedStruct>(), 16);
        assert_eq!(std::mem::align_of::<AlignedStruct>(), 16);
    }
}