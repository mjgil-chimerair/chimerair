//! Zig generics and comptime modeling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A generic function instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericInstantiation {
    /// Generic function ID
    pub generic_id: u64,
    /// Type arguments
    pub type_args: Vec<u64>,
    /// Comptime arguments
    pub comptime_args: Vec<u64>,
    /// Instantiated function ID
    pub instantiated_id: u64,
}

impl GenericInstantiation {
    /// Create a new instantiation
    pub fn new(generic_id: u64) -> Self {
        Self {
            generic_id,
            type_args: Vec::new(),
            comptime_args: Vec::new(),
            instantiated_id: 0,
        }
    }

    /// Add a type argument
    pub fn with_type_arg(mut self, ty: u64) -> Self {
        self.type_args.push(ty);
        self
    }

    /// Add a comptime argument
    pub fn with_comptime_arg(mut self, arg: u64) -> Self {
        self.comptime_args.push(arg);
        self
    }
}

/// Comptime value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComptimeValue {
    /// Integer value
    Int(i64),
    /// Unsigned integer value
    Uint(u64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// String value
    String(String),
    /// Type value (refers to a type ID)
    Type(u64),
    /// Undefined/null
    Undefined,
    /// Runtime-known value (not a compile-time constant)
    Runtime,
}

/// Builtin query types in Zig
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuiltinQuery {
    /// @sizeOf - size of a type
    SizeOf,
    /// @alignOf - alignment of a type
    AlignOf,
    /// @typeInfo - type information
    TypeInfo,
    /// @typeOf - type of an expression
    TypeOf,
    /// @field - field access at comptime
    Field,
    /// @tagName - tag name for an enum
    TagName,
    /// @errorName - error name lookup
    ErrorName,
    /// @errorReturnTrace - error return trace
    ErrorReturnTrace,
    /// @This - current type
    This,
    /// @panic - panic with message
    Panic,
    /// @compileLog - log during comptime
    CompileLog,
    /// @embedFile - embedded file contents
    EmbedFile,
    /// @sqrt - square root
    Sqrt,
    /// @sin - sine
    Sin,
    /// @cos - cosine
    Cos,
    /// @abs - absolute value
    Abs,
    /// @floor - floor
    Floor,
    /// @ceil - ceiling
    Ceil,
    /// @round - round
    Round,
    /// @trunc - truncate
    Trunc,
    /// @popCount - population count
    PopCount,
    /// @clz - count leading zeros
    Clz,
    /// @ctz - count trailing zeros
    Ctz,
    /// @bitReverse - reverse bits
    BitReverse,
    /// @byteSwap - byte swap
    ByteSwap,
    /// @import - import a file
    Import,
    /// @cImport - C import
    CImport,
    /// @cInclude - C include
    CInclude,
    /// @setRuntimeSafety - set runtime safety
    SetRuntimeSafety,
    /// @setFloatMode - set float mode
    SetFloatMode,
    /// @frame - current frame
    Frame,
    /// @frameAddress - frame address
    FrameAddress,
    /// @returnAddress - return address
    ReturnAddress,
    /// @breakpoint - breakpoint
    Breakpoint,
    /// @trap - trap
    Trap,
    /// @debugTrap - debug trap
    DebugTrap,
    /// @shuffle - vector shuffle
    Shuffle,
    /// @select - select
    Select,
    /// @splat - splat value
    Splat,
    /// @reduce - reduce vector
    Reduce,
    /// @atomicLoad - atomic load
    AtomicLoad,
    /// @atomicStore - atomic store
    AtomicStore,
    /// @atomicRmw - atomic read-modify-write
    AtomicRmw,
    /// @atomicStoreUnordered - unordered atomic store
    AtomicStoreUnordered,
    /// @fence - fence
    Fence,
    /// @mulWithOverflow - multiply with overflow
    MulWithOverflow,
    /// @shlWithOverflow - shift left with overflow
    ShlWithOverflow,
    /// @shrWithOverflow - shift right with overflow
    ShrWithOverflow,
    /// @addWithOverflow - add with overflow
    AddWithOverflow,
    /// @subWithOverflow - subtract with overflow
    SubWithOverflow,
}

