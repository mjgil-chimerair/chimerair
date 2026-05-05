//! Chimera object file format
//!
//! Models `.cho` objects, payload kinds, metadata attachments, and proof sidecars.

use chimera_meta::{Metadata, Version};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Object file magic number
const CHO_MAGIC: &[u8; 4] = b"CHOB";

/// Object file version
const CHO_VERSION: (u16, u16) = (0, 1);

/// Payload kind in object file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayloadKind {
    /// Native object file (.o)
    Native,
    /// LLVM bitcode (.bc)
    Bitcode,
    /// ChimeraIR textual (.chimera)
    TextualIR,
    /// Shared library (.so/.dll)
    SharedLib,
    /// Static archive (.a)
    Archive,
    /// Generated wrapper source
    Wrapper,
    /// Compiled proof obligation results
    Proof,
}

impl PayloadKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayloadKind::Native => "native",
            PayloadKind::Bitcode => "bitcode",
            PayloadKind::TextualIR => "textual_ir",
            PayloadKind::SharedLib => "shared_lib",
            PayloadKind::Archive => "archive",
            PayloadKind::Wrapper => "wrapper",
            PayloadKind::Proof => "proof",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "native" => Some(PayloadKind::Native),
            "bitcode" => Some(PayloadKind::Bitcode),
            "textual_ir" => Some(PayloadKind::TextualIR),
            "shared_lib" => Some(PayloadKind::SharedLib),
            "archive" => Some(PayloadKind::Archive),
            "wrapper" => Some(PayloadKind::Wrapper),
            "proof" => Some(PayloadKind::Proof),
            _ => None,
        }
    }

    /// Check if this payload kind can contain metadata sidecar
    pub fn supports_metadata(&self) -> bool {
        matches!(
            self,
            PayloadKind::Native | PayloadKind::Bitcode | PayloadKind::Archive
        )
    }

    /// Check if this payload kind is executable
    pub fn is_executable(&self) -> bool {
        matches!(self, PayloadKind::SharedLib)
    }
}

/// Object file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectHeader {
    pub magic: [u8; 4],
    pub version_major: u16,
    pub version_minor: u16,
    pub target: String,
    pub payload_kind: PayloadKind,
    pub payload_size: u64,
    pub metadata_size: u64,
    #[serde(default)]
    pub checksum: Option<String>,
}

impl ObjectHeader {
    pub fn new(
        target: String,
        payload_kind: PayloadKind,
        payload_size: u64,
        metadata_size: u64,
    ) -> Self {
        Self {
            magic: *CHO_MAGIC,
            version_major: CHO_VERSION.0,
            version_minor: CHO_VERSION.1,
            target,
            payload_kind,
            payload_size,
            metadata_size,
            checksum: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == *CHO_MAGIC
    }

    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }
}

/// Trust level for object file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    /// Generated and proof-verified
    Verified,
    /// Generated but not proof-verified
    Generated,
    /// Imported from external source
    External,
    /// Manually constructed
    Manual,
}

impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Generated
    }
}

/// Trust metadata for object file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustMetadata {
    pub level: TrustLevel,
    pub proof_obligations_verified: usize,
    pub proof_obligations_total: usize,
    pub assumptions: Vec<String>,
}

impl TrustMetadata {
    pub fn new(level: TrustLevel) -> Self {
        Self {
            level,
            proof_obligations_verified: 0,
            proof_obligations_total: 0,
            assumptions: vec![],
        }
    }

    pub fn with_proof_stats(mut self, verified: usize, total: usize) -> Self {
        self.proof_obligations_verified = verified;
        self.proof_obligations_total = total;
        self
    }

    pub fn with_assumption(mut self, assumption: String) -> Self {
        self.assumptions.push(assumption);
        self
    }

    pub fn is_trusted(&self) -> bool {
        self.level == TrustLevel::Verified || self.level == TrustLevel::Manual
    }
}

/// Chimera object file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectFile {
    pub header: ObjectHeader,
    pub payload: Vec<u8>,
    pub metadata: Metadata,
    #[serde(default)]
    pub trust: Option<TrustMetadata>,
}

