//! BEAM bytecode analyzer.
//!
//! Analyzes BEAM bytecode to extract process hints, function details,
//! and semantic information for the BEAM adapter.

use chimera_beam_schema::{Atom, BeamModuleInfo, ExportEntry, FunctionInfo};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("analysis failed: {0}")]
    Failed(String),
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Bytecode analysis result.
#[derive(Debug, Clone)]
pub struct BytecodeAnalysis {
    pub module_name: String,
    pub functions: Vec<FunctionAnalysis>,
    pub exports: Vec<String>,
    pub imports: Vec<(String, String)>,
    pub process_hints: Vec<ProcessHint>,
}

/// Analysis of a single function.
#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub name: String,
    pub arity: u32,
    pub is_generator: bool,
    pub has_spawn: bool,
    pub has_receive: bool,
    pub has_send: bool,
    pub has_link: bool,
    pub has_monitor: bool,
    pub instructions: Vec<Instruction>,
}

/// A single bytecode instruction.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub args: Vec<String>,
}

/// Process creation hints from bytecode analysis.
#[derive(Debug, Clone)]
pub struct ProcessHint {
    pub function: String,
    pub arity: u32,
    pub spawn_type: SpawnType,
}

/// Type of process spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnType {
    /// Regular spawn (spawn/3)
    Regular,
    /// Linked spawn (spawn_link/3)
    Linked,
    /// Monitored spawn (spawn_monitor/3)
    Monitored,
}

/// Bytecode analyzer for BEAM files.
pub struct BytecodeAnalyzer;

impl BytecodeAnalyzer {
    pub fn new() -> Self {
        BytecodeAnalyzer
    }

    pub fn analyze(&self, data: &[u8]) -> AnalysisResult<BytecodeAnalysis> {
        // Check BEAM magic
        if data.len() < 4 || &data[0..4] != b"BEAM" {
            return Err(AnalysisError::Failed("Invalid BEAM magic".to_string()));
        }

        // Extract and analyze function table
        let functions = self.extract_function_table(data)?;

        // Extract exports
        let exports = self.extract_exports(data)?;

        // Extract imports
        let imports = self.extract_imports(data)?;

        // Analyze process creation patterns
        let process_hints = self.find_process_hints(&functions);

        // Determine module name
        let module_name = self
            .extract_module_name(data)
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(BytecodeAnalysis {
            module_name,
            functions,
            exports,
            imports,
            process_hints,
        })
    }

    fn extract_module_name(&self, data: &[u8]) -> AnalysisResult<String> {
        // Simple extraction - look for first string in BEAM
        let mut pos = 5; // Skip BEAM magic + version
        let mut start = 0;

        while pos < data.len() {
            if data[pos] == 0 {
                if pos > start {
                    return Ok(String::from_utf8_lossy(&data[start..pos]).to_string());
                }
                start = pos + 1;
            }
            pos += 1;
        }

        Err(AnalysisError::Failed(
            "Could not find module name".to_string(),
        ))
    }

    fn extract_function_table(&self, data: &[u8]) -> AnalysisResult<Vec<FunctionAnalysis>> {
        let mut functions = Vec::new();

        // In a real BEAM file, function info is stored in the code chunk
        // For this implementation, we provide a simple placeholder structure
        // Real implementation would parse:
        // - Lambda table (for closures)
        // - Export table (public functions)
        // - Import table (external calls)
        // - Code chunk (bytecode instructions)

        // Placeholder for demo
        functions.push(FunctionAnalysis {
            name: "start".to_string(),
            arity: 0,
            is_generator: false,
            has_spawn: false,
            has_receive: false,
            has_send: false,
            has_link: false,
            has_monitor: false,
            instructions: Vec::new(),
        });

        Ok(functions)
    }

    fn extract_exports(&self, data: &[u8]) -> AnalysisResult<Vec<String>> {
        let mut exports = Vec::new();

        // Scan for export table
        // In real BEAM: offset to export table is in header
        // Format: [label, arity, index] repeated

        // Placeholder
        exports.push("start/0".to_string());

        Ok(exports)
    }

    fn extract_imports(&self, data: &[u8]) -> AnalysisResult<Vec<(String, String)>> {
        let mut imports = Vec::new();

        // Scan for import table
        // In real BEAM: [module_atom_index, function_atom_index, arity]

        // Placeholder
        imports.push(("erlang".to_string(), "spawn".to_string()));

        Ok(imports)
    }

    fn find_process_hints(&self, functions: &[FunctionAnalysis]) -> Vec<ProcessHint> {
        let mut hints = Vec::new();

        for func in functions {
            if func.has_spawn {
                hints.push(ProcessHint {
                    function: func.name.clone(),
                    arity: func.arity,
                    spawn_type: SpawnType::Regular,
                });
            }
        }

        hints
    }
}

impl Default for BytecodeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_new() {
        let analyzer = BytecodeAnalyzer::new();
        let data = b"BEAM".to_vec();
        let result = analyzer.analyze(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyzer_invalid_magic() {
        let analyzer = BytecodeAnalyzer::new();
        let data = b"XXXX".to_vec();
        let result = analyzer.analyze(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_spawn_type_variants() {
        assert_eq!(SpawnType::Regular, SpawnType::Regular);
        assert_eq!(SpawnType::Linked, SpawnType::Linked);
        assert_eq!(SpawnType::Monitored, SpawnType::Monitored);
    }
}