impl BuiltinQuery {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sizeOf" => Some(BuiltinQuery::SizeOf),
            "alignOf" => Some(BuiltinQuery::AlignOf),
            "typeInfo" => Some(BuiltinQuery::TypeInfo),
            "typeOf" => Some(BuiltinQuery::TypeOf),
            "field" => Some(BuiltinQuery::Field),
            "tagName" => Some(BuiltinQuery::TagName),
            "errorName" => Some(BuiltinQuery::ErrorName),
            "errorReturnTrace" => Some(BuiltinQuery::ErrorReturnTrace),
            "This" => Some(BuiltinQuery::This),
            "panic" => Some(BuiltinQuery::Panic),
            "compileLog" => Some(BuiltinQuery::CompileLog),
            "embedFile" => Some(BuiltinQuery::EmbedFile),
            "sqrt" => Some(BuiltinQuery::Sqrt),
            "sin" => Some(BuiltinQuery::Sin),
            "cos" => Some(BuiltinQuery::Cos),
            "abs" => Some(BuiltinQuery::Abs),
            "floor" => Some(BuiltinQuery::Floor),
            "ceil" => Some(BuiltinQuery::Ceil),
            "round" => Some(BuiltinQuery::Round),
            "trunc" => Some(BuiltinQuery::Trunc),
            "popCount" => Some(BuiltinQuery::PopCount),
            "clz" => Some(BuiltinQuery::Clz),
            "ctz" => Some(BuiltinQuery::Ctz),
            "bitReverse" => Some(BuiltinQuery::BitReverse),
            "byteSwap" => Some(BuiltinQuery::ByteSwap),
            "import" => Some(BuiltinQuery::Import),
            "cImport" => Some(BuiltinQuery::CImport),
            "cInclude" => Some(BuiltinQuery::CInclude),
            "setRuntimeSafety" => Some(BuiltinQuery::SetRuntimeSafety),
            "setFloatMode" => Some(BuiltinQuery::SetFloatMode),
            "frame" => Some(BuiltinQuery::Frame),
            "frameAddress" => Some(BuiltinQuery::FrameAddress),
            "returnAddress" => Some(BuiltinQuery::ReturnAddress),
            "breakpoint" => Some(BuiltinQuery::Breakpoint),
            "trap" => Some(BuiltinQuery::Trap),
            "debugTrap" => Some(BuiltinQuery::DebugTrap),
            "shuffle" => Some(BuiltinQuery::Shuffle),
            "select" => Some(BuiltinQuery::Select),
            "splat" => Some(BuiltinQuery::Splat),
            "reduce" => Some(BuiltinQuery::Reduce),
            "atomicLoad" => Some(BuiltinQuery::AtomicLoad),
            "atomicStore" => Some(BuiltinQuery::AtomicStore),
            "atomicRmw" => Some(BuiltinQuery::AtomicRmw),
            "atomicStoreUnordered" => Some(BuiltinQuery::AtomicStoreUnordered),
            "fence" => Some(BuiltinQuery::Fence),
            "mulWithOverflow" => Some(BuiltinQuery::MulWithOverflow),
            "shlWithOverflow" => Some(BuiltinQuery::ShlWithOverflow),
            "shrWithOverflow" => Some(BuiltinQuery::ShrWithOverflow),
            "addWithOverflow" => Some(BuiltinQuery::AddWithOverflow),
            "subWithOverflow" => Some(BuiltinQuery::SubWithOverflow),
            _ => None,
        }
    }
}

/// A comptime function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeCall {
    /// Callee (function ID or builtin)
    pub callee: ComptimeCallee,
    /// Arguments
    pub args: Vec<u64>,
    /// Result type (for type-valued results)
    pub result_type: Option<u64>,
}

