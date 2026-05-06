//! Canonical serialization for deterministic hashing.
//!
//! Ensures consistent byte ordering across platforms.

/// Trait for types that can be canonicalized for hashing.
pub trait Canonicalize {
    fn canonicalize(&self, f: &mut CanonicalFormatter);
}

/// A formatter that accumulates canonical bytes.
#[derive(Debug)]
pub struct CanonicalFormatter {
    bytes: Vec<u8>,
}

impl CanonicalFormatter {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn write(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }

    pub fn write_u8(&mut self, val: u8) {
        self.bytes.push(val);
    }

    pub fn write_u16(&mut self, val: u16) {
        self.bytes.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u32(&mut self, val: u32) {
        self.bytes.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_u64(&mut self, val: u64) {
        self.bytes.extend_from_slice(&val.to_le_bytes());
    }

    pub fn write_i32(&mut self, val: i32) {
        self.bytes.extend_from_slice(&(val as u32).to_le_bytes());
    }

    pub fn write_bool(&mut self, val: bool) {
        self.bytes.push(if val { 1 } else { 0 });
    }

    pub fn write_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.bytes.extend_from_slice(bytes);
    }
}

impl Default for CanonicalFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl serde::Serialize for CanonicalFormatter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.bytes)
    }
}

/// Helper to canonicalize an optional value.
pub fn canonicalize_opt<T: Canonicalize>(opt: &Option<T>, f: &mut CanonicalFormatter) {
    match opt {
        Some(v) => {
            f.write_bool(true);
            v.canonicalize(f);
        }
        None => {
            f.write_bool(false);
        }
    }
}

/// Helper to canonicalize a sequence.
pub fn canonicalize_seq<T: Canonicalize>(seq: &[T], f: &mut CanonicalFormatter) {
    f.write_u32(seq.len() as u32);
    for item in seq {
        item.canonicalize(f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_formatter_u64() {
        let mut f = CanonicalFormatter::new();
        f.write_u64(0x123456789ABCDEF0);
        let bytes = f.into_bytes();
        assert_eq!(bytes.len(), 8);
        assert_eq!(bytes, 0x123456789ABCDEF0u64.to_le_bytes());
    }

    #[test]
    fn test_canonical_formatter_str() {
        let mut f = CanonicalFormatter::new();
        f.write_str("hello");
        let bytes = f.into_bytes();
        // len (4 bytes) + "hello" (5 bytes) = 9 bytes
        assert_eq!(bytes.len(), 9);
    }

    #[test]
    fn test_canonical_formatter_bool() {
        let mut f = CanonicalFormatter::new();
        f.write_bool(true);
        f.write_bool(false);
        let bytes = f.into_bytes();
        assert_eq!(bytes, vec![1, 0]);
    }
}
