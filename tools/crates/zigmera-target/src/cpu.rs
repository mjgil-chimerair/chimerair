//! CPU feature detection and representation.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// CPU features for a target.
#[derive(Debug, Clone, Default)]
pub struct CpuFeatures {
    features: HashSet<String>,
}

impl CpuFeatures {
    pub fn new() -> Self {
        Self { features: HashSet::new() }
    }

    pub fn with_features(features: Vec<&str>) -> Self {
        Self {
            features: features.into_iter().map(String::from).collect(),
        }
    }

    pub fn add(&mut self, feature: &str) {
        self.features.insert(feature.to_string());
    }

    pub fn has(&self, feature: &str) -> bool {
        self.features.contains(feature)
    }

    pub fn remove(&mut self, feature: &str) {
        self.features.remove(feature);
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    pub fn len(&self) -> usize {
        self.features.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.features.iter().map(|s| s.as_str())
    }
}

impl PartialEq for CpuFeatures {
    fn eq(&self, other: &Self) -> bool {
        self.features == other.features
    }
}

impl Eq for CpuFeatures {}

impl Serialize for CpuFeatures {
    fn serialize<S>(&self, serializer: S) -> S::Result
    where
        S: serde::Serializer,
    {
        let sorted: Vec<&str> = let mut v: Vec<_> = self.features.iter().map(|s| s.as_str()).collect();
        v.sort();
        serializer.serialize_str(&v.join(","))
    }
}

impl<'de> Deserialize<'de> for CpuFeatures {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let features = if s.is_empty() {
            vec![]
        } else {
            s.split(',').map(String::from).collect()
        };
        Ok(Self { features })
    }
}

/// Common x86_64 CPU features.
pub mod x86_64 {
    pub const SSE: &str = "sse";
    pub const SSE2: &str = "sse2";
    pub const SSE3: &str = "sse3";
    pub const SSSE3: &str = "ssse3";
    pub const SSE4_1: &str = "sse4.1";
    pub const SSE4_2: &str = "sse4.2";
    pub const AVX: &str = "avx";
    pub const AVX2: &str = "avx2";
    pub const AVX512F: &str = "avx512f";
    pub const BMI1: &str = "bmi1";
    pub const BMI2: &str = "bmi2";
    pub const LZCNT: &str = "lzcnt";
    pub const POPCNT: &str = "popcnt";
}

/// Common aarch64 CPU features.
pub mod aarch64 {
    pub const NEON: &str = "neon";
    pub const AES: &str = "aes";
    pub const SHA2: &str = "sha2";
    pub const CRC32: &str = "crc32";
    pub const LSE: &str = "lse";
    pub const FP16: &str = "fp16";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_features_add_has() {
        let mut features = CpuFeatures::new();
        features.add("sse2");
        assert!(features.has("sse2"));
        assert!(!features.has("avx"));
    }

    #[test]
    fn test_cpu_features_with_features() {
        let features = CpuFeatures::with_features(vec!["sse2", "sse3"]);
        assert!(features.has("sse2"));
        assert!(features.has("sse3"));
    }

    #[test]
    fn test_cpu_features_remove() {
        let mut features = CpuFeatures::with_features(vec!["sse2", "avx"]);
        features.remove("avx");
        assert!(features.has("sse2"));
        assert!(!features.has("avx"));
    }

    #[test]
    fn test_cpu_features_serialize() {
        let features = CpuFeatures::with_features(vec!["sse2", "avx"]);
        let json = serde_json::to_string(&features).unwrap();
        let restored: CpuFeatures = serde_json::from_str(&json).unwrap();
        assert!(restored.has("sse2"));
        assert!(restored.has("avx"));
    }

    #[test]
    fn test_cpu_features_empty() {
        let features = CpuFeatures::new();
        assert!(features.is_empty());
        assert_eq!(features.len(), 0);
    }

    #[test]
    fn test_cpu_features_eq() {
        let a = CpuFeatures::with_features(vec!["sse2"]);
        let b = CpuFeatures::with_features(vec!["sse2"]);
        let c = CpuFeatures::with_features(vec!["avx"]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}