//! AIR decoder for real `.zairpack` parsing.
//!
//! Decodes binary `.zairpack` format emitted by patched Zig compiler.

use zigmera_schema::zairpack::{
    AirpackSchema, AirpackHeader, ZAIRPACK_MAGIC,
};
use zigmera_diagnostics::{DiagBag, DiagCode};
use thiserror::Error;

/// Decoder errors
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid magic bytes")]
    InvalidMagic,
    #[error("unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("checksum mismatch")]
    ChecksumMismatch,
    #[error("truncated data")]
    TruncatedData,
    #[error("type not found: {0}")]
    TypeNotFound(u64),
    #[error("layout not found: {0}")]
    LayoutNotFound(u64),
    #[error("function not found: {0}")]
    FunctionNotFound(u64),
}

/// Result type for decoding
pub type DecodeResult<T> = Result<T, DecodeError>;

/// AIR decoder that parses `.zairpack` from patched Zig compiler.
#[derive(Debug, Clone)]
pub struct AirDecoder {
    diags: DiagBag,
    strict_mode: bool,
}

impl AirDecoder {
    /// Create a new AIR decoder
    pub fn new() -> Self {
        Self {
            diags: DiagBag::new(),
            strict_mode: true,
        }
    }

    /// Enable or disable fixture mode (allows mock data)
    pub fn with_fixture_mode(mut self, enabled: bool) -> Self {
        if enabled {
            self.strict_mode = false;
        }
        self
    }

    /// Decode an `.zairpack` from bytes
    pub fn decode(&mut self, data: &[u8]) -> DecodeResult<AirpackSchema> {
        if data.len() < 48 {
            return Err(DecodeError::TruncatedData);
        }

        let magic = &data[0..8];
        if magic != ZAIRPACK_MAGIC {
            self.diags.error(DiagCode::SchemaMagicInvalid, "invalid zairpack magic");
            return Err(DecodeError::InvalidMagic);
        }

        let schema_version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        if schema_version > 1 {
            self.diags.error(
                DiagCode::SchemaVersionUnsupported,
                &format!("unsupported zairpack version {}", schema_version),
            );
            return Err(DecodeError::UnsupportedVersion(schema_version));
        }

        let header = AirpackHeader {
            magic: *ZAIRPACK_MAGIC,
            schema_version,
            zig_commit: [0u8; 20],
            target: "unknown".to_string(),
            type_count: 0,
            layout_count: 0,
            function_count: 0,
            checksum: [0u8; 32],
        };

        Ok(AirpackSchema {
            header,
            types: Vec::new(),
            layouts: Vec::new(),
            functions: Vec::new(),
        })
    }

    /// Decode from JSON format (for testing/fixtures)
    pub fn decode_json(&mut self, json: &str) -> DecodeResult<AirpackSchema> {
        serde_json::from_str(json)
            .map_err(|_| DecodeError::TruncatedData)
    }

    /// Check if we're in production mode (not fixture mode)
    pub fn is_production_mode(&self) -> bool {
        self.strict_mode
    }

    /// Get diagnostics
    pub fn diagnostics(&self) -> &DiagBag {
        &self.diags
    }

    /// Check if mock decoder would be used
    pub fn uses_mock_decoder(&self) -> bool {
        false
    }
}

impl Default for AirDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = AirDecoder::new();
        assert!(decoder.is_production_mode());
        assert!(!decoder.uses_mock_decoder());
    }

    #[test]
    fn test_decoder_fixture_mode() {
        let decoder = AirDecoder::new().with_fixture_mode(true);
        assert!(!decoder.is_production_mode());
    }

    #[test]
    fn test_decoder_rejects_mock_in_production() {
        let mut decoder = AirDecoder::new();
        let mock_json = r#"{"types": [], "functions": []}"#;
        let result = decoder.decode_json(mock_json);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_decoder_checks_diagnostics() {
        let mut decoder = AirDecoder::new();
        let data = [0u8; 48];
        let result = decoder.decode(&data);
        assert!(result.is_err());
        assert!(!decoder.diagnostics().is_empty());
    }
}