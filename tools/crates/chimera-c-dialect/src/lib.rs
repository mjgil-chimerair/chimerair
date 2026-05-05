//! Chimera C semantic dialect crate.
//!
//! Represents C declarations, types, expressions, statements, memory
//! operations, calls, volatile, restrict, const, atomics, and unsafe/trust
//! assumptions before Chimera lowering.
//!
//! Task 15: C semantic dialect crate

use chimera_c_abi::{AbiContext, PointerNullability};
use chimera_c_layout::{CLayoutExtractor, LayoutContext};
use chimera_c_schema::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for dialect operations
pub type Result<T> = std::result::Result<T, DialectError>;

/// Dialect errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DialectError {
    #[error("type mismatch: {0}")]
    TypeMismatch(String),
    #[error("incomplete type: {0}")]
    IncompleteType(String),
    #[error("invalid expression: {0}")]
    InvalidExpression(String),
    #[error("invalid statement: {0}")]
    InvalidStatement(String),
    #[error("symbol not found: {0}")]
    SymbolNotFound(String),
    #[error("duplicate definition: {0}")]
    DuplicateDefinition(String),
    #[error("linkage conflict: {0}")]
    LinkageConflict(String),
    #[error("unsafe operation requires contract: {0}")]
    UnsafeOperationRequiresContract(String),
    #[error("volatile access in safe context: {0}")]
    VolatileInSafeContext(String),
    #[error("const violation: {0}")]
    ConstViolation(String),
    #[error("atomic violation: {0}")]
    AtomicViolation(String),
    #[error("trust assumption violation: {0}")]
    TrustAssumptionViolation(String),
}

/// C dialect context - contains all declarations and types
#[derive(Debug, Clone, Default)]
pub struct CDialectContext {
    /// All declarations indexed by ID
    pub declarations: HashMap<DeclId, CDeclaration>,
    /// Symbol table for name lookup
    pub symbols: HashMap<String, DeclId>,
    /// Types indexed by reference
    pub types: HashMap<TypeRef, CType>,
    /// Translation unit info
    pub translation_unit: Option<TranslationUnitInfo>,
    /// ABI context
    pub abi_context: AbiContext,
    /// Layout context
    pub layout_context: LayoutContext,
    /// Trust assumptions
    pub trust_assumptions: Vec<TrustAssumption>,
    /// Unsafe operations needing contracts
    pub unsafe_operations: Vec<UnsafeOperation>,
    /// Volatile accesses
    pub volatile_accesses: Vec<VolatileAccess>,
    /// Atomic operations
    pub atomic_operations: Vec<AtomicOperation>,
}

impl CDialectContext {
    /// Add a declaration
    pub fn add_declaration(&mut self, decl: CDeclaration) -> DeclId {
        let id = decl.id();
        self.declarations.insert(id, decl.clone());
        if let Some(name) = decl.name() {
            self.symbols.insert(name.to_string(), id);
        }
        id
    }

    /// Look up a declaration by name
    pub fn lookup(&self, name: &str) -> Option<&CDeclaration> {
        self.symbols
            .get(name)
            .and_then(|id| self.declarations.get(id))
    }

    /// Look up a declaration by ID
    pub fn get_declaration(&self, id: &DeclId) -> Option<&CDeclaration> {
        self.declarations.get(id)
    }

    /// Add a type
    pub fn add_type(&mut self, type_ref: TypeRef, typ: CType) {
        self.types.insert(type_ref, typ);
    }

    /// Get a type
    pub fn get_type(&self, type_ref: &TypeRef) -> Option<&CType> {
        self.types.get(type_ref)
    }

    /// Add a trust assumption
    pub fn add_trust_assumption(&mut self, assumption: TrustAssumption) {
        self.trust_assumptions.push(assumption);
    }

    /// Add an unsafe operation
    pub fn add_unsafe_operation(&mut self, op: UnsafeOperation) {
        self.unsafe_operations.push(op);
    }
}

/// C declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CDeclaration {
    Function(CFunctionDecl),
    GlobalVariable(CGlobalVarDecl),
    Struct(CStructDecl),
    Union(CUnionDecl),
    Enum(CEnumDecl),
    Typedef(CTypedefDecl),
    EnumConstant(CEnumConstant),
    Macro(CMacroDecl),
}

