// Test fixture: safety.rs - panic safety testing
// @verify chimera-rust-effects
// @expected ch_status SUCCESS

use std::panic::{catch_unwind, AssertUnwindSafe};

pub struct SafeResource {
    pub data: u32,
}

impl Drop for SafeResource {
    fn drop(&mut self) {
        // This drop is panic-safe - it cannot panic
        self.data = 0;
    }
}

pub fn create_resource(val: u32) -> SafeResource {
    SafeResource { data: val }
}

pub fn process_with_cleanup(input: u32) -> Result<u32, &'static str> {
    let resource = create_resource(input);
    
    let result = match input {
        0 => Err("zero not allowed"),
        1 => Ok(resource.data * 2),
        _ => Ok(resource.data + 10),
    };
    
    // Drop happens here - guaranteed even if result is Err
    result
}

pub fn catch_panic_during_process() -> Option<u32> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        panic!("intentional panic for testing");
    }));
    
    result.is_err()
}