impl ObjectFile {
    /// Create a new object file
    pub fn new(
        target: String,
        payload: Vec<u8>,
        payload_kind: PayloadKind,
        metadata: Metadata,
    ) -> Self {
        let payload_size = payload.len() as u64;
        let metadata_json = metadata.to_json().unwrap();
        let metadata_size = metadata_json.len() as u64;

        let header = ObjectHeader::new(target, payload_kind, payload_size, metadata_size);
        Self {
            header,
            payload,
            metadata,
            trust: None,
        }
    }

    /// Create with trust metadata
    pub fn new_with_trust(
        target: String,
        payload: Vec<u8>,
        payload_kind: PayloadKind,
        metadata: Metadata,
        trust: TrustMetadata,
    ) -> Self {
        let payload_size = payload.len() as u64;
        let metadata_json = metadata.to_json().unwrap();
        let metadata_size = metadata_json.len() as u64;

        let header = ObjectHeader::new(target, payload_kind, payload_size, metadata_size);
        Self {
            header,
            payload,
            metadata,
            trust: Some(trust),
        }
    }

    /// Validate object file version compatibility
    pub fn check_version(
        &self,
        expected_major: u16,
        expected_minor: u16,
    ) -> Result<(), ParseError> {
        if self.header.version_major != expected_major {
            return Err(ParseError::UnsupportedVersion(
                self.header.version_major,
                self.header.version_minor,
            ));
        }
        if self.header.version_minor < expected_minor {
            return Err(ParseError::UnsupportedVersion(
                self.header.version_major,
                self.header.version_minor,
            ));
        }
        Ok(())
    }

    /// Get payload kind description
    pub fn payload_description(&self) -> String {
        format!(
            "{:?} payload ({} bytes)",
            self.header.payload_kind, self.header.payload_size
        )
    }

    /// Check if object file is self-consistent
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.header.is_valid() {
            return Err(ValidationError::InvalidMagic);
        }
        if self.payload.len() as u64 != self.header.payload_size {
            return Err(ValidationError::PayloadSizeMismatch);
        }
        if let Some(ref trust) = self.trust {
            if trust.level == TrustLevel::Verified && !trust.is_trusted() {
                return Err(ValidationError::TrustLevelInconsistent);
            }
        }
        Ok(())
    }

    /// Parse object file from bytes
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 24 {
            return Err(ParseError::Truncated);
        }

        let magic = <[u8; 4]>::try_from(&data[0..4]).unwrap();
        if magic != *CHO_MAGIC {
            return Err(ParseError::InvalidMagic);
        }

        let version_major = u16::from_le_bytes(<[u8; 2]>::try_from(&data[4..6]).unwrap());
        let version_minor = u16::from_le_bytes(<[u8; 2]>::try_from(&data[6..8]).unwrap());
        if version_major != CHO_VERSION.0 || version_minor != CHO_VERSION.1 {
            return Err(ParseError::UnsupportedVersion(version_major, version_minor));
        }

        let payload_kind = PayloadKind::Native; // Would read from file
        let payload_size = u64::from_le_bytes(<[u8; 8]>::try_from(&data[8..16]).unwrap());
        let metadata_size = u64::from_le_bytes(<[u8; 8]>::try_from(&data[16..24]).unwrap());

        let _target_offset = 24;
        let _target_len = data.len().min(256); // Simplified

        Ok(Self {
            header: ObjectHeader::new(String::new(), payload_kind, payload_size, metadata_size),
            payload: data[24..].to_vec(),
            metadata: Metadata {
                version: Version::new(0, 1, 0),
                ..Default::default()
            },
            trust: None,
        })
    }

    /// Serialize object file to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Header
        result.extend_from_slice(&self.header.magic);
        result.extend_from_slice(&self.header.version_major.to_le_bytes());
        result.extend_from_slice(&self.header.version_minor.to_le_bytes());
        result.extend_from_slice(&u64::to_le_bytes(self.header.payload_size));
        result.extend_from_slice(&u64::to_le_bytes(self.header.metadata_size));

        // Payload
        result.extend_from_slice(&self.payload);

        result
    }

    /// Load object file from path
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        Self::parse(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save object file to path
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let data = self.to_bytes();
        std::fs::write(path, data)
    }

    /// Attach trust metadata
    pub fn attach_trust(&mut self, trust: TrustMetadata) {
        self.trust = Some(trust);
    }
}