impl CDeclaration {
    /// Get declaration name
    pub fn name(&self) -> Option<&str> {
        match self {
            CDeclaration::Function(f) => Some(&f.name),
            CDeclaration::GlobalVariable(g) => Some(&g.name),
            CDeclaration::Struct(s) => s.name.as_deref(),
            CDeclaration::Union(u) => u.name.as_deref(),
            CDeclaration::Enum(e) => e.name.as_deref(),
            CDeclaration::Typedef(t) => Some(&t.name),
            CDeclaration::EnumConstant(e) => Some(&e.name),
            CDeclaration::Macro(m) => Some(&m.name),
        }
    }

    /// Get declaration ID
    pub fn id(&self) -> DeclId {
        match self {
            CDeclaration::Function(f) => f.id,
            CDeclaration::GlobalVariable(g) => g.id,
            CDeclaration::Struct(s) => s.id,
            CDeclaration::Union(u) => u.id,
            CDeclaration::Enum(e) => e.id,
            CDeclaration::Typedef(t) => t.id,
            CDeclaration::EnumConstant(e) => e.id,
            CDeclaration::Macro(m) => m.id,
        }
    }

    /// Check if this is a definition (has body or complete type)
    pub fn is_definition(&self) -> bool {
        match self {
            CDeclaration::Function(f) => f.has_body,
            CDeclaration::GlobalVariable(g) => g.initializer.is_some(),
            CDeclaration::Struct(s) => !s.is_incomplete,
            CDeclaration::Union(u) => !u.is_incomplete,
            CDeclaration::Enum(e) => !e.is_incomplete,
            _ => true,
        }
    }
}

/// Function declaration in the dialect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunctionDecl {
    pub id: DeclId,
    pub name: String,
    pub linkage: CDeclarationLinkage,
    pub storage_class: CStorageClass,
    pub calling_convention: String,
    pub params: Vec<CParamDecl>,
    pub return_type: TypeRef,
    pub attributes: Vec<CAttribute>,
    pub source_span: SourceSpan,
    pub has_body: bool,
    pub is_inline: bool,
}

/// Parameter declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CParamDecl {
    pub name: String,
    pub typ: TypeRef,
    pub has_default: bool,
}

/// Global variable declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CGlobalVarDecl {
    pub id: DeclId,
    pub name: String,
    pub linkage: CDeclarationLinkage,
    pub storage_class: CStorageClass,
    pub typ: TypeRef,
    pub initializer: Option<CInitializer>,
    pub source_span: SourceSpan,
    pub is_thread_local: bool,
    pub is_static: bool,
}

/// Linkage classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CDeclarationLinkage {
    None,
    Internal,
    External,
    Weak,
}

/// Storage class specifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CStorageClass {
    None,
    Auto,
    Static,
    Extern,
    Register,
    ThreadLocal,
}

/// C attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAttribute {
    pub name: String,
    pub args: Option<Vec<String>>,
}

/// Struct declaration in the dialect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CStructDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub fields: Vec<CFieldDecl>,
    pub is_packed: bool,
    pub pack_align: Option<u32>,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Field declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFieldDecl {
    pub name: String,
    pub typ: TypeRef,
    pub bitfield_width: Option<u32>,
    pub offset: u64,
    pub size: u64,
    pub align: u32,
}

/// Union declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CUnionDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub variants: Vec<CFieldDecl>,
    pub size: u64,
    pub align: u32,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Enum declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CEnumDecl {
    pub id: DeclId,
    pub name: Option<String>,
    pub underlying_type: TypeRef,
    pub constants: Vec<CEnumConstant>,
    pub is_incomplete: bool,
    pub source_span: SourceSpan,
}

/// Enum constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CEnumConstant {
    pub id: DeclId,
    pub name: String,
    pub value: Option<i64>,
    pub source_span: SourceSpan,
}

/// Typedef declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTypedefDecl {
    pub id: DeclId,
    pub name: String,
    pub underlying_type: TypeRef,
    pub source_span: SourceSpan,
}

