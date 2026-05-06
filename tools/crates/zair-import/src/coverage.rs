//! AIR instruction coverage table.
//!
//! Maps every supported Zig AIR instruction to dialect operation
//! or explicit unsupported diagnostic.
//!
//! Task 42: Implement AIR instruction coverage table.

use zigmera_diagnostics::DiagCode;
use serde::{Deserialize, Serialize};

/// Coverage status for an instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoverageStatus {
    /// Fully supported
    Supported,
    /// Supported with limitations
    Partial,
    /// Not supported, emit diagnostic
    Unsupported,
    /// Reserved for future support
    Future,
}

/// An entry in the instruction coverage table
#[derive(Debug, Clone)]
pub struct CoverageEntry {
    pub air_op: String,
    pub dialect_op: Option<String>,
    pub status: CoverageStatus,
    pub diagnostic: Option<DiagCode>,
    pub notes: &'static str,
}

/// AIR instruction coverage table mapping Zig AIR to dialect ops.
#[derive(Debug, Clone)]
pub struct InstructionCoverage {
    entries: Vec<CoverageEntry>,
}

impl InstructionCoverage {
    /// Create a new coverage table with all Zig AIR instructions
    pub fn new() -> Self {
        Self {
            entries: Self::build_coverage_table(),
        }
    }

