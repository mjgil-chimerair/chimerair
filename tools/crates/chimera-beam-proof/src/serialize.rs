//! Proof artifact serialization.
//!
//! Serializes and deserializes proof artifacts to/from various formats.

use serde::{Deserialize, Serialize};
use serde_json;
use std::path::Path;

use super::emitter::ProofArtifact;
use super::fact::ProofFact;

/// Proof artifact header for serialized format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofHeader {
    /// Magic bytes.
    pub magic: [u8; 8],
    /// Format version.
    pub version: u32,
    /// Module fingerprint.
    pub module_fingerprint: u32,
    /// Module name length.
    pub module_name_len: u32,
    /// Module name.
    pub module_name: Option<String>,
    /// Number of facts.
    pub fact_count: u32,
    /// Artifact hash.
    pub artifact_hash: String,
    /// Generation timestamp.
    pub generated_at: u64,
}

/// Magic bytes for proof artifact files.
pub const PROOF_MAGIC: [u8; 8] = *b"BeamProf";

/// Current serialization format version.
pub const SERIALIZE_VERSION: u32 = 1;

/// Serialize a proof artifact to JSON bytes.
pub fn serialize_proof(artifact: &ProofArtifact) -> Result<Vec<u8>, SerializeError> {
    let json =
        serde_json::to_string(artifact).map_err(|e| SerializeError::JsonError(e.to_string()))?;

    Ok(json.into_bytes())
}

/// Serialize a proof artifact to a file.
pub fn serialize_proof_to_file(
    artifact: &ProofArtifact,
    path: &Path,
) -> Result<(), SerializeError> {
    let bytes = serialize_proof(artifact)?;
    std::fs::write(path, &bytes).map_err(|e| SerializeError::IoError(e.to_string()))?;

    Ok(())
}

/// Deserialize a proof artifact from JSON bytes.
pub fn deserialize_proof(bytes: &[u8]) -> Result<ProofArtifact, SerializeError> {
    serde_json::from_slice(bytes).map_err(|e| SerializeError::JsonError(e.to_string()))
}

/// Deserialize a proof artifact from a file.
pub fn deserialize_proof_from_file(path: &Path) -> Result<ProofArtifact, SerializeError> {
    let bytes = std::fs::read(path).map_err(|e| SerializeError::IoError(e.to_string()))?;

    deserialize_proof(&bytes)
}

/// Serialize facts to JSON bytes.
pub fn serialize_facts(facts: &[ProofFact]) -> Result<Vec<u8>, SerializeError> {
    let json =
        serde_json::to_string(facts).map_err(|e| SerializeError::JsonError(e.to_string()))?;

    Ok(json.into_bytes())
}

/// Deserialize facts from JSON bytes.
pub fn deserialize_facts(bytes: &[u8]) -> Result<Vec<ProofFact>, SerializeError> {
    serde_json::from_slice(bytes).map_err(|e| SerializeError::JsonError(e.to_string()))
}

/// Serialize in compact binary format.
pub fn serialize_proof_binary(artifact: &ProofArtifact) -> Result<Vec<u8>, SerializeError> {
    let mut result = Vec::new();

    // Write magic
    result.extend_from_slice(&PROOF_MAGIC);

    // Write version
    result.extend_from_slice(&SERIALIZE_VERSION.to_le_bytes());

    // Write module fingerprint
    result.extend_from_slice(&artifact.module_fingerprint.to_le_bytes());

    // Write module name
    let module_name = artifact.module_name.as_ref();
    let module_name_bytes = module_name.map(|n| n.as_bytes()).unwrap_or(&[]);
    result.extend_from_slice(&(module_name_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(module_name_bytes);

    // Write fact count
    result.extend_from_slice(&(artifact.facts.len() as u32).to_le_bytes());

    // Write artifact hash
    let hash_bytes = artifact.artifact_hash.as_bytes();
    result.extend_from_slice(&(hash_bytes.len() as u32).to_le_bytes());
    result.extend_from_slice(hash_bytes);

    // Write timestamp
    result.extend_from_slice(&artifact.generated_at.to_le_bytes());

    // Write facts as JSON
    let facts_json = serde_json::to_vec(&artifact.facts)
        .map_err(|e| SerializeError::JsonError(e.to_string()))?;
    result.extend_from_slice(&(facts_json.len() as u32).to_le_bytes());
    result.extend_from_slice(&facts_json);

    Ok(result)
}

/// Deserialize from compact binary format.
pub fn deserialize_proof_binary(bytes: &[u8]) -> Result<ProofArtifact, SerializeError> {
    let mut cursor = bytes;

    // Read and verify magic
    let magic: [u8; 8] = cursor[..8]
        .try_into()
        .map_err(|_| SerializeError::InvalidFormat("Missing magic".to_string()))?;
    if magic != PROOF_MAGIC {
        return Err(SerializeError::InvalidFormat("Invalid magic".to_string()));
    }
    cursor = &cursor[8..];

    // Read version
    let version = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing version".to_string()))?,
    );
    if version != SERIALIZE_VERSION {
        return Err(SerializeError::InvalidFormat(
            "Unsupported version".to_string(),
        ));
    }
    cursor = &cursor[4..];

    // Read module fingerprint
    let module_fingerprint = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing fingerprint".to_string()))?,
    );
    cursor = &cursor[4..];

    // Read module name
    let module_name_len = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing module name len".to_string()))?,
    );
    cursor = &cursor[4..];

    let module_name = if module_name_len > 0 {
        let name_end = 4 + module_name_len as usize;
        let name_bytes = &cursor[..module_name_len as usize];
        let name = String::from_utf8(name_bytes.to_vec())
            .map_err(|_| SerializeError::InvalidFormat("Invalid module name".to_string()))?;
        cursor = &cursor[name_end..];
        Some(name)
    } else {
        None
    };

    // Read fact count
    let fact_count = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing fact count".to_string()))?,
    );
    cursor = &cursor[4..];

    // Read artifact hash
    let hash_len = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing hash len".to_string()))?,
    );
    cursor = &cursor[4..];

    let hash_end = 4 + hash_len as usize;
    let artifact_hash = String::from_utf8(cursor[..hash_len as usize].to_vec())
        .map_err(|_| SerializeError::InvalidFormat("Invalid hash".to_string()))?;
    cursor = &cursor[hash_end..];

    // Read timestamp
    let generated_at = u64::from_le_bytes(
        cursor[..8]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing timestamp".to_string()))?,
    );
    cursor = &cursor[8..];

    // Read facts JSON
    let facts_json_len = u32::from_le_bytes(
        cursor[..4]
            .try_into()
            .map_err(|_| SerializeError::InvalidFormat("Missing facts len".to_string()))?,
    );
    cursor = &cursor[4..];

    let facts: Vec<ProofFact> = serde_json::from_slice(&cursor[..facts_json_len as usize])
        .map_err(|e| SerializeError::JsonError(e.to_string()))?;

    Ok(ProofArtifact {
        version: SERIALIZE_VERSION,
        module_fingerprint,
        module_name,
        facts,
        artifact_hash,
        generated_at,
    })
}