/// Macro declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CMacroDecl {
    pub id: DeclId,
    pub name: String,
    pub value: Option<String>,
    pub is_function_like: bool,
    pub params: Option<Vec<String>>,
    pub source_span: SourceSpan,
}

/// Initializer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CInitializer {
    Zero,
    Constant(String),
}

/// C type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CType {
    Void,
    Bool,
    Char {
        is_signed: bool,
    },
    Short {
        is_signed: bool,
    },
    Int {
        is_signed: bool,
    },
    Long {
        is_signed: bool,
    },
    LongLong {
        is_signed: bool,
    },
    Float,
    Double,
    LongDouble,
    Pointer {
        pointee: TypeRef,
        constness: u32,
        nullability: PointerNullability,
    },
    Array {
        element: TypeRef,
        length: Option<u64>,
    },
    FunctionPointer {
        params: Vec<TypeRef>,
        ret: Option<TypeRef>,
        cconv: String,
    },
    Struct(TypeRef),
    Union(TypeRef),
    Enum(TypeRef),
    Typedef(TypeRef),
    Volatile(TypeRef),
    Atomic(TypeRef),
    Incomplete,
}

impl CType {
    /// Check if this is an incomplete type
    pub fn is_incomplete(&self) -> bool {
        matches!(self, CType::Incomplete)
    }

    /// Check if this is a scalar type
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            CType::Bool
                | CType::Char { .. }
                | CType::Short { .. }
                | CType::Int { .. }
                | CType::Long { .. }
                | CType::LongLong { .. }
                | CType::Float
                | CType::Double
                | CType::LongDouble
                | CType::Pointer { .. }
        )
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CUnaryOperator {
    Plus,
    Minus,
    Not,
    BitNot,
    Deref,
    AddrOf,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CBinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Translation unit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnitInfo {
    pub id: TUId,
    pub source_file: String,
    pub header_dependencies: Vec<String>,
    pub macro_dependencies: Vec<String>,
}

/// Trust assumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssumption {
    pub kind: TrustKind,
    pub description: String,
    pub location: SourceSpan,
    pub is_explicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustKind {
    UnsafeOperation,
    RawPtrDeref,
    FFICall,
    MutableStatic,
    UnionAccess,
    InlineAsm,
    VolatileAccess,
    AtomicAccess,
}

/// Unsafe operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeOperation {
    pub kind: UnsafeOperationKind,
    pub description: String,
    pub location: SourceSpan,
    pub contract: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnsafeOperationKind {
    PointerDeref,
    ArrayIndex,
    UnionAccess,
    BitfieldAccess,
    Varargs,
    InlineAsm,
}

/// Volatile access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatileAccess {
    pub expression: String,
    pub is_read: bool,
    pub location: SourceSpan,
}

/// Atomic operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicOperation {
    pub expression: String,
    pub ordering: AtomicOrdering,
    pub location: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AtomicOrdering {
    Relaxed,
    Consume,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

/// C statement representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CStatement {
    /// Expression statement (including assignment, call, etc.)
    Expression(CExpression),
    /// Compound statement (block)
    Block(Vec<CStatement>),
    /// Return statement
    Return(Option<CExpression>),
    /// If statement
    If {
        condition: CExpression,
        then_block: Box<CStatement>,
        else_block: Option<Box<CStatement>>,
    },
    /// Switch statement
    Switch {
        expression: CExpression,
        cases: Vec<SwitchCase>,
    },
    /// While loop
    While {
        condition: CExpression,
        body: Box<CStatement>,
    },
    /// Do-while loop
    DoWhile {
        body: Box<CStatement>,
        condition: CExpression,
    },
    /// For loop
    For {
        init: Option<Box<CStatement>>,
        condition: Option<CExpression>,
        increment: Option<Box<CStatement>>,
        body: Box<CStatement>,
    },
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Label statement
    Label {
        label: String,
        statement: Box<CStatement>,
    },
    /// Goto statement
    Goto(String),
    /// Null statement
    Null,
}

/// Switch case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCase {
    pub value: i64,
    pub statement: Box<CStatement>,
}