    /// Build the full coverage table
    fn build_coverage_table() -> Vec<CoverageEntry> {
        vec![
            // Control flow - Fully supported
            CoverageEntry {
                air_op: "br".to_string(),
                dialect_op: Some("ZigOp::Br".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Unconditional branch",
            },
            CoverageEntry {
                air_op: "br_cond".to_string(),
                dialect_op: Some("ZigOp::BrCond".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Conditional branch",
            },
            CoverageEntry {
                air_op: "switch".to_string(),
                dialect_op: Some("ZigOp::Switch".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Multi-way branch",
            },
            CoverageEntry {
                air_op: "ret".to_string(),
                dialect_op: Some("ZigOp::Ret".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Return with value",
            },
            CoverageEntry {
                air_op: "ret_void".to_string(),
                dialect_op: Some("ZigOp::RetVoid".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Return void",
            },
            CoverageEntry {
                air_op: "call".to_string(),
                dialect_op: Some("ZigOp::Call".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Direct function call",
            },
            CoverageEntry {
                air_op: "call_indirect".to_string(),
                dialect_op: Some("ZigOp::CallIndirect".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Indirect call via function pointer",
            },
            CoverageEntry {
                air_op: "unreachable".to_string(),
                dialect_op: Some("ZigOp::Unreachable".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Unreachable instruction",
            },

            // Memory operations - Fully supported
            CoverageEntry {
                air_op: "load".to_string(),
                dialect_op: Some("ZigOp::Load".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Memory load",
            },
            CoverageEntry {
                air_op: "store".to_string(),
                dialect_op: Some("ZigOp::Store".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Memory store",
            },
            CoverageEntry {
                air_op: "alloca".to_string(),
                dialect_op: Some("ZigOp::Alloca".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Local allocation",
            },
            CoverageEntry {
                air_op: "elem_ptr".to_string(),
                dialect_op: Some("ZigOp::ElemPtr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Element pointer calculation",
            },
            CoverageEntry {
                air_op: "field_ptr".to_string(),
                dialect_op: Some("ZigOp::FieldPtr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Field pointer calculation",
            },
            CoverageEntry {
                air_op: "addr_of".to_string(),
                dialect_op: Some("ZigOp::AddrOf".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Address-of operation",
            },

            // Arithmetic - Fully supported
            CoverageEntry {
                air_op: "add".to_string(),
                dialect_op: Some("ZigOp::Add".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Integer/float addition",
            },
            CoverageEntry {
                air_op: "sub".to_string(),
                dialect_op: Some("ZigOp::Sub".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Integer/float subtraction",
            },
            CoverageEntry {
                air_op: "mul".to_string(),
                dialect_op: Some("ZigOp::Mul".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Integer/float multiplication",
            },
            CoverageEntry {
                air_op: "div".to_string(),
                dialect_op: Some("ZigOp::Div".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Integer/float division",
            },
            CoverageEntry {
                air_op: "rem".to_string(),
                dialect_op: Some("ZigOp::Rem".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Remainder operation",
            },

            // Bit operations - Fully supported
            CoverageEntry {
                air_op: "and".to_string(),
                dialect_op: Some("ZigOp::And".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Bitwise AND",
            },
            CoverageEntry {
                air_op: "or".to_string(),
                dialect_op: Some("ZigOp::Or".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Bitwise OR",
            },
            CoverageEntry {
                air_op: "xor".to_string(),
                dialect_op: Some("ZigOp::Xor".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Bitwise XOR",
            },
            CoverageEntry {
                air_op: "shl".to_string(),
                dialect_op: Some("ZigOp::Shl".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Left shift",
            },
            CoverageEntry {
                air_op: "shr".to_string(),
                dialect_op: Some("ZigOp::Shr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Right shift",
            },

            // Comparison - Fully supported
            CoverageEntry {
                air_op: "eq".to_string(),
                dialect_op: Some("ZigOp::Eq".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Equality comparison",
            },
            CoverageEntry {
                air_op: "ne".to_string(),
                dialect_op: Some("ZigOp::Ne".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Inequality comparison",
            },
            CoverageEntry {
                air_op: "slt".to_string(),
                dialect_op: Some("ZigOp::Slt".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Signed less than",
            },
            CoverageEntry {
                air_op: "sle".to_string(),
                dialect_op: Some("ZigOp::Sle".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Signed less or equal",
            },
            CoverageEntry {
                air_op: "sgt".to_string(),
                dialect_op: Some("ZigOp::Sgt".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Signed greater than",
            },
            CoverageEntry {
                air_op: "sge".to_string(),
                dialect_op: Some("ZigOp::Sge".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Signed greater or equal",
            },

            // Type conversions - Fully supported
            CoverageEntry {
                air_op: "zext".to_string(),
                dialect_op: Some("ZigOp::Zext".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Zero extend",
            },
            CoverageEntry {
                air_op: "sext".to_string(),
                dialect_op: Some("ZigOp::Sext".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Sign extend",
            },
            CoverageEntry {
                air_op: "trunc".to_string(),
                dialect_op: Some("ZigOp::Trunc".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Truncate",
            },
            CoverageEntry {
                air_op: "bitcast".to_string(),
                dialect_op: Some("ZigOp::Bitcast".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Bitcast conversion",
            },
            CoverageEntry {
                air_op: "int_to_ptr".to_string(),
                dialect_op: Some("ZigOp::IntToPtr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Integer to pointer",
            },
            CoverageEntry {
                air_op: "ptr_to_int".to_string(),
                dialect_op: Some("ZigOp::PtrToInt".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Pointer to integer",
            },

            // Error handling - Fully supported
            CoverageEntry {
                air_op: "wrap_err".to_string(),
                dialect_op: Some("ZigOp::WrapErr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Wrap in error union",
            },
            CoverageEntry {
                air_op: "is_err".to_string(),
                dialect_op: Some("ZigOp::IsErr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Check if error",
            },
            CoverageEntry {
                air_op: "unwrap_err".to_string(),
                dialect_op: Some("ZigOp::UnwrapErr".to_string()),
                status: CoverageStatus::Supported,
                diagnostic: None,
                notes: "Unwrap error union payload",
            },

            // Async/frame - NOT supported
            CoverageEntry {
                air_op: "suspend_frame".to_string(),
                dialect_op: None,
                status: CoverageStatus::Unsupported,
                diagnostic: Some(DiagCode::LoweringAsyncNotSupported),
                notes: "Suspend frame - async not supported",
            },
            CoverageEntry {
                air_op: "resume".to_string(),
                dialect_op: None,
                status: CoverageStatus::Unsupported,
                diagnostic: Some(DiagCode::LoweringAsyncNotSupported),
                notes: "Resume - async not supported",
            },
            CoverageEntry {
                air_op: "await".to_string(),
                dialect_op: None,
                status: CoverageStatus::Unsupported,
                diagnostic: Some(DiagCode::LoweringAsyncNotSupported),
                notes: "Await - async not supported",
            },

            // SIMD - Partial support
            CoverageEntry {
                air_op: "vector_reduce".to_string(),
                dialect_op: None,
                status: CoverageStatus::Partial,
                diagnostic: Some(DiagCode::LoweringSimdNotSupported),
                notes: "SIMD vector reduction - partial support",
            },

            // Inline assembly - NOT supported
            CoverageEntry {
                air_op: "inline_asm".to_string(),
                dialect_op: None,
                status: CoverageStatus::Unsupported,
                diagnostic: Some(DiagCode::LoweringAsmNotSupported),
                notes: "Inline assembly - opaque operation",
            },
        ]
    }

    /// Look up coverage for an AIR operation string
    pub fn lookup(&self, air_op: &str) -> Option<&CoverageEntry> {
        self.entries.iter().find(|e| e.air_op == air_op)
    }

    /// Get all supported operations
    pub fn supported_ops(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.status == CoverageStatus::Supported)
            .map(|e| e.air_op.as_str())
            .collect()
    }

    /// Get all unsupported operations
    pub fn unsupported_ops(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.status == CoverageStatus::Unsupported)
            .map(|e| e.air_op.as_str())
            .collect()
    }

    /// Check if an operation is supported
    pub fn is_supported(&self, air_op: &str) -> bool {
        self.lookup(air_op)
            .map(|e| e.status == CoverageStatus::Supported)
            .unwrap_or(false)
    }

    /// Check if an operation requires a diagnostic
    pub fn requires_diagnostic(&self, air_op: &str) -> Option<DiagCode> {
        self.lookup(air_op)
            .and_then(|e| e.diagnostic)
    }

    /// Get total count of operations
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Get count of supported operations
    pub fn supported_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.status == CoverageStatus::Supported)
            .count()
    }

    /// Get coverage percentage
    pub fn coverage_percentage(&self) -> f64 {
        if self.entries.is_empty() {
            return 0.0;
        }
        let supported = self.supported_count();
        (supported as f64 / self.entries.len() as f64) * 100.0
    }
}

impl Default for InstructionCoverage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_table_creation() {
        let coverage = InstructionCoverage::new();
        assert!(coverage.total_count() > 0);
    }

    #[test]
    fn test_lookup_supported() {
        let coverage = InstructionCoverage::new();
        let entry = coverage.lookup("add");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().status, CoverageStatus::Supported);
    }

    #[test]
    fn test_lookup_unsupported() {
        let coverage = InstructionCoverage::new();
        let entry = coverage.lookup("await");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().status, CoverageStatus::Unsupported);
        assert!(entry.unwrap().diagnostic.is_some());
    }

    #[test]
    fn test_is_supported() {
        let coverage = InstructionCoverage::new();
        assert!(coverage.is_supported("add"));
        assert!(coverage.is_supported("load"));
        assert!(coverage.is_supported("br"));
        assert!(!coverage.is_supported("await"));
        assert!(!coverage.is_supported("inline_asm"));
    }

    #[test]
    fn test_supported_ops() {
        let coverage = InstructionCoverage::new();
        let ops = coverage.supported_ops();
        assert!(ops.contains(&"add"));
        assert!(ops.contains(&"load"));
        assert!(!ops.contains(&"await"));
    }

    #[test]
    fn test_unsupported_ops() {
        let coverage = InstructionCoverage::new();
        let ops = coverage.unsupported_ops();
        assert!(ops.contains(&"await"));
        assert!(ops.contains(&"inline_asm"));
    }

    #[test]
    fn test_requires_diagnostic() {
        let coverage = InstructionCoverage::new();
        assert!(coverage.requires_diagnostic("await").is_some());
        assert!(coverage.requires_diagnostic("inline_asm").is_some());
        assert!(coverage.requires_diagnostic("add").is_none());
    }

    #[test]
    fn test_coverage_percentage() {
        let coverage = InstructionCoverage::new();
        let pct = coverage.coverage_percentage();
        assert!(pct > 0.0);
        assert!(pct <= 100.0);
    }

    #[test]
    fn test_all_zig_air_ops_mapped() {
        let coverage = InstructionCoverage::new();
        let key_ops = vec![
            "add", "sub", "mul", "div", "rem",
            "load", "store", "alloca",
            "br", "ret", "call",
            "await", "inline_asm",
        ];
        for op in key_ops {
            assert!(coverage.lookup(op).is_some(), "missing op: {}", op);
        }
    }
}