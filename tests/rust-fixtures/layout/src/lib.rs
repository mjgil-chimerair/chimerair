//! Layout fixture demonstrating repr(C) struct with generated layout assertions.
//!
//! This fixture tests the chimera-rust-layout crate's handling of struct
//! field offsets, padding, alignment, and niche optimization.

/// A repr(C) struct with multiple field types for layout testing.
///
/// This struct is used to verify that Chimera generates correct layout
/// assertions for struct size, alignment, and field offsets.
#[repr(C)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// A repr(C) struct with mixed types for padding testing.
#[repr(C)]
pub struct MixedFields {
    pub a: u8,      // 1 byte, offset 0
    pub b: u32,     // 4 bytes, offset 4 (3 bytes padding after a)
    pub c: u8,      // 1 byte, offset 8
    pub d: u64,     // 8 bytes, offset 16 (7 bytes padding after c)
}                   // Total size: 24 bytes

/// A repr(C) struct with nested repr(C) structs.
#[repr(C)]
pub struct Rectangle {
    pub origin: Point2D,  // 16 bytes at offset 0
    pub width: f64,       // 8 bytes at offset 16
    pub height: f64,      // 8 bytes at offset 24
}                          // Total size: 32 bytes

/// A repr(C) struct with explicit alignment using #[repr(align(N))].
#[repr(C)]
pub struct AlignedStruct {
    pub a: u8,
    pub b: u32,
}

/// A repr(C) packed struct without padding.
#[repr(C, packed)]
pub struct PackedStruct {
    pub a: u8,   // 1 byte, offset 0
    pub b: u32,  // 4 bytes, offset 1
    pub c: u8,   // 1 byte, offset 5
}                  // Total size: 6 bytes

/// A repr(C) struct with an array field.
#[repr(C)]
pub struct WithArray {
    pub flags: u32,      // 4 bytes at offset 0
    pub data: [u8; 16],  // 16 bytes at offset 4
    pub count: u64,      // 8 bytes at offset 20
}                         // Total size: 28 bytes

/// Calculate distance between two points.
#[no_mangle]
pub extern "C" fn point_distance(p1: Point2D, p2: Point2D) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

/// Calculate area of a rectangle.
#[no_mangle]
pub extern "C" fn rectangle_area(r: Rectangle) -> f64 {
    r.width * r.height
}

/// Check if a point is inside a rectangle.
#[no_mangle]
pub extern "C" fn point_in_rectangle(point: Point2D, rect: Rectangle) -> bool {
    point.x >= rect.origin.x
        && point.x <= rect.origin.x + rect.width
        && point.y >= rect.origin.y
        && point.y <= rect.origin.y + rect.height
}

/// Get the size of a Point2D struct.
#[no_mangle]
pub extern "C" fn size_of_point() -> usize {
    std::mem::size_of::<Point2D>()
}

/// Get the alignment of a Point2D struct.
#[no_mangle]
pub extern "C" fn align_of_point() -> usize {
    std::mem::align_of::<Point2D>()
}

/// Get the size of a MixedFields struct.
#[no_mangle]
pub extern "C" fn size_of_mixed() -> usize {
    std::mem::size_of::<MixedFields>()
}

/// Get the size of a PackedStruct (no padding).
#[no_mangle]
pub extern "C" fn size_of_packed() -> usize {
    std::mem::size_of::<PackedStruct>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_size() {
        assert_eq!(std::mem::size_of::<Point2D>(), 16);
        assert_eq!(std::mem::align_of::<Point2D>(), 8);
    }

    #[test]
    fn test_mixed_fields_size() {
        // MixedFields: u8(1) + 3 padding + u32(4) + u8(1) + 7 padding + u64(8) = 24
        assert_eq!(std::mem::size_of::<MixedFields>(), 24);
        assert_eq!(std::mem::align_of::<MixedFields>(), 8);
    }

    #[test]
    fn test_packed_size() {
        // PackedStruct: u8(1) + u32(4) + u8(1) = 6
        assert_eq!(std::mem::size_of::<PackedStruct>(), 6);
    }

    #[test]
    fn test_rectangle_size() {
        assert_eq!(std::mem::size_of::<Rectangle>(), 32);
    }

    #[test]
    fn test_with_array_size() {
        // WithArray: u32(4) + [u8; 16](16) + 4 padding + u64(8) = 32
        assert_eq!(std::mem::size_of::<WithArray>(), 32);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point2D { x: 0.0, y: 0.0 };
        let p2 = Point2D { x: 3.0, y: 4.0 };
        assert_eq!(point_distance(p1, p2), 5.0);
    }

    #[test]
    fn test_rectangle_area() {
        let rect = Rectangle {
            origin: Point2D { x: 0.0, y: 0.0 },
            width: 10.0,
            height: 5.0,
        };
        assert_eq!(rectangle_area(rect), 50.0);
    }

    #[test]
    fn test_point_in_rectangle() {
        let rect = Rectangle {
            origin: Point2D { x: 0.0, y: 0.0 },
            width: 10.0,
            height: 10.0,
        };
        let rect2 = Rectangle {
            origin: Point2D { x: 0.0, y: 0.0 },
            width: 10.0,
            height: 10.0,
        };
        let inside = Point2D { x: 5.0, y: 5.0 };
        let outside = Point2D { x: 15.0, y: 15.0 };
        assert!(point_in_rectangle(inside, rect));
        assert!(!point_in_rectangle(outside, rect2));
    }
}