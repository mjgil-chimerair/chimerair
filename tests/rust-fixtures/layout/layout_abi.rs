// Test fixture: layout_abi.rs - layout and ABI testing
// @verify chimera-rust-layout
// @expected ch_status SUCCESS

#[repr(C)]
pub struct LayoutTest {
    pub a: u8,      // offset 0
    pub b: u32,    // offset 4 (3 bytes padding)
    pub c: u64,    // offset 8
}

// Layout must be: size=16, align=8

#[repr(C)]
pub struct NestedLayout {
    pub x: LayoutTest,
    pub y: u8,     // offset 16
    // total size should be 24 (3 * 8)
}

// Layout must be: size=24, align=8

extern "C" {
    fn c_function(ptr: *mut LayoutTest, val: u32) -> u64;
}

#[no_mangle]
pub extern "C" fn rust_callback(data: *mut LayoutTest) -> u32 {
    unsafe {
        (*data).b = 42;
        (*data).a
    }
}