/// C expression representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CExpression {
    /// Integer constant
    IntegerConstant { value: i64, typ: TypeRef },
    /// Floating point constant
    FloatConstant { value: f64, typ: TypeRef },
    /// String constant
    StringConstant { value: String },
    /// Variable reference
    Variable { name: String, typ: TypeRef },
    /// Unary operation
    Unary {
        operator: CUnaryOperator,
        operand: Box<CExpression>,
        typ: TypeRef,
    },
    /// Binary operation
    Binary {
        operator: CBinaryOperator,
        left: Box<CExpression>,
        right: Box<CExpression>,
        typ: TypeRef,
    },
    /// Assignment
    Assignment {
        target: Box<CExpression>,
        value: Box<CExpression>,
        typ: TypeRef,
    },
    /// Function call
    Call {
        function: Box<CExpression>,
        arguments: Vec<CExpression>,
        typ: TypeRef,
    },
    /// Field access
    FieldAccess {
        target: Box<CExpression>,
        field: String,
        typ: TypeRef,
    },
    /// Array subscript
    Subscript {
        target: Box<CExpression>,
        index: Box<CExpression>,
        typ: TypeRef,
    },
    /// Cast expression
    Cast {
        expression: Box<CExpression>,
        target_type: TypeRef,
        typ: TypeRef,
    },
    /// Address-of operation
    AddressOf(Box<CExpression>),
    /// Dereference operation
    Deref(Box<CExpression>),
    /// Comma expression
    Comma(Vec<CExpression>),
    /// Conditional expression
    Conditional {
        condition: Box<CExpression>,
        then_expr: Box<CExpression>,
        else_expr: Box<CExpression>,
        typ: TypeRef,
    },
    /// Sizeof expression
    Sizeof {
        expression: Option<Box<CExpression>>,
        typ: Option<TypeRef>,
        result_type: TypeRef,
    },
    /// Statement expression (GNU extension)
    StatementExpression(Box<CStatement>),
    /// Generic selection (C11)
    GenericSelection {
        expression: Box<CExpression>,
        associations: Vec<GenericAssoc>,
    },
}

/// Generic association for C11 generic selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericAssoc {
    pub type_match: Option<TypeRef>,
    pub expression: CExpression,
}

/// C dialect verifier
pub struct CDialectVerifier {
    /// Context to verify against
    context: CDialectContext,
    /// Errors found
    errors: Vec<DialectError>,
}

impl CDialectVerifier {
    /// Create a new verifier
    pub fn new(context: CDialectContext) -> Self {
        Self {
            context,
            errors: vec![],
        }
    }

    /// Verify the entire context
    pub fn verify(&mut self) -> Vec<DialectError> {
        self.verify_declarations();
        self.verify_types();
        self.verify_type_consistency();
        self.errors.clone()
    }

    /// Verify all declarations
    fn verify_declarations(&mut self) {
        let mut seen: HashMap<String, DeclId> = HashMap::new();
        for (id, decl) in &self.context.declarations {
            if let Some(name) = decl.name() {
                if let Some(existing_id) = seen.get(name) {
                    if existing_id != id {
                        self.errors
                            .push(DialectError::DuplicateDefinition(name.to_string()));
                    }
                } else {
                    seen.insert(name.to_string(), *id);
                }
            }
        }

        for decl in self.context.declarations.values() {
            if let CDeclaration::Function(f) = decl {
                if f.has_body && f.storage_class == CStorageClass::Extern {
                    self.errors
                        .push(DialectError::LinkageConflict(f.name.clone()));
                }
            }
        }
    }

    /// Verify all types
    fn verify_types(&mut self) {
        for (type_ref, typ) in &self.context.types {
            if typ.is_incomplete() {
                let is_struct_or_union = matches!(
                    self.context.get_type(type_ref),
                    Some(CType::Struct(_)) | Some(CType::Union(_))
                );
                if !is_struct_or_union {
                    self.errors
                        .push(DialectError::IncompleteType(format!("{:?}", type_ref)));
                }
            }
        }
    }