impl ComptimeCall {
    /// Create a new comptime call
    pub fn new(callee: ComptimeCallee) -> Self {
        Self {
            callee,
            args: Vec::new(),
            result_type: None,
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, arg: u64) -> Self {
        self.args.push(arg);
        self
    }

    /// Add arguments
    pub fn with_args(mut self, args: Vec<u64>) -> Self {
        self.args.extend(args);
        self
    }

    /// Set result type
    pub fn with_result_type(mut self, ty: u64) -> Self {
        self.result_type = Some(ty);
        self
    }
}

/// Callee of a comptime call
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComptimeCallee {
    /// User-defined function (by ID)
    Function(u64),
    /// Builtin query
    Builtin(BuiltinQuery),
    /// Anonymous comptime block
    ComptimeBlock,
}

impl ComptimeCallee {
    /// Check if this is a builtin
    pub fn is_builtin(&self) -> bool {
        matches!(self, ComptimeCallee::Builtin(_))
    }

    /// Get builtin query if this is a builtin
    pub fn as_builtin(&self) -> Option<BuiltinQuery> {
        match self {
            ComptimeCallee::Builtin(q) => Some((*q).clone()),
            _ => None,
        }
    }
}

/// An inline loop iteration (comptime `inline for`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeInlineLoop {
    /// Iterator expression
    pub iterator: u64,
    /// Body block with captured iteration variable
    pub body_block: u64,
    /// Iteration variable name
    pub var_name: String,
    /// Number of iterations (if known)
    pub num_iterations: Option<usize>,
}

/// Generic model for tracking type parameters and instantiations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericModel {
    /// Type parameters (name -> type ID)
    type_params: HashMap<String, u64>,
    /// Comptime parameters (name -> type ID)
    comptime_params: HashMap<String, u64>,
    /// Instantiations keyed by instantiation ID
    instantiations: HashMap<u64, GenericInstantiation>,
    /// Generic function IDs
    generic_functions: Vec<u64>,
}

impl Default for GenericModel {
    fn default() -> Self {
        Self::new()
    }
}

impl GenericModel {
    /// Create a new generic model
    pub fn new() -> Self {
        Self {
            type_params: HashMap::new(),
            comptime_params: HashMap::new(),
            instantiations: HashMap::new(),
            generic_functions: Vec::new(),
        }
    }

    /// Add a type parameter
    pub fn add_type_param(&mut self, name: &str, type_id: u64) {
        self.type_params.insert(name.to_string(), type_id);
    }

    /// Add a comptime parameter
    pub fn add_comptime_param(&mut self, name: &str, type_id: u64) {
        self.comptime_params.insert(name.to_string(), type_id);
    }

    /// Get a type parameter
    pub fn get_type_param(&self, name: &str) -> Option<u64> {
        self.type_params.get(name).copied()
    }

    /// Get a comptime parameter
    pub fn get_comptime_param(&self, name: &str) -> Option<u64> {
        self.comptime_params.get(name).copied()
    }

    /// Register a generic function
    pub fn register_generic(&mut self, func_id: u64) {
        self.generic_functions.push(func_id);
    }

    /// Add an instantiation
    pub fn add_instantiation(&mut self, inst: GenericInstantiation) {
        self.instantiations.insert(inst.instantiated_id, inst);
    }

    /// Get an instantiation by ID
    pub fn get_instantiation(&self, id: u64) -> Option<&GenericInstantiation> {
        self.instantiations.get(&id)
    }

    /// Check if a function is generic
    pub fn is_generic(&self, func_id: u64) -> bool {
        self.generic_functions.contains(&func_id)
    }

    /// Number of type parameters
    pub fn num_type_params(&self) -> usize {
        self.type_params.len()
    }

    /// Number of comptime parameters
    pub fn num_comptime_params(&self) -> usize {
        self.comptime_params.len()
    }

    /// Number of instantiations
    pub fn num_instantiations(&self) -> usize {
        self.instantiations.len()
    }
}

/// Comptime model for tracking compile-time known values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComptimeModel {
    /// Known comptime values by instruction ID
    values: HashMap<u64, ComptimeValue>,
    /// Functions that are comptime-only
    comptime_functions: Vec<u64>,
}

