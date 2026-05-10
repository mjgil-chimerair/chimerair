//! Source and debug location preservation for AIR import.
//!
//! Preserves file ID, line/column, byte spans, inline stack,
//! generated/comptime provenance.
//!
//! Task 43: Implement source/debug location preservation.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A source location in Zig source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file ID
    pub file_id: u32,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
    /// Byte span start
    pub span_start: u32,
    /// Byte span end
    pub span_end: u32,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(file_id: u32, line: u32, column: u32, span_start: u32, span_end: u32) -> Self {
        Self {
            file_id,
            line,
            column,
            span_start,
            span_end,
        }
    }

    /// Create an unknown/unavailable location
    pub fn unknown() -> Self {
        Self {
            file_id: 0,
            line: 0,
            column: 0,
            span_start: 0,
            span_end: 0,
        }
    }

    /// Check if this location is unknown
    pub fn is_unknown(&self) -> bool {
        self.file_id == 0 && self.line == 0 && self.column == 0
    }

    /// Check if this location points to generated code (not original source)
    pub fn is_generated(&self) -> bool {
        self.file_id == u32::MAX
    }

    /// Check if this location points to comptime-evaluated code
    pub fn is_comptime(&self) -> bool {
        self.file_id == u32::MAX - 1
    }

    /// Get the span length
    pub fn span_length(&self) -> u32 {
        self.span_end.saturating_sub(self.span_start)
    }

    /// Format as "file:line:col"
    pub fn format_short(&self) -> String {
        format!("{}:{}:{}", self.file_id, self.line, self.column)
    }
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self::unknown()
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file_id, self.line, self.column)
    }
}

/// An inline stack frame
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineFrame {
    /// Function name
    pub function: String,
    /// Location in that function
    pub location: SourceLocation,
}

/// Provenance information for a location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    /// Original Zig source
    Source,
    /// Generated code (e.g., comptime expansion)
    Generated,
    /// Comptime-evaluated code
    Comptime,
    /// Inlined code
    Inlined(Vec<InlineFrame>),
}

/// Full source location with provenance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocationEx {
    /// The location itself
    pub loc: SourceLocation,
    /// Provenance of this location
    pub provenance: Provenance,
}

impl SourceLocationEx {
    /// Create from a basic source location (assumes source provenance)
    pub fn from_location(loc: SourceLocation) -> Self {
        Self {
            loc,
            provenance: Provenance::Source,
        }
    }

    /// Create for generated code
    pub fn generated(loc: SourceLocation) -> Self {
        Self {
            loc,
            provenance: Provenance::Generated,
        }
    }

    /// Create for comptime code
    pub fn comptime(loc: SourceLocation) -> Self {
        Self {
            loc,
            provenance: Provenance::Comptime,
        }
    }

    /// Create for inlined code
    pub fn inlined(loc: SourceLocation, frames: Vec<InlineFrame>) -> Self {
        Self {
            loc,
            provenance: Provenance::Inlined(frames),
        }
    }

    /// Get the underlying location
    pub fn location(&self) -> SourceLocation {
        self.loc
    }

    /// Check if this is an original source location
    pub fn is_source(&self) -> bool {
        matches!(self.provenance, Provenance::Source)
    }

    /// Check if this is generated code
    pub fn is_generated(&self) -> bool {
        matches!(self.provenance, Provenance::Generated)
    }

    /// Check if this is comptime code
    pub fn is_comptime(&self) -> bool {
        matches!(self.provenance, Provenance::Comptime)
    }

    /// Get inline stack depth
    pub fn inline_depth(&self) -> usize {
        match &self.provenance {
            Provenance::Inlined(frames) => frames.len(),
            _ => 0,
        }
    }
}

impl Default for SourceLocationEx {
    fn default() -> Self {
        Self::from_location(SourceLocation::unknown())
    }
}

/// Encoder for preserving source locations during AIR import
#[derive(Debug, Clone)]
pub struct SourceLocationEncoder {
    file_mapping: std::collections::HashMap<u32, String>,
    next_file_id: u32,
}

impl SourceLocationEncoder {
    /// Create a new encoder
    pub fn new() -> Self {
        Self {
            file_mapping: std::collections::HashMap::new(),
            next_file_id: 1,
        }
    }

    /// Register a file path and get its ID
    pub fn register_file(&mut self, path: &str) -> u32 {
        for (id, p) in &self.file_mapping {
            if p == path {
                return *id;
            }
        }
        let id = self.next_file_id;
        self.next_file_id += 1;
        self.file_mapping.insert(id, path.to_string());
        id
    }

