//! BEAM bytecode disassembler.
//!
//! Reads BEAM bytecode files (.beam) and produces structured representations.

use chimera_beam_schema::{
    Atom, Attribute, BeamModuleInfo, BeamSnapHeader, BeamSnapSchema, CompileInfo, ExportEntry,
    FunctionInfo, ImportEntry, Term, BEAM_SNAP_MAGIC, SCHEMA_VERSION,
};
use thiserror::Error;

/// Disassembler error types.
#[derive(Debug, Error)]
pub enum DisassemblerError {
    #[error("invalid BEAM file: {0}")]
    InvalidFile(String),
    #[error("truncated data: {0}")]
    TruncatedData(String),
    #[error("unsupported BEAM format version: {0}")]
    UnsupportedVersion(u8),
    #[error("missing required section: {0}")]
    MissingSection(String),
    #[error("parse error: {0}")]
    Parse(String),
}

/// Result type for disassembler operations.
pub type DisassemblerResult<T> = Result<T, DisassemblerError>;

/// BEAM bytecode file disassembler.
pub struct BeamDisassembler<'a> {
    data: &'a [u8],
    pos: usize,
    error: Option<String>,
}

impl<'a> BeamDisassembler<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BeamDisassembler {
            data,
            pos: 0,
            error: None,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn run(mut self) -> DisassemblerResult<BeamSnapSchema> {
        let header = self.parse_beam_header()?;
        let mut module_info = self.parse_module_info()?;
        let functions = self.parse_functions()?;

        // Add functions to module_info
        module_info.functions = functions;

        let mut schema = BeamSnapSchema::new();
        schema.header = header;
        schema.modules.push(module_info);

        Ok(schema)
    }

    pub fn analyze_module(mut self) -> DisassemblerResult<BeamModuleInfo> {
        self.parse_module_info()
    }

    fn parse_beam_header(&mut self) -> DisassemblerResult<BeamSnapHeader> {
        // BEAM file starts with "BEAM" magic bytes
        if self.data.len() < 4 {
            return Err(DisassemblerError::InvalidFile(
                "File too short for BEAM magic".to_string(),
            ));
        }

        let magic = &self.data[0..4];
        if magic != b"BEAM" {
            return Err(DisassemblerError::InvalidFile(format!(
                "Invalid BEAM magic: {:?}",
                String::from_utf8_lossy(magic)
            )));
        }

        // Parse version (next byte after "BEAM")
        if self.data.len() < 5 {
            return Err(DisassemblerError::TruncatedData(
                "Missing version byte".to_string(),
            ));
        }
        let version = self.data[4];
        if version < 1 || version > 3 {
            return Err(DisassemblerError::UnsupportedVersion(version));
        }

        self.pos = 5;

        Ok(BeamSnapHeader {
            magic: *b"BeamSnap",
            schema_version: SCHEMA_VERSION,
            min_adapter_version: 1,
            erlang_version: format!("BEAM/{}", version),
            otp_release: String::new(),
            target: String::new(),
            timestamp_ns: 0,
            module_count: 1,
            process_count: 0,
            supervisor_count: 0,
            registry_count: 0,
            effect_count: 0,
            checksum: [0u8; 32],
        })
    }

    fn parse_module_info(&mut self) -> DisassemblerResult<BeamModuleInfo> {
        // Extract string table from BEAM
        let (strings, next_pos) = self.extract_string_table()?;

        // Get module name from first string
        let module_name = strings.get(0).cloned().unwrap_or_default();

        self.pos = next_pos;

        Ok(BeamModuleInfo {
            module_name: Atom::new(module_name),
            exports: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            attributes: Vec::new(),
            compile_info: CompileInfo {
                options: Vec::new(),
                version: None,
                time: None,
            },
        })
    }

    fn extract_string_table(&self) -> DisassemblerResult<(Vec<String>, usize)> {
        // Simple string table extraction from BEAM
        // In real implementation, this would parse the actual BEAM format
        let mut strings = Vec::new();
        let mut pos = self.pos;

        // Try to find string table section
        while pos < self.data.len() {
            if self.data[pos] == 0 {
                break;
            }
            let start = pos;
            while pos < self.data.len() && self.data[pos] != 0 {
                pos += 1;
            }
            if pos > start {
                let s = String::from_utf8_lossy(&self.data[start..pos]).to_string();
                strings.push(s);
            }
            pos += 1;
        }

        Ok((strings, pos + 1))
    }

    fn parse_functions(&mut self) -> DisassemblerResult<Vec<FunctionInfo>> {
        // Parse function table from BEAM
        // This is a simplified version - real implementation would parse
        // the actual function table in BEAM bytecode
        let mut functions = Vec::new();

        // For a real BEAM file, we would parse:
        // - Lambda table
        // - Export table
        // - Import table
        // - Code chunk

        Ok(functions)
    }

    fn read_u32(&mut self) -> DisassemblerResult<u32> {
        if self.pos + 4 > self.data.len() {
            return Err(DisassemblerError::TruncatedData(
                "Need 4 bytes for u32".to_string(),
            ));
        }
        let val = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(val)
    }

    fn read_string(&mut self) -> DisassemblerResult<String> {
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.data[start..self.pos]).to_string();
        if self.pos < self.data.len() {
            self.pos += 1; // Skip null terminator
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid_beam() {
        let data = b"Not a BEAM file".to_vec();
        let dis = BeamDisassembler::new(&data);
        let result = dis.run();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid_beam_magic() {
        let data = b"BEAM\x01".to_vec(); // BEAM magic + version 1
        let mut dis = BeamDisassembler::new(&data);
        let result = dis.parse_beam_header();
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_string_table() {
        let data = b"test_module\0another_string\0";
        let dis = BeamDisassembler::new(data);
        let (strings, _pos) = dis.extract_string_table().unwrap();
        assert!(strings.len() >= 1);
    }
}