impl Default for ComptimeModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ComptimeModel {
    /// Create a new comptime model
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            comptime_functions: Vec::new(),
        }
    }

    /// Record a comptime value
    pub fn record_value(&mut self, inst_id: u64, value: ComptimeValue) {
        self.values.insert(inst_id, value);
    }

    /// Get a comptime value
    pub fn get_value(&self, inst_id: u64) -> Option<&ComptimeValue> {
        self.values.get(&inst_id)
    }

    /// Register a comptime-only function
    pub fn register_comptime_function(&mut self, func_id: u64) {
        self.comptime_functions.push(func_id);
    }

    /// Check if a function is comptime-only
    pub fn is_comptime_function(&self, func_id: u64) -> bool {
        self.comptime_functions.contains(&func_id)
    }

    /// Check if an instruction has a known comptime value
    pub fn has_value(&self, inst_id: u64) -> bool {
        self.values.contains_key(&inst_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_instantiation() {
        let inst = GenericInstantiation::new(100)
            .with_type_arg(1)
            .with_type_arg(2)
            .with_comptime_arg(50);

        assert_eq!(inst.generic_id, 100);
        assert_eq!(inst.type_args.len(), 2);
        assert_eq!(inst.comptime_args.len(), 1);
    }

    #[test]
    fn test_generic_model() {
        let mut model = GenericModel::new();
        model.add_type_param("T", 1);
        model.add_type_param("U", 2);
        model.register_generic(100);

        assert_eq!(model.get_type_param("T"), Some(1));
        assert_eq!(model.get_type_param("U"), Some(2));
        assert!(model.is_generic(100));
        assert!(!model.is_generic(999));
        assert_eq!(model.num_type_params(), 2);
    }

    #[test]
    fn test_comptime_value() {
        assert!(matches!(ComptimeValue::Int(42), ComptimeValue::Int(42)));
        assert!(matches!(
            ComptimeValue::Bool(true),
            ComptimeValue::Bool(true)
        ));
        assert!(matches!(ComptimeValue::Type(100), ComptimeValue::Type(100)));
    }

    #[test]
    fn test_comptime_model() {
        let mut model = ComptimeModel::new();
        model.record_value(1, ComptimeValue::Int(42));
        model.record_value(2, ComptimeValue::Bool(true));
        model.register_comptime_function(100);

        assert_eq!(model.get_value(1), Some(&ComptimeValue::Int(42)));
        assert!(model.is_comptime_function(100));
        assert!(!model.is_comptime_function(200));
    }

    #[test]
    fn test_builtin_query_parse() {
        assert_eq!(BuiltinQuery::parse("sizeOf"), Some(BuiltinQuery::SizeOf));
        assert_eq!(BuiltinQuery::parse("alignOf"), Some(BuiltinQuery::AlignOf));
        assert_eq!(
            BuiltinQuery::parse("typeInfo"),
            Some(BuiltinQuery::TypeInfo)
        );
        assert_eq!(BuiltinQuery::parse("typeOf"), Some(BuiltinQuery::TypeOf));
        assert_eq!(BuiltinQuery::parse("field"), Some(BuiltinQuery::Field));
        assert_eq!(
            BuiltinQuery::parse("embedFile"),
            Some(BuiltinQuery::EmbedFile)
        );
        assert_eq!(BuiltinQuery::parse("unknown"), None);
    }

    #[test]
    fn test_builtin_query_all_variants() {
        // Verify all builtin queries can be parsed
        let queries = [
            "sizeOf",
            "alignOf",
            "typeInfo",
            "typeOf",
            "field",
            "tagName",
            "errorName",
            "errorReturnTrace",
            "This",
            "panic",
            "compileLog",
            "embedFile",
            "sqrt",
            "sin",
            "cos",
            "abs",
            "floor",
            "ceil",
            "round",
            "trunc",
            "popCount",
            "clz",
            "ctz",
            "bitReverse",
            "byteSwap",
            "import",
            "cImport",
            "cInclude",
            "setRuntimeSafety",
            "setFloatMode",
            "frame",
            "frameAddress",
            "returnAddress",
            "breakpoint",
            "trap",
            "debugTrap",
            "shuffle",
            "select",
            "splat",
            "reduce",
            "atomicLoad",
            "atomicStore",
            "atomicRmw",
            "atomicStoreUnordered",
            "fence",
            "mulWithOverflow",
            "shlWithOverflow",
            "shrWithOverflow",
            "addWithOverflow",
            "subWithOverflow",
        ];
        for q in queries {
            assert!(BuiltinQuery::parse(q).is_some(), "failed to parse: {}", q);
        }
    }

    #[test]
    fn test_comptime_call_creation() {
        let call = ComptimeCall::new(ComptimeCallee::Function(42))
            .with_arg(1)
            .with_arg(2)
            .with_result_type(100);

        assert!(matches!(call.callee, ComptimeCallee::Function(42)));
        assert_eq!(call.args.len(), 2);
        assert_eq!(call.result_type, Some(100));
    }

    #[test]
    fn test_comptime_call_builtin() {
        let call = ComptimeCall::new(ComptimeCallee::Builtin(BuiltinQuery::SizeOf))
            .with_arg(50)
            .with_result_type(1);

        assert!(call.callee.is_builtin());
        assert_eq!(call.callee.as_builtin(), Some(BuiltinQuery::SizeOf));
    }

    #[test]
    fn test_comptime_callee_is_builtin() {
        let func = ComptimeCallee::Function(100);
        let builtin = ComptimeCallee::Builtin(BuiltinQuery::AlignOf);
        let block = ComptimeCallee::ComptimeBlock;

        assert!(!func.is_builtin());
        assert!(builtin.is_builtin());
        assert!(!block.is_builtin());
    }

    #[test]
    fn test_comptime_inline_loop() {
        let loop_data = ComptimeInlineLoop {
            iterator: 10,
            body_block: 20,
            var_name: "i".to_string(),
            num_iterations: Some(5),
        };
        assert_eq!(loop_data.iterator, 10);
        assert_eq!(loop_data.var_name, "i");
        assert_eq!(loop_data.num_iterations, Some(5));
    }

    #[test]
    fn test_comptime_call_chained_args() {
        let call = ComptimeCall::new(ComptimeCallee::Function(1))
            .with_args(vec![10, 20, 30])
            .with_result_type(100);

        assert_eq!(call.args.len(), 3);
        assert_eq!(call.args[0], 10);
        assert_eq!(call.args[2], 30);
    }

    #[test]
    fn test_generic_instantiation_with_type_and_comptime_args() {
        let inst = GenericInstantiation::new(1)
            .with_type_arg(100)
            .with_type_arg(200)
            .with_comptime_arg(5)
            .with_comptime_arg(10);

        assert_eq!(inst.generic_id, 1);
        assert_eq!(inst.type_args.len(), 2);
        assert_eq!(inst.comptime_args.len(), 2);
    }

    #[test]
    fn test_generic_model_instantiations() {
        let mut model = GenericModel::new();
        model.register_generic(100);

        let inst = GenericInstantiation::new(100)
            .with_type_arg(1)
            .with_type_arg(2);
        model.add_instantiation(inst);

        assert_eq!(model.num_instantiations(), 1);
        assert!(!model.is_generic(999)); // only 100 is registered
    }

    #[test]
    fn test_comptime_value_variants() {
        assert!(matches!(ComptimeValue::Int(-42), ComptimeValue::Int(-42)));
        assert!(matches!(ComptimeValue::Uint(42), ComptimeValue::Uint(42)));
        assert!(matches!(
            ComptimeValue::Float(3.14),
            ComptimeValue::Float(3.14)
        ));
        let string_val = ComptimeValue::String("test".to_string());
        assert!(matches!(string_val, ComptimeValue::String(_)));
        assert!(matches!(ComptimeValue::Undefined, ComptimeValue::Undefined));
        assert!(matches!(ComptimeValue::Runtime, ComptimeValue::Runtime));
    }

    #[test]
    fn test_comptime_model_has_value() {
        let mut model = ComptimeModel::new();
        model.record_value(1, ComptimeValue::Int(42));

        assert!(model.has_value(1));
        assert!(!model.has_value(2));
    }
}
