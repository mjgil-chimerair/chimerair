//! SHA-256 based hasher for Zigmera artifacts.

use sha2::{Digest, Sha256};

/// Output length for SHA-256 hashes (256 bits = 32 bytes).
pub const SHA256_OUT_LEN: usize = 32;

/// A SHA-256 hash output (256-bit).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Hash([u8; SHA256_OUT_LEN]);

impl Sha256Hash {
    pub fn new(hasher: Sha256) -> Self {
        let mut out = [0u8; SHA256_OUT_LEN];
        out.copy_from_slice(hasher.finalize().as_slice());
        Self(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SHA256_OUT_LEN {
            return None;
        }
        let mut out = [0u8; SHA256_OUT_LEN];
        out.copy_from_slice(bytes);
        Some(Self(out))
    }

    pub fn as_bytes(&self) -> &[u8; SHA256_OUT_LEN] {
        &self.0
    }

    pub fn as_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl serde::Serialize for Sha256Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

/// A domain-tagged SHA-256 hasher.
#[derive(Debug, Clone)]
pub struct Sha256Hasher {
    hasher: Sha256,
}

impl Sha256Hasher {
    pub fn new(domain: &[u8]) -> Self {
        let mut hasher = Sha256::new();
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

    pub fn finalize(self) -> Sha256Hash {
        Sha256Hash::new(self.hasher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash_from_bytes_valid() {
        let bytes = [0u8; 32];
        let hash = Sha256Hash::from_bytes(&bytes).unwrap();
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_sha256_hash_from_bytes_invalid_length() {
        let bytes = [0u8; 31];
        assert!(Sha256Hash::from_bytes(&bytes).is_none());
    }

    #[test]
    fn test_sha256_hasher_domain_tag() {
        let mut hasher1 = Sha256Hasher::with_schema_tag("zsnap");
        hasher1.update(b"test data");
        let hash1 = hasher1.finalize();

        let mut hasher2 = Sha256Hasher::with_schema_tag("zsnap");
        hasher2.update(b"test data");
        let hash2 = hasher2.finalize();

        assert_eq!(hash1, hash2);

        let mut hasher3 = Sha256Hasher::with_schema_tag("zdep");
        hasher3.update(b"test data");
        let hash3 = hasher3.finalize();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_sha256_hasher_determinism() {
        let mut hasher = Sha256Hasher::with_schema_tag("test");
        hasher.update_u64(42);
        hasher.update_str("hello");
        hasher.update_bool(true);
        let hash1 = hasher.finalize();

        let mut hasher2 = Sha256Hasher::with_schema_tag("test");
        hasher2.update_u64(42);
        hasher2.update_str("hello");
        hasher2.update_bool(true);
        let hash2 = hasher2.finalize();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_sha256_hash_hex() {
        let hash = Sha256Hash([0u8; 32]);
        assert_eq!(hash.as_hex(), "0".repeat(64));
    }
}
