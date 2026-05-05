//! BEAM bytecode disassembler and analyzer.
//!
//! Analyzes BEAM bytecode files (.beam) to extract module information,
//! function details, and process hints for the BEAM adapter.

pub mod analyzer;
pub mod disassembler;

pub use analyzer::{AnalysisResult, BytecodeAnalyzer, FunctionAnalysis};
pub use disassembler::{BeamDisassembler, DisassemblerError, DisassemblerResult};

use chimera_beam_schema::{BeamModuleInfo, BeamSnapSchema};

/// Disassemble a BEAM bytecode file and produce a BeamSnapSchema.
pub fn disassemble(data: &[u8]) -> DisassemblerResult<BeamSnapSchema> {
    let mut disassembler = BeamDisassembler::new(data);
    disassembler.run()
}

/// Disassemble a BEAM bytecode file and produce a BeamModuleInfo.
pub fn analyze(data: &[u8]) -> DisassemblerResult<BeamModuleInfo> {
    let mut disassembler = BeamDisassembler::new(data);
    disassembler.analyze_module()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembler_creation() {
        let data = Vec::new();
        let dis = BeamDisassembler::new(&data);
        assert!(!dis.has_error());
    }
}
