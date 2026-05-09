//! BLAKE3-based hasher with domain tagging for Zigmera artifacts.

use blake3::Hasher;

/// Output length for BLAKE3 hashes (256 bits = 32 bytes).
pub const BLAKE3_OUT_LEN: usize = 32;

/// A BLAKE3 hash output (256-bit).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Blake3Hash([u8; BLAKE3_OUT_LEN]);

impl Blake3Hash {
    pub fn new(hasher: &Hasher) -> Self {
        let mut out = [0u8; BLAKE3_OUT_LEN];
        hasher.finalize_xof().fill(&mut out);
        Self(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != BLAKE3_OUT_LEN {
            return None;
        }
        let mut out = [0u8; BLAKE3_OUT_LEN];
        out.copy_from_slice(bytes);
        Some(Self(out))
    }

    pub fn as_bytes(&self) -> &[u8; BLAKE3_OUT_LEN] {
        &self.0
    }

    pub fn as_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl serde::Serialize for Blake3Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

/// A domain-tagged BLAKE3 hasher that combines domain context with data.
#[derive(Debug, Clone)]
pub struct Blake3Hasher {
    hasher: Hasher,
}

impl Blake3Hasher {
    pub fn new(domain: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"ZIGMERA-v1");
        hasher.update(domain);
        Self { hasher }
    }

    pub fn with_schema_tag(domain_tag: &str) -> Self {
        Self::new(domain_tag.as_bytes())
    }

    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub fn update_str(&mut self, s: &str) {
        self.hasher.update(s.as_bytes());
    }

    pub fn update_u64(&mut self, val: u64) {
        self.hasher.update(&val.to_le_bytes());
    }

    pub fn update_bool(&mut self, val: bool) {
        self.hasher.update(if val { &[1u8] } else { &[0u8] });
    }

    pub fn finalize(self) -> Blake3Hash {
        Blake3Hash::new(&self.hasher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash_from_bytes_valid() {
        let bytes = [0u8; 32];
        let hash = Blake3Hash::from_bytes(&bytes).unwrap();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_blake3_hash_from_bytes_invalid_length() {
        let bytes = [0u8; 31];
        assert!(Blake3Hash::from_bytes(&bytes).is_none());
    }

    #[test]
    fn test_blake3_hasher_domain_tag() {
        let mut hasher1 = Blake3Hasher::with_schema_tag("zsnap");
        hasher1.update(b"test data");
        let hash1 = hasher1.finalize();

        let mut hasher2 = Blake3Hasher::with_schema_tag("zsnap");
        hasher2.update(b"test data");
        let hash2 = hasher2.finalize();

        // Same domain + same data = same hash
        assert_eq!(hash1, hash2);

        let mut hasher3 = Blake3Hasher::with_schema_tag("zdep");
        hasher3.update(b"test data");
        let hash3 = hasher3.finalize();

        // Different domain = different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_blake3_hasher_determinism() {
        let mut hasher = Blake3Hasher::with_schema_tag("test");
        hasher.update_u64(42);
        hasher.update_str("hello");
        hasher.update_bool(true);
        let hash1 = hasher.finalize();

        let mut hasher2 = Blake3Hasher::with_schema_tag("test");
        hasher2.update_u64(42);
        hasher2.update_str("hello");
        hasher2.update_bool(true);
        let hash2 = hasher2.finalize();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3_hash_hex() {
        let hash = Blake3Hash([0u8; 32]);
        assert_eq!(hash.as_hex(), "0".repeat(64));
    }
}