    /// Get file path for an ID
    pub fn get_file(&self, file_id: u32) -> Option<&str> {
        self.file_mapping.get(&file_id).map(|s| s.as_str())
    }

    /// Encode a location from raw values
    pub fn encode(&self, file_id: u32, line: u32, col: u32, start: u32, end: u32) -> SourceLocation {
        SourceLocation::new(file_id, line, col, start, end)
    }

    /// Encode with provenance
    pub fn encode_with_provenance(
        &self,
        file_id: u32,
        line: u32,
        col: u32,
        start: u32,
        end: u32,
        provenance: Provenance,
    ) -> SourceLocationEx {
        SourceLocationEx {
            loc: self.encode(file_id, line, col, start, end),
            provenance,
        }
    }

    /// Get number of registered files
    pub fn file_count(&self) -> usize {
        self.file_mapping.len()
    }
}

impl Default for SourceLocationEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_creation() {
        let loc = SourceLocation::new(1, 10, 5, 100, 110);
        assert_eq!(loc.file_id, 1);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.span_length(), 10);
    }

    #[test]
    fn test_source_location_unknown() {
        let loc = SourceLocation::unknown();
        assert!(loc.is_unknown());
    }

    #[test]
    fn test_source_location_generated() {
        let mut loc = SourceLocation::unknown();
        loc.file_id = u32::MAX;
        assert!(loc.is_generated());
    }

    #[test]
    fn test_source_location_comptime() {
        let mut loc = SourceLocation::unknown();
        loc.file_id = u32::MAX - 1;
        assert!(loc.is_comptime());
    }

    #[test]
    fn test_source_location_format_short() {
        let loc = SourceLocation::new(1, 10, 5, 100, 110);
        assert_eq!(loc.format_short(), "1:10:5");
    }

    #[test]
    fn test_source_location_display() {
        let loc = SourceLocation::new(1, 10, 5, 100, 110);
        assert_eq!(format!("{}", loc), "1:10:5");
    }

    #[test]
    fn test_source_location_ex_from_location() {
        let loc = SourceLocation::new(1, 10, 5, 100, 110);
        let ex = SourceLocationEx::from_location(loc);
        assert!(ex.is_source());
        assert!(!ex.is_generated());
    }

    #[test]
    fn test_source_location_ex_generated() {
        let loc = SourceLocation::new(u32::MAX, 0, 0, 0, 0);
        let ex = SourceLocationEx::generated(loc);
        assert!(ex.is_generated());
    }

    #[test]
    fn test_source_location_ex_comptime() {
        let loc = SourceLocation::new(u32::MAX - 1, 0, 0, 0, 0);
        let ex = SourceLocationEx::comptime(loc);
        assert!(ex.is_comptime());
    }

    #[test]
    fn test_source_location_ex_inlined() {
        let loc = SourceLocation::new(1, 10, 5, 100, 110);
        let frames = vec![
            InlineFrame {
                function: "foo".to_string(),
                location: SourceLocation::new(1, 5, 1, 50, 60),
            },
        ];
        let ex = SourceLocationEx::inlined(loc, frames);
        assert_eq!(ex.inline_depth(), 1);
    }

    #[test]
    fn test_source_location_encoder_register() {
        let mut encoder = SourceLocationEncoder::new();
        let id1 = encoder.register_file("foo.zig");
        let id2 = encoder.register_file("bar.zig");
        assert_ne!(id1, id2);
        assert_eq!(encoder.get_file(id1), Some("foo.zig"));
        assert_eq!(encoder.get_file(id2), Some("bar.zig"));
    }

    #[test]
    fn test_source_location_encoder_duplicate() {
        let mut encoder = SourceLocationEncoder::new();
        let id1 = encoder.register_file("foo.zig");
        let id2 = encoder.register_file("foo.zig");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_source_location_encoder_encode() {
        let encoder = SourceLocationEncoder::new();
        let loc = encoder.encode(1, 10, 5, 100, 110);
        assert_eq!(loc.file_id, 1);
        assert_eq!(loc.line, 10);
    }

    #[test]
    fn test_source_location_encoder_encode_with_provenance() {
        let encoder = SourceLocationEncoder::new();
        let ex = encoder.encode_with_provenance(1, 10, 5, 100, 110, Provenance::Source);
        assert!(ex.is_source());
    }
}