/// Serialization error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializeError {
    /// JSON serialization error.
    JsonError(String),
    /// I/O error.
    IoError(String),
    /// Invalid format error.
    InvalidFormat(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::JsonError(s) => write!(f, "JSON error: {}", s),
            SerializeError::IoError(s) => write!(f, "I/O error: {}", s),
            SerializeError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
        }
    }
}

impl std::error::Error for SerializeError {}

#[cfg(test)]
mod tests {
    use super::super::emitter::ProofEmitter;
    use super::super::fact::{FactId, ProofKind, ProofTarget};
    use super::*;

    fn create_test_artifact() -> ProofArtifact {
        let mut emitter = ProofEmitter::new(0x1234);
        emitter.emit_memory_safety(ProofTarget::Module("test_mod".to_string()), "Heap is valid");

        ProofArtifact::from_emitter(&emitter, Some("test_mod".to_string()), 1000)
    }

    #[test]
    fn test_serialize_proof() {
        let artifact = create_test_artifact();
        let bytes = serialize_proof(&artifact).unwrap();

        assert!(!bytes.is_empty());
        assert!(bytes.len() > 100);
    }

    #[test]
    fn test_deserialize_proof() {
        let artifact = create_test_artifact();
        let bytes = serialize_proof(&artifact).unwrap();
        let deserialized = deserialize_proof(&bytes).unwrap();

        assert_eq!(deserialized.module_fingerprint, artifact.module_fingerprint);
        assert_eq!(deserialized.module_name, artifact.module_name);
        assert_eq!(deserialized.fact_count(), artifact.fact_count());
    }

    #[test]
    fn test_serialize_deserialize_facts() {
        let artifact = create_test_artifact();
        let bytes = serialize_facts(&artifact.facts).unwrap();
        let deserialized = deserialize_facts(&bytes).unwrap();

        assert_eq!(deserialized.len(), artifact.facts.len());
    }

    #[test]
    fn test_serialize_proof_binary() {
        let artifact = create_test_artifact();
        let bytes = serialize_proof_binary(&artifact).unwrap();

        // Should have magic (8) + version (4) + fingerprint (4) + module_name_len (4) + name + fact_count (4) + hash_len (4) + hash + timestamp (8) + facts_len (4) + facts
        assert!(bytes.len() > 50);
    }

    #[test]
    fn test_deserialize_proof_binary_invalid_magic() {
        let bytes = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let result = deserialize_proof_binary(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_proof_binary_invalid_version() {
        let mut bytes = PROOF_MAGIC.to_vec();
        bytes.extend_from_slice(&9999u32.to_le_bytes()); // Invalid version
        let result = deserialize_proof_binary(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_empty_artifact() {
        let emitter = ProofEmitter::new(0xABCD);
        let artifact = ProofArtifact::from_emitter(&emitter, None, 0);
        let bytes = serialize_proof(&artifact).unwrap();
        let deserialized = deserialize_proof(&bytes).unwrap();

        assert_eq!(deserialized.fact_count(), 0);
    }

    #[test]
    fn test_proof_header_magic() {
        assert_eq!(PROOF_MAGIC, *b"BeamProf");
    }

    #[test]
    fn test_serialize_error_display() {
        let err = SerializeError::JsonError("test".to_string());
        assert_eq!(format!("{}", err), "JSON error: test");

        let err = SerializeError::IoError("test".to_string());
        assert_eq!(format!("{}", err), "I/O error: test");

        let err = SerializeError::InvalidFormat("test".to_string());
        assert_eq!(format!("{}", err), "Invalid format: test");
    }
}
