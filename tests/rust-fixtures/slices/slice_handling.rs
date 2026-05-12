// Test fixture: slice_handling.rs - slice/pass-by-value testing
// @verify chimera-rust-schema
// @expected ch_status SUCCESS

pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer { data: Vec::new() }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Buffer { data: slice.to_vec() }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub fn sum_bytes(data: &[u8]) -> u64 {
    data.iter().map(|&b| b as u64).sum()
}

pub fn find_byte(data: &[u8], target: u8) -> Option<usize> {
    data.iter().position(|&b| b == target)
}
