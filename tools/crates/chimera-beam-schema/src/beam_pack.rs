//! `.beam_pack` BEAM package schema v1.
//!
//! Full package including compiled modules and metadata.

use serde::{Deserialize, Serialize};

/// Magic bytes for `.beam_pack` binary format.
pub const BEAM_PACK_MAGIC: &[u8; 8] = b"BeamPack";

/// Current schema version.
pub const PACK_SCHEMA_VERSION: u32 = 1;

/// Module checksum for integrity verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamChecksum {
    pub algorithm: String,
    pub value: [u8; 32],
}

impl BeamChecksum {
    pub fn blake3(value: [u8; 32]) -> Self {
        BeamChecksum {
            algorithm: "blake3".to_string(),
            value,
        }
    }
}

/// Compiled BEAM module within a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamPackModule {
    pub module_name: String,
    pub bytecode: Vec<u8>,
    pub source_path: Option<String>,
    pub abstract_code: Option<AbstractCode>,
    pub core_erlang: Option<CoreErlangModule>,
    pub debug_info: DebugInfo,
    pub checksum: BeamChecksum,
}

/// Abstract code representation (from BEAM's abstract format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractCode {
    pub version: u32,
    pub forms: Vec<AbstractForm>,
}

/// Abstract syntax form (a top-level construct).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstractForm {
    ModuleAttribute {
        line: u32,
        key: String,
        value: Term,
    },
    FunctionAttribute {
        line: u32,
        name: String,
        arity: u32,
    },
    FunctionDefinition {
        line: u32,
        name: String,
        arity: u32,
        clauses: Vec<Clause>,
    },
}

/// A clause within a function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clause {
    pub line: u32,
    pub patterns: Vec<Term>,
    pub guards: Vec<Term>,
    pub body: Vec<Term>,
}

/// Core Erlang module representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreErlangModule {
    pub module_name: String,
    pub definitions: Vec<CoreDefinition>,
}

/// Core Erlang definition (function or variable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreDefinition {
    Function {
        name: String,
        arity: u32,
        annotation: Vec<String>,
        vars: Vec<(String, Type)>,
        body: CoreExpr,
    },
    Variable {
        name: String,
        value: CoreExpr,
    },
}

/// Core Erlang expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreExpr {
    /// Variable reference.
    Var(String),
    /// Literal atom.
    Atom(String),
    /// Integer literal.
    Int(i64),
    /// Tuple constructor.
    Tuple(Vec<CoreExpr>),
    /// List constructor (cons cell).
    Cons {
        head: Box<CoreExpr>,
        tail: Box<CoreExpr>,
    },
    /// Empty list.
    Nil,
    /// Binary constructor.
    Binary(Vec<CoreExpr>),
    /// Application of a function.
    Apply {
        module: Option<String>,
        name: String,
        args: Vec<CoreExpr>,
        tail: bool,
    },
    /// Lambda (fun expression).
    Lambda {
        vars: Vec<String>,
        body: Box<CoreExpr>,
    },
    /// Let binding.
    Let {
        bindings: Vec<(String, CoreExpr)>,
        body: Box<CoreExpr>,
    },
    /// Sequence of expressions.
    Seq {
        first: Box<CoreExpr>,
        then: Box<CoreExpr>,
    },
    /// Case expression (pattern matching).
    Case {
        expr: Box<CoreExpr>,
        clauses: Vec<CoreCaseClause>,
    },
    /// Try-catch expression.
    Try {
        expr: Box<CoreExpr>,
        vars: Vec<String>,
        body: Box<CoreExpr>,
        catch_vars: Vec<String>,
        handler: Box<CoreExpr>,
    },
    /// Receive expression.
    Receive {
        clauses: Vec<CoreCaseClause>,
        timeout: Option<Box<CoreExpr>>,
        after: Option<Box<CoreExpr>>,
    },
    /// Primitive operation (BIF).
    PrimOp { name: String, args: Vec<CoreExpr> },
    /// Local function reference.
    Local { name: String, arity: u32 },
    /// External function reference.
    External {
        module: String,
        name: String,
        arity: u32,
    },
    /// Exit signal.
    Exit { reason: Box<CoreExpr> },
    /// Throw signal.
    Throw { reason: Box<CoreExpr> },
    /// Raise (for exception propagation).
    Raise {
        reason: Box<CoreExpr>,
        stacktrace: Box<CoreExpr>,
    },
    /// Catch wrapper.
    Catch { expr: Box<CoreExpr> },
}