    /// Verify type consistency across declarations and expressions
    fn verify_type_consistency(&mut self) {
        // Collect all type refs that need verification first
        let mut type_refs_to_check: Vec<(TypeRef, String)> = Vec::new();

        for decl in self.context.declarations.values() {
            match decl {
                CDeclaration::Function(f) => {
                    type_refs_to_check.push((f.return_type, "function return type".to_string()));
                    for param in &f.params {
                        type_refs_to_check.push((param.typ, "parameter type".to_string()));
                    }
                }
                CDeclaration::GlobalVariable(g) => {
                    type_refs_to_check.push((g.typ, "global variable type".to_string()));
                }
                CDeclaration::Struct(s) => {
                    for field in &s.fields {
                        type_refs_to_check.push((field.typ, "struct field type".to_string()));
                    }
                }
                CDeclaration::Union(u) => {
                    for field in &u.variants {
                        type_refs_to_check.push((field.typ, "union field type".to_string()));
                    }
                }
                CDeclaration::Typedef(t) => {
                    type_refs_to_check
                        .push((t.underlying_type, "typedef underlying type".to_string()));
                }
                _ => {}
            }
        }

        // Now verify all collected type refs
        for (type_ref, context) in type_refs_to_check {
            self.verify_type_completeness(type_ref, &context);
        }
    }

    /// Verify function parameters have complete types
    fn verify_function_params(&mut self, params: &[CParamDecl]) {
        for param in params {
            self.verify_type_completeness(param.typ, "parameter type");
        }
    }

    /// Verify a type reference is complete where required
    fn verify_type_completeness(&mut self, type_ref: TypeRef, context: &str) {
        if let Some(typ) = self.context.get_type(&type_ref) {
            if typ.is_incomplete() {
                self.errors.push(DialectError::IncompleteType(format!(
                    "{} requires complete type: {:?}",
                    context, type_ref
                )));
            }
        }
    }

    /// Verify a statement for correctness
    pub fn verify_statement(&mut self, _stmt: &CStatement) {
        // Statement verification would check:
        // - Return type matches function signature
        // - Assignment type compatibility
        // - Control flow validity
        // For now, this is a stub that can be extended with deeper checks
    }

    /// Verify an expression for correctness
    pub fn verify_expression(&mut self, _expr: &CExpression) {
        // Expression verification would check:
        // - Operand types match operator requirements
        // - Function call argument count matches parameter count
        // - Field access is on struct/union type
        // For now, this is a stub that can be extended with deeper checks
    }

    /// Get all errors
    pub fn errors(&self) -> &[DialectError] {
        &self.errors
    }
}

/// C dialect builder
pub struct CDialectBuilder {
    context: CDialectContext,
    layout_extractor: CLayoutExtractor,
}

impl CDialectBuilder {
    /// Create a new builder
    pub fn new(target_triple: &str) -> Self {
        Self {
            context: CDialectContext::default(),
            layout_extractor: CLayoutExtractor::new(target_triple),
        }
    }

    /// Add a declaration
    pub fn add_declaration(&mut self, decl: CDeclaration) -> DeclId {
        self.context.add_declaration(decl)
    }

    /// Add a type
    pub fn add_type(&mut self, type_ref: TypeRef, typ: CType) {
        self.context.add_type(type_ref, typ);
    }

    /// Build the context
    pub fn build(self) -> CDialectContext {
        self.context
    }

