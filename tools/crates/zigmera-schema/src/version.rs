//! Schema versioning for Zigmera artifacts.

use serde::{Deserialize, Serialize};

/// Current schema version.
pub const CURRENT_VERSION: u32 = 1;

/// Schema version information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn current() -> Self {
        Self::new(0, 1, 0)
    }

    pub fn encode_u32(&self) -> u32 {
        (self.major << 16) | (self.minor << 8) | self.patch
    }

    pub fn decode_u32(val: u32) -> Self {
        Self {
            major: (val >> 16) & 0xFFFF,
            minor: (val >> 8) & 0xFF,
            patch: val & 0xFF,
        }
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