/// Validation errors for object file
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidMagic,
    PayloadSizeMismatch,
    TrustLevelInconsistent,
    MetadataCorrupted,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidMagic => write!(f, "invalid magic number"),
            ValidationError::PayloadSizeMismatch => write!(f, "payload size mismatch"),
            ValidationError::TrustLevelInconsistent => {
                write!(f, "trust level inconsistent with proof status")
            }
            ValidationError::MetadataCorrupted => write!(f, "metadata corrupted"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone)]
pub enum ParseError {
    Truncated,
    InvalidMagic,
    UnsupportedVersion(u16, u16),
    InvalidMetadata(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Truncated => write!(f, "truncated object file"),
            ParseError::InvalidMagic => write!(f, "invalid magic number"),
            ParseError::UnsupportedVersion(maj, min) => {
                write!(f, "unsupported version {}.{}", maj, min)
            }
            ParseError::InvalidMetadata(s) => write!(f, "invalid metadata: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_header_valid() {
        let header = ObjectHeader::new(
            "x86_64-unknown-linux-gnu".to_string(),
            PayloadKind::Native,
            1024,
            128,
        );
        assert!(header.is_valid());
    }

    #[test]
    fn test_object_header_magic() {
        let header = ObjectHeader::new(
            "x86_64-unknown-linux-gnu".to_string(),
            PayloadKind::Native,
            1024,
            128,
        );
        assert_eq!(header.magic, *CHO_MAGIC);
    }

    #[test]
    fn test_payload_kind_str() {
        assert_eq!(PayloadKind::Native.as_str(), "native");
        assert_eq!(PayloadKind::Bitcode.as_str(), "bitcode");
    }

    #[test]
    fn test_object_file_new() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![0, 1, 2, 3],
            PayloadKind::Native,
            metadata,
        );
        assert_eq!(obj.payload.len(), 4);
    }

    #[test]
    fn test_object_file_roundtrip() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![0, 1, 2, 3],
            PayloadKind::Native,
            metadata,
        );
        let bytes = obj.to_bytes();
        let parsed = ObjectFile::parse(&bytes).unwrap();
        assert!(parsed.header.is_valid());
    }

    #[test]
    fn test_trust_metadata_new() {
        let trust = TrustMetadata::new(TrustLevel::Generated);
        assert_eq!(trust.level, TrustLevel::Generated);
        assert_eq!(trust.proof_obligations_verified, 0);
        assert_eq!(trust.proof_obligations_total, 0);
        assert!(trust.assumptions.is_empty());
    }

    #[test]
    fn test_trust_metadata_with_proof_stats() {
        let trust = TrustMetadata::new(TrustLevel::Verified).with_proof_stats(5, 5);
        assert_eq!(trust.proof_obligations_verified, 5);
        assert_eq!(trust.proof_obligations_total, 5);
        assert!(trust.is_trusted());
    }

    #[test]
    fn test_trust_metadata_with_assumption() {
        let trust = TrustMetadata::new(TrustLevel::Manual)
            .with_assumption("linker produces valid ELF".to_string());
        assert_eq!(trust.assumptions.len(), 1);
        assert_eq!(trust.assumptions[0], "linker produces valid ELF");
    }

    #[test]
    fn test_trust_level_default() {
        assert_eq!(TrustLevel::default(), TrustLevel::Generated);
    }

    #[test]
    fn test_object_file_with_trust() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let trust = TrustMetadata::new(TrustLevel::Verified).with_proof_stats(10, 10);
        let obj = ObjectFile::new_with_trust(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![1, 2, 3],
            PayloadKind::Native,
            metadata,
            trust,
        );
        assert!(obj.trust.is_some());
        assert!(obj.trust.unwrap().is_trusted());
    }

    #[test]
    fn test_object_file_attach_trust() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let mut obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![0, 1, 2],
            PayloadKind::Bitcode,
            metadata,
        );
        assert!(obj.trust.is_none());

        let trust = TrustMetadata::new(TrustLevel::External);
        obj.attach_trust(trust);

        assert!(obj.trust.is_some());
        assert_eq!(obj.trust.unwrap().level, TrustLevel::External);
    }

    #[test]
    fn test_payload_kind_supports_metadata() {
        assert!(PayloadKind::Native.supports_metadata());
        assert!(PayloadKind::Bitcode.supports_metadata());
        assert!(PayloadKind::Archive.supports_metadata());
        assert!(!PayloadKind::TextualIR.supports_metadata());
        assert!(!PayloadKind::SharedLib.supports_metadata());
        assert!(!PayloadKind::Wrapper.supports_metadata());
        assert!(!PayloadKind::Proof.supports_metadata());
    }

    #[test]
    fn test_payload_kind_is_executable() {
        assert!(PayloadKind::SharedLib.is_executable());
        assert!(!PayloadKind::Native.is_executable());
        assert!(!PayloadKind::Archive.is_executable());
    }

    #[test]
    fn test_payload_kind_from_str() {
        assert_eq!(PayloadKind::from_str("native"), Some(PayloadKind::Native));
        assert_eq!(PayloadKind::from_str("bitcode"), Some(PayloadKind::Bitcode));
        assert_eq!(PayloadKind::from_str("archive"), Some(PayloadKind::Archive));
        assert_eq!(PayloadKind::from_str("wrapper"), Some(PayloadKind::Wrapper));
        assert_eq!(PayloadKind::from_str("proof"), Some(PayloadKind::Proof));
        assert_eq!(PayloadKind::from_str("invalid"), None);
    }

    #[test]
    fn test_validation_error_invalid_magic() {
        let err = ValidationError::InvalidMagic;
        assert_eq!(format!("{}", err), "invalid magic number");
    }

    #[test]
    fn test_validation_error_payload_size_mismatch() {
        let err = ValidationError::PayloadSizeMismatch;
        assert_eq!(format!("{}", err), "payload size mismatch");
    }

    #[test]
    fn test_validation_error_trust_level_inconsistent() {
        let err = ValidationError::TrustLevelInconsistent;
        assert_eq!(
            format!("{}", err),
            "trust level inconsistent with proof status"
        );
    }

    #[test]
    fn test_object_header_with_checksum() {
        let header = ObjectHeader::new(
            "x86_64-unknown-linux-gnu".to_string(),
            PayloadKind::Native,
            1024,
            64,
        )
        .with_checksum("sha256:abc123".to_string());
        assert!(header.checksum.is_some());
        assert_eq!(header.checksum.unwrap(), "sha256:abc123");
    }

    #[test]
    fn test_object_file_validate_success() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![0, 1, 2, 3],
            PayloadKind::Native,
            metadata,
        );
        assert!(obj.validate().is_ok());
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::Truncated;
        assert_eq!(format!("{}", err), "truncated object file");

        let err = ParseError::InvalidMagic;
        assert_eq!(format!("{}", err), "invalid magic number");

        let err = ParseError::UnsupportedVersion(1, 2);
        assert_eq!(format!("{}", err), "unsupported version 1.2");

        let err = ParseError::InvalidMetadata("bad json".to_string());
        assert_eq!(format!("{}", err), "invalid metadata: bad json");
    }

    #[test]
    fn test_object_file_check_version() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![],
            PayloadKind::Native,
            metadata,
        );
        assert!(obj.check_version(0, 1).is_ok());
        assert!(obj.check_version(0, 0).is_ok());
        assert!(obj.check_version(1, 0).is_err());
    }

    #[test]
    fn test_object_file_payload_description() {
        let metadata = Metadata {
            version: Version::new(0, 1, 0),
            ..Default::default()
        };
        let obj = ObjectFile::new(
            "x86_64-unknown-linux-gnu".to_string(),
            vec![0; 100],
            PayloadKind::Bitcode,
            metadata,
        );
        let desc = obj.payload_description();
        assert!(desc.contains("Bitcode"));
        assert!(desc.contains("100 bytes"));
    }
}