    /// Get the layout extractor
    pub fn layout_extractor(&self) -> &CLayoutExtractor {
        &self.layout_extractor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdeclaration_name() {
        let func = CDeclaration::Function(CFunctionDecl {
            id: DeclId(0),
            name: "my_func".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(4),
            attributes: vec![],
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: true,
            is_inline: false,
        });

        assert_eq!(func.name(), Some("my_func"));
    }

    #[test]
    fn test_ctype_is_incomplete() {
        let incomplete = CType::Incomplete;
        assert!(incomplete.is_incomplete());

        let pointer = CType::Pointer {
            pointee: TypeRef(0),
            constness: 0,
            nullability: PointerNullability::Raw,
        };
        assert!(!pointer.is_incomplete());
    }

    #[test]
    fn test_ctype_is_scalar() {
        assert!(CType::Int { is_signed: true }.is_scalar());
        assert!(CType::Pointer {
            pointee: TypeRef(0),
            constness: 0,
            nullability: PointerNullability::Raw
        }
        .is_scalar());
        assert!(!CType::Struct(TypeRef(0)).is_scalar());
    }

    #[test]
    fn test_cdialect_context_lookup() {
        let mut ctx = CDialectContext::default();
        let func = CDeclaration::Function(CFunctionDecl {
            id: DeclId(0),
            name: "test_func".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(4),
            attributes: vec![],
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        });

        ctx.add_declaration(func);
        assert!(ctx.lookup("test_func").is_some());
        assert!(ctx.lookup("nonexistent").is_none());
    }

    #[test]
    fn test_cdialect_verifier_no_errors() {
        let ctx = CDialectContext::default();
        let mut verifier = CDialectVerifier::new(ctx);
        verifier.verify();
        assert!(verifier.errors().is_empty());
    }

    #[test]
    fn test_cdialect_verifier_duplicate() {
        let mut ctx = CDialectContext::default();
        let func1 = CDeclaration::Function(CFunctionDecl {
            id: DeclId(0),
            name: "duplicate".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(4),
            attributes: vec![],
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: true,
            is_inline: false,
        });
        let func2 = CDeclaration::Function(CFunctionDecl {
            id: DeclId(1),
            name: "duplicate".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(4),
            attributes: vec![],
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 5,
                col: 1,
                byte_offset: 50,
                byte_length: 10,
            },
            has_body: true,
            is_inline: false,
        });

        ctx.add_declaration(func1);
        ctx.add_declaration(func2);

        let mut verifier = CDialectVerifier::new(ctx);
        verifier.verify();
        assert!(!verifier.errors().is_empty());
    }

    #[test]
    fn test_cunary_operator_variants() {
        assert_eq!(
            std::mem::discriminant(&CUnaryOperator::Plus),
            std::mem::discriminant(&CUnaryOperator::Plus)
        );
        assert_ne!(
            std::mem::discriminant(&CUnaryOperator::Plus),
            std::mem::discriminant(&CUnaryOperator::Minus)
        );
    }

    #[test]
    fn test_cbinary_operator_variants() {
        assert_eq!(
            std::mem::discriminant(&CBinaryOperator::Add),
            std::mem::discriminant(&CBinaryOperator::Add)
        );
        assert_ne!(
            std::mem::discriminant(&CBinaryOperator::Add),
            std::mem::discriminant(&CBinaryOperator::Sub)
        );
    }

    #[test]
    fn test_atomic_ordering_variants() {
        assert_eq!(
            std::mem::discriminant(&AtomicOrdering::Relaxed),
            std::mem::discriminant(&AtomicOrdering::Relaxed)
        );
        assert_ne!(
            std::mem::discriminant(&AtomicOrdering::Relaxed),
            std::mem::discriminant(&AtomicOrdering::SeqCst)
        );
    }

    #[test]
    fn test_trust_kind_variants() {
        let kind = TrustKind::UnsafeOperation;
        assert_eq!(kind, TrustKind::UnsafeOperation);
    }

    #[test]
    fn test_unsafe_operation_kind_variants() {
        let kind = UnsafeOperationKind::PointerDeref;
        assert_eq!(kind, UnsafeOperationKind::PointerDeref);
    }

    #[test]
    fn test_cdialect_builder_new() {
        let builder = CDialectBuilder::new("x86_64-unknown-linux-gnu");
        assert!(builder.context.declarations.is_empty());
    }

    #[test]
    fn test_cdialect_builder_add_declaration() {
        let mut builder = CDialectBuilder::new("x86_64-unknown-linux-gnu");
        let func = CDeclaration::Function(CFunctionDecl {
            id: DeclId(0),
            name: "added_func".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(4),
            attributes: vec![],
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        });

        let id = builder.add_declaration(func);
        assert_eq!(id, DeclId(0));
    }

    #[test]
    fn test_pointer_nullability_serialization() {
        let nullability = PointerNullability::Nullable;
        let json = serde_json::to_string(&nullability).unwrap();
        assert!(json.contains("nullable"));
    }

    #[test]
    fn test_cstorage_class_serialization() {
        let sc = CStorageClass::Static;
        let json = serde_json::to_string(&sc).unwrap();
        assert!(json.contains("static"));
    }