/// A case clause within case/receive expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreCaseClause {
    pub patterns: Vec<CorePattern>,
    pub guards: Vec<CoreExpr>,
    pub body: Vec<CoreExpr>,
}

/// Core Erlang pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorePattern {
    Wildcard,
    Var(String),
    Atom(String),
    Int(i64),
    Tuple(Vec<CorePattern>),
    Cons {
        head: Box<CorePattern>,
        tail: Box<CorePattern>,
    },
    Nil,
    Binary(Vec<CorePattern>),
}

/// Type annotation for Core Erlang.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Any,
    Atom,
    Integer,
    Float,
    Binary,
    List(Box<Type>),
    Tuple(Vec<Type>),
    Function { args: Vec<Type>, result: Box<Type> },
}

/// Debug information for a compiled module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    pub source_path: Option<String>,
    pub line_info: Vec<LineInfo>,
    pub break_points: Vec<BreakPoint>,
    pub stack_traces: bool,
}

/// Line information mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineInfo {
    pub address: u32,
    pub line: u32,
}

/// Breakpoint information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakPoint {
    pub line: u32,
    pub enable: bool,
}

/// Term representation (reused from beam_snap).
use super::beam_snap::Term;

/// `.beam_pack` package header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamPackHeader {
    pub magic: [u8; 8],
    pub schema_version: u32,
    pub min_adapter_version: u32,
    pub module_count: u32,
    pub total_size: u64,
    pub checksum: [u8; 32],
}

impl BeamPackHeader {
    pub fn new(module_count: u32) -> Self {
        BeamPackHeader {
            magic: *BEAM_PACK_MAGIC,
            schema_version: PACK_SCHEMA_VERSION,
            min_adapter_version: 1,
            module_count,
            total_size: 0,
            checksum: [0u8; 32],
        }
    }
}

/// Full BEAM package schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamPackSchema {
    pub header: BeamPackHeader,
    pub modules: Vec<BeamPackModule>,
}

impl BeamPackSchema {
    pub fn new() -> Self {
        BeamPackSchema {
            header: BeamPackHeader::new(0),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: BeamPackModule) {
        self.header.module_count += 1;
        self.modules.push(module);
    }
}

impl Default for BeamPackSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_checksum_blake3() {
        let checksum = BeamChecksum::blake3([0u8; 32]);
        assert_eq!(checksum.algorithm, "blake3");
    }

    #[test]
    fn test_beam_pack_header_new() {
        let header = BeamPackHeader::new(3);
        assert_eq!(header.module_count, 3);
        assert_eq!(header.magic, *BEAM_PACK_MAGIC);
    }

    #[test]
    fn test_beam_pack_schema_add_module() {
        let mut schema = BeamPackSchema::new();
        let module = BeamPackModule {
            module_name: "test".to_string(),
            bytecode: vec![0x00, 0x00],
            source_path: None,
            abstract_code: None,
            core_erlang: None,
            debug_info: DebugInfo::default(),
            checksum: BeamChecksum::blake3([0u8; 32]),
        };
        schema.add_module(module);
        assert_eq!(schema.header.module_count, 1);
        assert_eq!(schema.modules.len(), 1);
    }

    #[test]
    fn test_abstract_form_serialization() {
        let form = AbstractForm::ModuleAttribute {
            line: 1,
            key: "author".to_string(),
            value: Term::atom("test"),
        };
        let json = serde_json::to_string(&form).unwrap();
        assert!(json.contains("ModuleAttribute"));
    }

    #[test]
    fn test_core_expr_apply() {
        let expr = CoreExpr::Apply {
            module: Some("erlang".to_string()),
            name: "spawn".to_string(),
            args: vec![],
            tail: false,
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("Apply"));
    }

    #[test]
    fn test_core_pattern_cons() {
        let pattern = CorePattern::Cons {
            head: Box::new(CorePattern::Atom("a".to_string())),
            tail: Box::new(CorePattern::Nil),
        };
        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("Cons"));
    }

    #[test]
    fn test_debug_info_default() {
        let info = DebugInfo::default();
        assert!(info.source_path.is_none());
        assert!(info.break_points.is_empty());
    }
}

impl Default for DebugInfo {
    fn default() -> Self {
        DebugInfo {
            source_path: None,
            line_info: Vec::new(),
            break_points: Vec::new(),
            stack_traces: false,
        }
    }
}