    #[test]
    fn test_cstatement_null() {
        let stmt = CStatement::Null;
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("Null"));
    }

    #[test]
    fn test_cstatement_return() {
        let stmt = CStatement::Return(None);
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("Return"));
    }

    #[test]
    fn test_cstatement_return_with_expr() {
        let ret_expr = CExpression::IntegerConstant {
            value: 42,
            typ: TypeRef(1),
        };
        let stmt = CStatement::Return(Some(ret_expr));
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("Return"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_cstatement_if() {
        let cond = CExpression::IntegerConstant {
            value: 1,
            typ: TypeRef(1),
        };
        let then_block = Box::new(CStatement::Null);
        let stmt = CStatement::If {
            condition: cond,
            then_block,
            else_block: None,
        };
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("If"));
    }

    #[test]
    fn test_cstatement_while() {
        let cond = CExpression::IntegerConstant {
            value: 1,
            typ: TypeRef(1),
        };
        let stmt = CStatement::While {
            condition: cond,
            body: Box::new(CStatement::Null),
        };
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("While"));
    }

    #[test]
    fn test_cstatement_for() {
        let stmt = CStatement::For {
            init: None,
            condition: None,
            increment: None,
            body: Box::new(CStatement::Null),
        };
        let json = serde_json::to_string(&stmt).unwrap();
        assert!(json.contains("For"));
    }

    #[test]
    fn test_cexpression_integer() {
        let expr = CExpression::IntegerConstant {
            value: 100,
            typ: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("100"));
    }

    #[test]
    fn test_cexpression_binary() {
        let left = CExpression::IntegerConstant {
            value: 1,
            typ: TypeRef(1),
        };
        let right = CExpression::IntegerConstant {
            value: 2,
            typ: TypeRef(1),
        };
        let expr = CExpression::Binary {
            operator: CBinaryOperator::Add,
            left: Box::new(left),
            right: Box::new(right),
            typ: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("Binary"));
        assert!(json.contains("Add"));
    }

    #[test]
    fn test_cexpression_call() {
        let func = CExpression::Variable {
            name: "printf".to_string(),
            typ: TypeRef(2),
        };
        let arg = CExpression::IntegerConstant {
            value: 0,
            typ: TypeRef(1),
        };
        let expr = CExpression::Call {
            function: Box::new(func),
            arguments: vec![arg],
            typ: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("Call"));
        assert!(json.contains("printf"));
    }

    #[test]
    fn test_cexpression_assignment() {
        let target = CExpression::Variable {
            name: "x".to_string(),
            typ: TypeRef(1),
        };
        let value = CExpression::IntegerConstant {
            value: 5,
            typ: TypeRef(1),
        };
        let expr = CExpression::Assignment {
            target: Box::new(target),
            value: Box::new(value),
            typ: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("Assignment"));
    }

    #[test]
    fn test_cexpression_field_access() {
        let target = CExpression::Variable {
            name: "point".to_string(),
            typ: TypeRef(1),
        };
        let expr = CExpression::FieldAccess {
            target: Box::new(target),
            field: "x".to_string(),
            typ: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("FieldAccess"));
        assert!(json.contains("x"));
    }

    #[test]
    fn test_cexpression_sizeof() {
        let expr = CExpression::Sizeof {
            expression: None,
            typ: Some(TypeRef(1)),
            result_type: TypeRef(1),
        };
        let json = serde_json::to_string(&expr).unwrap();
        assert!(json.contains("Sizeof"));
    }

    #[test]
    fn test_cstatement_break_continue() {
        let break_stmt = CStatement::Break;
        let continue_stmt = CStatement::Continue;
        let break_json = serde_json::to_string(&break_stmt).unwrap();
        let continue_json = serde_json::to_string(&continue_stmt).unwrap();
        assert!(break_json.contains("Break"));
        assert!(continue_json.contains("Continue"));
    }

    #[test]
    fn test_switch_case() {
        let case = SwitchCase {
            value: 42,
            statement: Box::new(CStatement::Null),
        };
        let json = serde_json::to_string(&case).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn test_generic_assoc() {
        let assoc = GenericAssoc {
            type_match: Some(TypeRef(1)),
            expression: CExpression::IntegerConstant {
                value: 10,
                typ: TypeRef(1),
            },
        };
        let json = serde_json::to_string(&assoc).unwrap();
        assert!(json.contains("10"));
    }
}
