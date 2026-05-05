//! Chimera C to ChimeraIR lowering crate.
//!
//! Converts C dialect representations into ChimeraIR textual format
//! and Chimera MLIR ops/types using compiler-core definitions.
//!
//! Task 16: C dialect → ChimeraIR lowering crate
//! Task 101: Emit Chimera MLIR from C

pub mod mlir_emitter;

use chimera_c_dialect::{CDeclaration, CDialectContext, CStorageClass, CType};
use chimera_c_schema::{CchMeta, TypeRef};
use chimera_meta::{
    ImportMetadata, LayoutMetadata, Metadata, Signature, SourceLanguage, TrustAssumptionMetadata,
    Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Macro expansion entry in the expansion stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExpansion {
    /// Name of the macro that was expanded
    pub macro_name: String,
    /// Location where the macro was invoked
    pub invocation_location: SourceLocation,
    /// Location of the macro definition
    pub definition_location: Option<SourceLocation>,
}

/// A point in source code with file and position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file path
    pub file: String,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub col: u32,
    /// Byte offset from start of file
    pub byte_offset: u64,
}

/// Macro expansion stack tracking macro provenance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MacroExpansionStack {
    /// Stack of macro expansions, innermost first
    expansions: Vec<MacroExpansion>,
}

impl MacroExpansionStack {
    /// Push a macro expansion onto the stack
    pub fn push(&mut self, expansion: MacroExpansion) {
        self.expansions.push(expansion);
    }

    /// Pop the innermost macro expansion
    pub fn pop(&mut self) -> Option<MacroExpansion> {
        self.expansions.pop()
    }

    /// Get all expansions in order (innermost to outermost)
    pub fn expansions(&self) -> &[MacroExpansion] {
        &self.expansions
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.expansions.is_empty()
    }

    /// Format as ChimeraIR comment string
    pub fn to_chimera_comment(&self) -> String {
        if self.expansions.is_empty() {
            return String::new();
        }
        let mut lines = Vec::new();
        lines.push("  // Macro expansion stack:".to_string());
        for (i, exp) in self.expansions.iter().enumerate() {
            lines.push(format!(
                "  //   {i}: {name} at {file}:{line}:{col}",
                i = i,
                name = exp.macro_name,
                file = exp.invocation_location.file,
                line = exp.invocation_location.line,
                col = exp.invocation_location.col
            ));
        }
        lines.join("\n")
    }
}

/// Result type for lowering operations
pub type Result<T> = std::result::Result<T, LoweringError>;

/// Lowering errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum LoweringError {
    #[error("unsupported declaration: {0}")]
    UnsupportedDeclaration(String),
    #[error("unsupported type: {0}")]
    UnsupportedType(String),
    #[error("incomplete type: {0}")]
    IncompleteType(String),
    #[error("missing symbol: {0}")]
    MissingSymbol(String),
    #[error("lowering failed: {0}")]
    LoweringFailed(String),
}

/// C to ChimeraIR lowering context
#[derive(Debug, Clone, Default)]
pub struct CLoweredModule {
    /// Module name
    pub name: String,
    /// Lowered operations as text
    pub operations: Vec<String>,
    /// Symbol map: C name -> ChimeraIR name
    pub symbol_map: HashMap<String, String>,
    /// Type map: C TypeRef -> ChimeraIR type
    pub type_map: HashMap<TypeRef, String>,
    /// Source mappings
    pub source_locations: HashMap<String, chimera_c_schema::SourceSpan>,
    /// Target info
    pub target: CTargetInfo,
    /// Macro expansion provenance: symbol -> expansion stack
    pub macro_provenance: HashMap<String, MacroExpansionStack>,
    /// Include chain provenance: symbol -> list of included headers
    pub include_chain: HashMap<String, Vec<String>>,
}

impl CLoweredModule {
    /// Add macro provenance comment to an operation string
    pub fn add_macro_comment(&self, symbol: &str, op: &str) -> String {
        if let Some(stack) = self.macro_provenance.get(symbol) {
            if !stack.is_empty() {
                return format!("{}\n{}", op, stack.to_chimera_comment());
            }
        }
        op.to_string()
    }
}

/// Target information for lowering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTargetInfo {
    pub triple: String,
    pub pointer_width: u32,
    pub size_of_ptr: u32,
    pub size_of_long: u32,
    pub size_of_int: u32,
    pub little_endian: bool,
}

impl Default for CTargetInfo {
    fn default() -> Self {
        Self {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 8,
            size_of_int: 4,
            little_endian: true,
        }
    }
}

/// C to ChimeraIR lowering engine
#[derive(Debug, Clone)]
pub struct CLoweringEngine {
    ctx: CDialectContext,
    target: CTargetInfo,
    symbol_map: HashMap<String, String>,
    type_map: HashMap<TypeRef, String>,
    next_id: u64,
}

impl CLoweringEngine {
    /// Create a new lowering engine with dialect context
    pub fn new(ctx: CDialectContext) -> Self {
        Self {
            ctx,
            target: CTargetInfo::default(),
            symbol_map: HashMap::new(),
            type_map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Set target info
    pub fn with_target(mut self, target: CTargetInfo) -> Self {
        self.target = target;
        self
    }

    /// Generate a unique ID
    fn fresh_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("ch_{id}")
    }

    /// Map a C name to a ChimeraIR name
    fn mangle_name(&mut self, name: &str) -> String {
        let sanitized = name.replace(['.', '$', '#', ':'], "_");
        if let Some(existing) = self.symbol_map.get(name) {
            return existing.clone();
        }
        let ir_name = format!("c_{sanitized}");
        self.symbol_map.insert(name.to_string(), ir_name.clone());
        ir_name
    }

    /// Get the ChimeraIR integer type for a C primitive with target-aware width
    fn map_primitive_int(&self, is_signed: bool, bytes: u32) -> String {
        match bytes {
            1 => {
                if is_signed {
                    "si8"
                } else {
                    "ui8"
                }
            }
            2 => {
                if is_signed {
                    "si16"
                } else {
                    "ui16"
                }
            }
            4 => {
                if is_signed {
                    "si32"
                } else {
                    "ui32"
                }
            }
            8 => {
                if is_signed {
                    "si64"
                } else {
                    "ui64"
                }
            }
            16 => {
                if is_signed {
                    "si128"
                } else {
                    "ui128"
                }
            }
            _ => {
                if is_signed {
                    "si32"
                } else {
                    "ui32"
                }
            }
        }
        .to_string()
    }

    /// Get the ChimeraIR type for a C char with target-aware signedness
    fn map_char_type(&self, is_signed: bool) -> String {
        // C char is always 1 byte, but signedness depends on platform
        if is_signed {
            "si8".to_string()
        } else {
            "ui8".to_string()
        }
    }

    /// Get the ChimeraIR type for a C short (always 2 bytes)
    fn map_short_type(&self, is_signed: bool) -> String {
        self.map_primitive_int(is_signed, 2)
    }

    /// Get the ChimeraIR type for a C int (target-aware size)
    fn map_int_type(&self, is_signed: bool) -> String {
        self.map_primitive_int(is_signed, self.target.size_of_int)
    }

    /// Get the ChimeraIR type for a C long (target-aware size)
    fn map_long_type(&self, is_signed: bool) -> String {
        self.map_primitive_int(is_signed, self.target.size_of_long)
    }

    /// Get the ChimeraIR floating point type for C float/double/long double
    fn map_float_type(&self, typ: &CType) -> String {
        match typ {
            CType::Float => "f32".to_string(),
            CType::Double => "f64".to_string(),
            CType::LongDouble => "f80".to_string(),
            _ => "f64".to_string(),
        }
    }

    /// Lower the entire dialect context to ChimeraIR
    pub fn lower_module(&mut self) -> Result<CLoweredModule> {
        let mut operations = Vec::new();

        // Add target operation with full target information
        operations.push(format!(
            "// ChimeraIR C lowering - target: {}\n\
            chimera.target @\"{}\" ({{\n\
                pointer_width = {}\n\
                size_of_ptr = {}\n\
                size_of_long = {}\n\
                size_of_int = {}\n\
                little_endian = {}\n\
            }})",
            self.target.triple,
            self.target.triple,
            self.target.pointer_width,
            self.target.size_of_ptr,
            self.target.size_of_long,
            self.target.size_of_int,
            self.target.little_endian
        ));

        // Add module operation with name and language attribution
        operations.push(format!(
            "chimera.module @\"c_module\" {{\n\
                source_language = \"c\"\n\
                source_target = \"{}\"",
            self.target.triple
        ));

        // Lower all declarations - collect first, then lower
        let declarations: Vec<_> = self.ctx.declarations.values().cloned().collect();
        for decl in declarations {
            let op = self.lower_declaration(&decl)?;
            operations.push(op);
        }

        // Close module operation
        operations.push("}".to_string());

        Ok(CLoweredModule {
            name: "c_module".to_string(),
            operations,
            symbol_map: std::mem::take(&mut self.symbol_map),
            type_map: std::mem::take(&mut self.type_map),
            source_locations: HashMap::new(),
            target: self.target.clone(),
            macro_provenance: HashMap::new(),
            include_chain: HashMap::new(),
        })
    }

    /// Lower a single declaration
    fn lower_declaration(&mut self, decl: &CDeclaration) -> Result<String> {
        match decl {
            CDeclaration::Function(func) => self.lower_function(func),
            CDeclaration::GlobalVariable(var) => self.lower_global(var),
            CDeclaration::Struct(s) => self.lower_struct(s),
            CDeclaration::Union(u) => self.lower_union(u),
            CDeclaration::Enum(e) => self.lower_enum(e),
            CDeclaration::Typedef(t) => self.lower_typedef(t),
            CDeclaration::EnumConstant(c) => self.lower_enum_constant(c),
            CDeclaration::Macro(m) => self.lower_macro(m),
        }
    }

    /// Lower a function declaration
    fn lower_function(&mut self, func: &chimera_c_dialect::CFunctionDecl) -> Result<String> {
        let ir_name = self.mangle_name(&func.name);

        // Lower return type
        let _ = self.lower_type(&func.return_type)?;
        let ret_type_str = self
            .type_map
            .get(&func.return_type)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        // Lower parameters
        let mut params = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            let _ = self.lower_type(&param.typ);
            let ptype_str = self
                .type_map
                .get(&param.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            let pname = if param.name.is_empty() {
                format!("arg_{}", i)
            } else {
                param.name.clone()
            };
            params.push(format!("{pname}: {ptype_str}"));
        }

        // Build function operation
        let linkage = match func.linkage {
            chimera_c_dialect::CDeclarationLinkage::External => "external",
            chimera_c_dialect::CDeclarationLinkage::Internal => "internal",
            chimera_c_dialect::CDeclarationLinkage::None => "internal",
            chimera_c_dialect::CDeclarationLinkage::Weak => "weak",
        };

        let attrs = if func.is_inline {
            " { always_inline = true".to_string()
        } else {
            String::new()
        };

        // Detect error convention from function name patterns
        let error_effect = self.detect_error_convention(&func.name);
        let effect_str = if !error_effect.is_empty() {
            if attrs.is_empty() {
                format!(" {{ {error_effect}")
            } else {
                format!(", {error_effect}")
            }
        } else if !attrs.is_empty() {
            "}".to_string()
        } else {
            String::new()
        };

        let params_str = params.join(", ");

        Ok(format!(
            "  func.{linkage} @{ir_name}({params_str}) -> {ret_type_str}{attrs}{effect_str} {{"
        ))
    }

    /// Detect error convention from function name
    fn detect_error_convention(&self, name: &str) -> String {
        // Detect common C error convention patterns
        if name.ends_with("_err") || name.ends_with("_result") {
            "may_error = true".to_string()
        } else if name.starts_with("try_") {
            "may_error = true".to_string()
        } else if name.contains("init") || name.contains("open") || name.contains("connect") {
            // Common functions that can fail
            "may_error = true".to_string()
        } else {
            String::new()
        }
    }

    /// Lower a global variable
    fn lower_global(&mut self, var: &chimera_c_dialect::CGlobalVarDecl) -> Result<String> {
        let ir_name = self.mangle_name(&var.name);
        let _ = self.lower_type(&var.typ)?;
        let vtype_str = self
            .type_map
            .get(&var.typ)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        let linkage = match var.linkage {
            chimera_c_dialect::CDeclarationLinkage::External => "external",
            chimera_c_dialect::CDeclarationLinkage::Internal => "internal",
            chimera_c_dialect::CDeclarationLinkage::None => "internal",
            chimera_c_dialect::CDeclarationLinkage::Weak => "weak",
        };

        let mutable_str = if var.storage_class == CStorageClass::Static {
            "false"
        } else {
            "true"
        };

        let thread_local_str = if var.is_thread_local {
            "thread_local = true, "
        } else {
            ""
        };

        let static_str = if var.is_static {
            "visibility = private, "
        } else {
            ""
        };

        // Initialization trust - globals without initializers need special handling
        let init_trust = if var.initializer.is_none()
            && var.linkage == chimera_c_dialect::CDeclarationLinkage::External
        {
            "init_trust = uninitialized, "
        } else {
            ""
        };

        Ok(format!(
            "  global @{ir_name} {linkage} : {vtype_str} {thread_local_str}{static_str}{init_trust}mutable = {mutable_str}"
        ))
    }

    /// Lower a struct declaration
    fn lower_struct(&mut self, s: &chimera_c_dialect::CStructDecl) -> Result<String> {
        let name = s.name.as_deref().unwrap_or("unnamed_struct");
        let ir_name = self.mangle_name(name);

        if s.is_incomplete {
            return Ok(format!(
                "  chimera.type @\"{ir_name}\" {{ is_incomplete = true }}"
            ));
        }

        // Build struct type from fields
        let mut field_types = Vec::new();
        let mut total_size: u64 = 0;
        let mut max_align: u32 = 1;
        let mut field_layouts = Vec::new();

        for field in &s.fields {
            let _ = self.lower_type(&field.typ);
            let ftype_str = self
                .type_map
                .get(&field.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            field_types.push(format!(
                "{fname}: {ftype_str}",
                fname = field.name,
                ftype_str = ftype_str
            ));
            total_size = field.offset + field.size;
            max_align = max_align.max(field.align);
            field_layouts.push(format!(
                "{}: offset={}, size={}, align={}",
                field.name, field.offset, field.size, field.align
            ));
        }

        let fields_str = field_types.join(", ");
        let size_str = format!("size = {}", total_size);
        let align_str = format!("align = {}", max_align);
        let packed_str = if s.is_packed { "packed = true, " } else { "" };
        let pack_align_str = s
            .pack_align
            .map_or(String::new(), |a| format!("pack_align = {}, ", a));

        // Generate ABI hash from field layout
        let abi_hash = self.compute_struct_abi_hash(name, &field_layouts);

        Ok(format!(
            "  chimera.type @\"{ir_name}\" {{ {packed_str}{pack_align_str}fields = [{fields_str}], {size_str}, {align_str}, abi_hash = \"{abi_hash}\" }}"
        ))
    }

    /// Compute a simple ABI hash from struct name and field layouts using BLAKE3
    fn compute_struct_abi_hash(&self, name: &str, field_layouts: &[String]) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-struct-abi");
        hasher.update_str(name);
        for layout in field_layouts {
            hasher.update_str(layout);
        }
        hasher.finalize().as_hex()[..16].to_string()
    }

    /// Lower a union declaration
    fn lower_union(&mut self, u: &chimera_c_dialect::CUnionDecl) -> Result<String> {
        let name = u.name.as_deref().unwrap_or("unnamed_union");
        let ir_name = self.mangle_name(name);

        if u.is_incomplete {
            return Ok(format!(
                "  chimera.type @\"{ir_name}\" {{ is_incomplete = true }}"
            ));
        }

        // Build union type - all fields at offset 0
        let mut field_types = Vec::new();
        for field in &u.variants {
            let _ = self.lower_type(&field.typ);
            let ftype_str = self
                .type_map
                .get(&field.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            field_types.push(format!("{name}: {ftype_str}", name = field.name));
        }

        let fields_str = field_types.join(", ");
        let size_str = format!("size = {}", u.size);
        let align_str = format!("align = {}", u.align);

        // Unions crossing ABI boundary require explicit trust contract
        // The trust_assumption indicates unsafe union access
        Ok(format!(
            "  chimera.type @\"{ir_name}\" {{ union_fields = [{fields_str}], {size_str}, {align_str}, requires_trust = true }}"
        ))
    }

    /// Lower an enum declaration
    fn lower_enum(&mut self, e: &chimera_c_dialect::CEnumDecl) -> Result<String> {
        let name = e.name.as_deref().unwrap_or("unnamed_enum");
        let ir_name = self.mangle_name(name);

        // Lower underlying type
        let _ = self.lower_type(&e.underlying_type)?;
        let underlying = self
            .type_map
            .get(&e.underlying_type)
            .cloned()
            .unwrap_or_else(|| "i32".to_string());

        // Lower enum constants
        let mut consts = Vec::new();
        for c in &e.constants {
            let value_str = c.value.map_or("0".to_string(), |v| v.to_string());
            consts.push(format!("{} = {}", c.name, value_str));
        }
        let consts_str = consts.join(", ");

        Ok(format!(
            "  chimera.type @\"{ir_name}\" {{ is_enum = true, underlying = {underlying}, constants = [{consts_str}] }}"
        ))
    }

    /// Lower a typedef
    fn lower_typedef(&mut self, t: &chimera_c_dialect::CTypedefDecl) -> Result<String> {
        let ir_name = self.mangle_name(&t.name);
        let _ = self.lower_type(&t.underlying_type)?;
        let source_str = self
            .type_map
            .get(&t.underlying_type)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        Ok(format!("  chimera.typedef @\"{ir_name}\" = {source_str}"))
    }

    /// Lower an enum constant
    fn lower_enum_constant(&mut self, c: &chimera_c_dialect::CEnumConstant) -> Result<String> {
        let ir_name = self.mangle_name(&c.name);

        let value_str = match c.value {
            Some(v) => v.to_string(),
            None => "0".to_string(),
        };

        Ok(format!("  chimera.enum.const @{ir_name} = {value_str}"))
    }

    /// Lower a macro declaration
    fn lower_macro(&mut self, m: &chimera_c_dialect::CMacroDecl) -> Result<String> {
        // Macros are expanded during parsing; represent as constant if possible
        if let Some(value) = &m.value {
            let ir_name = self.mangle_name(&m.name);
            Ok(format!("  chimera.constant @{ir_name} = {value}"))
        } else {
            Ok(format!(
                "  // macro {name} (no constant value)",
                name = m.name
            ))
        }
    }

    /// Lower a type reference to ChimeraIR type string
    fn lower_type(&mut self, type_ref: &TypeRef) -> Result<()> {
        if self.type_map.contains_key(type_ref) {
            return Ok(());
        }

        // Get the type - use pattern matching to extract nested TypeRefs first
        // This avoids holding a borrow across recursive calls
        let nested_refs: Vec<TypeRef> = {
            let typ = self
                .ctx
                .get_type(type_ref)
                .ok_or_else(|| LoweringError::MissingSymbol(format!("type ref {:?}", type_ref)))?;
            self.extract_nested_type_refs(typ)
        };

        // Recursively lower nested types first
        for nested in &nested_refs {
            self.lower_type(nested)?;
        }

        // Now get the type again and build the type string
        let typ = self
            .ctx
            .get_type(type_ref)
            .ok_or_else(|| LoweringError::MissingSymbol(format!("type ref {:?}", type_ref)))?;

        let type_str = match typ {
            CType::Void => "()".to_string(),
            CType::Bool => "i1".to_string(),
            CType::Char { is_signed } => self.map_char_type(*is_signed),
            CType::Short { is_signed } => self.map_short_type(*is_signed),
            CType::Int { is_signed } => self.map_int_type(*is_signed),
            CType::Long { is_signed } => self.map_long_type(*is_signed),
            CType::LongLong { is_signed } => self.map_primitive_int(*is_signed, 8),
            CType::Float => self.map_float_type(typ),
            CType::Double => self.map_float_type(typ),
            CType::LongDouble => self.map_float_type(typ),
            CType::Pointer {
                pointee,
                constness: _,
                nullability,
            } => {
                let base_str = self
                    .type_map
                    .get(pointee)
                    .cloned()
                    .unwrap_or_else(|| "()".to_string());
                let null_str = match nullability {
                    chimera_c_abi::PointerNullability::Nullable => "nullable",
                    chimera_c_abi::PointerNullability::NonNull => "nonnull",
                    chimera_c_abi::PointerNullability::Raw => "raw",
                    chimera_c_abi::PointerNullability::Borrowed => "borrowed",
                    chimera_c_abi::PointerNullability::BorrowedMut => "borrowed_mut",
                };
                format!("!ch.ptr<{base_str}, null={null_str}>")
            }
            CType::Array { element, length } => {
                let elem_str = self
                    .type_map
                    .get(element)
                    .cloned()
                    .unwrap_or_else(|| "()".to_string());
                match length {
                    Some(len) => format!("!ch.array<{elem_str}, {len}>"),
                    None => format!("!ch.array<{elem_str}, ?>"),
                }
            }
            CType::FunctionPointer { params, ret, cconv } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| {
                        self.type_map
                            .get(p)
                            .cloned()
                            .unwrap_or_else(|| "()".to_string())
                    })
                    .collect();
                let ret_str = ret
                    .as_ref()
                    .map(|r| {
                        self.type_map
                            .get(r)
                            .cloned()
                            .unwrap_or_else(|| "()".to_string())
                    })
                    .unwrap_or_else(|| "()".to_string());
                format!(
                    "!ch.func<({}), -> {ret_str}, cconv = {cconv}>",
                    param_strs.join(", ")
                )
            }
            CType::Struct(type_ref)
            | CType::Union(type_ref)
            | CType::Enum(type_ref)
            | CType::Typedef(type_ref) => self
                .type_map
                .get(type_ref)
                .cloned()
                .unwrap_or_else(|| "()".to_string()),
            CType::Volatile(inner) => {
                let inner_str = self
                    .type_map
                    .get(inner)
                    .cloned()
                    .unwrap_or_else(|| "()".to_string());
                format!("!ch.volatile<{inner_str}>")
            }
            CType::Atomic(inner) => {
                let inner_str = self
                    .type_map
                    .get(inner)
                    .cloned()
                    .unwrap_or_else(|| "()".to_string());
                format!("!ch.atomic<{inner_str}>")
            }
            CType::Incomplete => "!ch.incomplete".to_string(),
        };

        self.type_map.insert(*type_ref, type_str);
        Ok(())
    }

    /// Extract nested TypeRefs from a CType without holding a borrow
    fn extract_nested_type_refs(&self, typ: &CType) -> Vec<TypeRef> {
        let mut refs = Vec::new();
        match typ {
            CType::Pointer { pointee, .. } => refs.push(*pointee),
            CType::Array { element, .. } => refs.push(*element),
            CType::FunctionPointer { params, ret, .. } => {
                refs.extend(params.iter().copied());
                if let Some(r) = ret {
                    refs.push(*r);
                }
            }
            CType::Struct(t) | CType::Union(t) | CType::Enum(t) | CType::Typedef(t) => {
                refs.push(*t)
            }
            CType::Volatile(inner) | CType::Atomic(inner) => refs.push(*inner),
            _ => {}
        }
        refs
    }
}

/// Emit lowered module as textual ChimeraIR
pub fn emit_chimera(ctx: CDialectContext) -> Result<String> {
    let mut engine = CLoweringEngine::new(ctx);
    let module = engine.lower_module()?;
    Ok(module.operations.join("\n"))
}

/// Emit lowered module as structured result
pub fn lower_to_module(ctx: CDialectContext) -> Result<CLoweredModule> {
    let mut engine = CLoweringEngine::new(ctx);
    engine.lower_module()
}

/// Emit C-specific metadata as common `.chmeta`
pub fn emit_metadata(cchmeta: &CchMeta) -> Metadata {
    let mut metadata = Metadata::default();
    metadata.version = Version::new(1, 0, 0);

    // Convert C ABI facts to imports
    for fact in &cchmeta.c_abi_facts {
        let import = ImportMetadata {
            symbol: fact.symbol.clone(),
            signature: Signature {
                cconv: chimera_meta::CallingConvention::C,
                params: fact.params.iter().map(|p| format!(":{}", p.size)).collect(),
                return_type: fact.ret.as_ref().map(|r| format!(":{}", r.size)),
            },
            language: SourceLanguage::C,
            target: Default::default(),
            errno_mapping: None,
            requires_drop: false,
        };
        metadata.imports.push(import);
    }

    // Convert layout facts to layout metadata
    for fact in &cchmeta.layout_facts {
        let layout = LayoutMetadata {
            name: fact.type_name.clone(),
            size: fact.size,
            align: fact.align as u64,
            fields: fact
                .fields
                .iter()
                .map(|f| chimera_meta::FieldLayout {
                    name: f.name.clone(),
                    offset: f.offset,
                    size: f.size,
                    typ: Default::default(),
                    align: f.align as u64,
                })
                .collect(),
            is_packed: fact.is_packed,
        };
        metadata.layouts.push(layout);
    }

    // Add trust assumptions
    for assumption in &cchmeta.trust_assumptions {
        let kind = match assumption.kind {
            chimera_c_schema::CTrustKind::ClangAstInterpretation => {
                chimera_meta::TrustAssumptionKind::ManualProof
            }
            chimera_c_schema::CTrustKind::LayoutComputedByClang => {
                chimera_meta::TrustAssumptionKind::ManualProof
            }
            chimera_c_schema::CTrustKind::MacroExpansionCorrect => {
                chimera_meta::TrustAssumptionKind::ManualProof
            }
            chimera_c_schema::CTrustKind::TargetTripleCorrect => {
                chimera_meta::TrustAssumptionKind::TrustedFunction
            }
            chimera_c_schema::CTrustKind::StandardLibraryConformance => {
                chimera_meta::TrustAssumptionKind::TrustedForeignAbi
            }
        };
        metadata.trust_assumptions.push(TrustAssumptionMetadata {
            kind,
            description: assumption.description.clone(),
            external_ref: assumption.external_ref.clone(),
        });
    }

    metadata
}

/// Convert CchMeta to Chimera Metadata
pub fn convert_to_metadata(cchmeta: &CchMeta) -> Metadata {
    emit_metadata(cchmeta)
}

/// Emit C object file sidecar (`.cho`) with chimera-object format
pub fn emit_object_file(
    target: &str,
    payload: Vec<u8>,
    payload_kind: chimera_object::PayloadKind,
    cchmeta: &CchMeta,
) -> chimera_object::ObjectFile {
    let metadata = emit_metadata(cchmeta);
    chimera_object::ObjectFile::new(target.to_string(), payload, payload_kind, metadata)
}

/// Create a C object file with native payload
pub fn emit_c_object_file(
    target: &str,
    native_payload: Vec<u8>,
    cchmeta: &CchMeta,
) -> chimera_object::ObjectFile {
    emit_object_file(
        target,
        native_payload,
        chimera_object::PayloadKind::Native,
        cchmeta,
    )
}

/// Create a C object file with ChimeraIR textual payload
pub fn emit_c_ir_object_file(
    target: &str,
    chimera_ir_text: &str,
    cchmeta: &CchMeta,
) -> chimera_object::ObjectFile {
    let payload = chimera_ir_text.as_bytes().to_vec();
    emit_object_file(
        target,
        payload,
        chimera_object::PayloadKind::TextualIR,
        cchmeta,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_c_dialect::{
        CDeclaration, CDeclarationLinkage, CDialectContext, CFunctionDecl, CStorageClass,
    };
    use chimera_c_schema::{
        AbiParamInfo, AbiRetInfo, ArtifactHeader, CAbiFact, CLayoutFact, CTrustAssumption,
        CTrustKind, DeclProvenance, ExtractionMethod, IncludeDep, LayoutFieldFact, MacroDep,
        PassingConvention, SourceSpan, CURRENT_SCHEMA_VERSION, C_ARTIFACT_MAGIC,
    };
    use chimera_c_schema::{DeclId, TypeRef};

    #[test]
    fn test_lowering_engine_new() {
        let ctx = CDialectContext::default();
        let engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.next_id, 0);
    }

    #[test]
    fn test_fresh_id_generation() {
        let ctx = CDialectContext::default();
        let mut engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.fresh_id(), "ch_0");
        assert_eq!(engine.fresh_id(), "ch_1");
        assert_eq!(engine.fresh_id(), "ch_2");
    }

    #[test]
    fn test_mangle_name_simple() {
        let ctx = CDialectContext::default();
        let mut engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.mangle_name("foo"), "c_foo");
        assert_eq!(engine.mangle_name("bar"), "c_bar");
    }

    #[test]
    fn test_mangle_name_deduplication() {
        let ctx = CDialectContext::default();
        let mut engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.mangle_name("foo"), "c_foo");
        assert_eq!(engine.mangle_name("foo"), "c_foo"); // Same name returns same
    }

    #[test]
    fn test_mangle_name_special_chars() {
        let ctx = CDialectContext::default();
        let mut engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.mangle_name("foo.bar"), "c_foo_bar");
        assert_eq!(engine.mangle_name("ns::func"), "c_ns__func");
    }

    #[test]
    fn test_target_info_default() {
        let target = CTargetInfo::default();
        assert_eq!(target.pointer_width, 64);
        assert_eq!(target.size_of_ptr, 8);
        assert!(target.little_endian);
    }

    #[test]
    fn test_ctarget_info_custom() {
        let target = CTargetInfo {
            triple: "aarch64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 8,
            size_of_int: 4,
            little_endian: true,
        };
        assert_eq!(target.triple, "aarch64-unknown-linux-gnu");
    }

    #[test]
    fn test_lowering_empty_module() {
        let ctx = CDialectContext::default();
        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name, "c_module");
    }

    #[test]
    fn test_emit_chimera_empty() {
        let ctx = CDialectContext::default();
        let result = emit_chimera(ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("chimera.module"));
    }

    #[test]
    fn test_lowering_function_decl() {
        let mut ctx = CDialectContext::default();
        // Add type for return type TypeRef(4)
        ctx.types.insert(TypeRef(4), CType::Int { is_signed: true });

        let func = CFunctionDecl {
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
        };
        ctx.add_declaration(CDeclaration::Function(func));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.symbol_map.contains_key("test_func"));
    }

    #[test]
    fn test_lowering_with_target() {
        let ctx = CDialectContext::default();
        let target = CTargetInfo {
            triple: "aarch64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 8,
            size_of_int: 4,
            little_endian: true,
        };
        let result = CLoweringEngine::new(ctx).with_target(target).lower_module();
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.operations.iter().any(|op| op.contains("aarch64")));
    }

    #[test]
    fn test_lowering_error_missing_type() {
        let ctx = CDialectContext::default();
        let result = emit_chimera(ctx);
        assert!(result.is_ok()); // Empty module has no types to lower
    }

    #[test]
    fn test_symbol_map_populated() {
        let mut ctx = CDialectContext::default();
        // Add type for return type TypeRef(4)
        ctx.types.insert(TypeRef(4), CType::Int { is_signed: true });

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "my_function".to_string(),
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
        };
        ctx.add_declaration(CDeclaration::Function(func));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert!(module.symbol_map.contains_key("my_function"));
    }

    #[test]
    fn test_map_primitive_int_x86_64() {
        let ctx = CDialectContext::default();
        let engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.map_primitive_int(true, 4), "si32");
        assert_eq!(engine.map_primitive_int(false, 4), "ui32");
        assert_eq!(engine.map_primitive_int(true, 8), "si64");
        assert_eq!(engine.map_primitive_int(false, 8), "ui64");
    }

    #[test]
    fn test_map_primitive_int_windows() {
        let ctx = CDialectContext::default();
        let target = CTargetInfo {
            triple: "x86_64-pc-windows-msvc".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 4, // Windows long is 4 bytes
            size_of_int: 4,
            little_endian: true,
        };
        let engine = CLoweringEngine::new(ctx).with_target(target);
        assert_eq!(engine.map_primitive_int(true, 4), "si32");
        assert_eq!(engine.map_long_type(true), "si32"); // Windows long is 4 bytes
    }

    #[test]
    fn test_map_char_type() {
        let ctx = CDialectContext::default();
        let engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.map_char_type(true), "si8");
        assert_eq!(engine.map_char_type(false), "ui8");
    }

    #[test]
    fn test_map_float_type() {
        let ctx = CDialectContext::default();
        let engine = CLoweringEngine::new(ctx);
        assert_eq!(engine.map_float_type(&CType::Float), "f32");
        assert_eq!(engine.map_float_type(&CType::Double), "f64");
        assert_eq!(engine.map_float_type(&CType::LongDouble), "f80");
    }

    #[test]
    fn test_target_aware_int_lowering() {
        let ctx = CDialectContext::default();
        let target = CTargetInfo {
            triple: "aarch64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 8, // Linux/aarch64 long is 8 bytes
            size_of_int: 4,
            little_endian: true,
        };
        let engine = CLoweringEngine::new(ctx).with_target(target);
        // On aarch64 Linux, long is 8 bytes
        assert_eq!(engine.map_long_type(true), "si64");
        assert_eq!(engine.map_int_type(true), "si32");
    }

    #[test]
    fn test_pointer_lowering_with_nullability() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let ptr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            ptr_type,
            CType::Pointer {
                pointee: int_type,
                constness: 0,
                nullability: chimera_c_abi::PointerNullability::Nullable,
            },
        );

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_textual_chimera_output_format() {
        let ctx = CDialectContext::default();
        let result = emit_chimera(ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Verify enhanced output contains source language attribution
        assert!(output.contains("source_language = \"c\""));
        assert!(output.contains("source_target = "));
    }

    #[test]
    fn test_target_info_included_in_output() {
        let ctx = CDialectContext::default();
        let target = CTargetInfo {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            size_of_ptr: 8,
            size_of_long: 8,
            size_of_int: 4,
            little_endian: true,
        };
        let result = CLoweringEngine::new(ctx).with_target(target).lower_module();
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("size_of_int = 4"));
        assert!(output.contains("little_endian = true"));
    }

    #[test]
    fn test_pointer_lowering_with_ptr_type_registered() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let ptr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            ptr_type,
            CType::Pointer {
                pointee: int_type,
                constness: 1,
                nullability: chimera_c_abi::PointerNullability::NonNull,
            },
        );

        let mut engine = CLoweringEngine::new(ctx);
        // Manually lower the type to populate type_map
        let result = engine.lower_type(&ptr_type);
        assert!(result.is_ok());
        // Verify the pointer type was registered in type_map
        assert!(engine.type_map.contains_key(&ptr_type));
        let type_str = engine.type_map.get(&ptr_type).unwrap();
        assert!(type_str.contains("ch.ptr"));
    }

    #[test]
    fn test_struct_lowering_with_layout_metadata() {
        use chimera_c_dialect::{CDeclaration, CFieldDecl, CStructDecl};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        let char_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types
            .insert(char_type, CType::Char { is_signed: false });

        let struct_decl = CStructDecl {
            id: DeclId(0),
            name: Some("Point".to_string()),
            fields: vec![
                CFieldDecl {
                    name: "x".to_string(),
                    typ: int_type,
                    bitfield_width: None,
                    offset: 0,
                    size: 4,
                    align: 4,
                },
                CFieldDecl {
                    name: "y".to_string(),
                    typ: int_type,
                    bitfield_width: None,
                    offset: 4,
                    size: 4,
                    align: 4,
                },
            ],
            is_packed: false,
            pack_align: None,
            is_incomplete: false,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 32,
            },
        };

        ctx.add_declaration(CDeclaration::Struct(struct_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        // Verify struct output contains layout metadata
        assert!(output.contains("fields = ["));
        assert!(output.contains("size = 8"));
        assert!(output.contains("align = 4"));
        assert!(output.contains("abi_hash"));
    }

    #[test]
    fn test_struct_lowering_packed() {
        use chimera_c_dialect::{CDeclaration, CFieldDecl, CStructDecl};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let struct_decl = CStructDecl {
            id: DeclId(0),
            name: Some("PackedStruct".to_string()),
            fields: vec![
                CFieldDecl {
                    name: "a".to_string(),
                    typ: int_type,
                    bitfield_width: None,
                    offset: 0,
                    size: 1,
                    align: 1,
                },
                CFieldDecl {
                    name: "b".to_string(),
                    typ: int_type,
                    bitfield_width: None,
                    offset: 1,
                    size: 4,
                    align: 1,
                },
            ],
            is_packed: true,
            pack_align: None,
            is_incomplete: false,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 16,
            },
        };

        ctx.add_declaration(CDeclaration::Struct(struct_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("packed = true"));
    }

    #[test]
    fn test_array_lowering_with_length() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let arr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            arr_type,
            CType::Array {
                element: int_type,
                length: Some(10),
            },
        );

        let mut engine = CLoweringEngine::new(ctx);
        let result = engine.lower_type(&arr_type);
        assert!(result.is_ok());
        let type_str = engine.type_map.get(&arr_type).unwrap();
        assert!(type_str.contains("!ch.array"));
        assert!(type_str.contains("10"));
    }

    #[test]
    fn test_array_lowering_without_length() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let arr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            arr_type,
            CType::Array {
                element: int_type,
                length: None,
            },
        );

        let mut engine = CLoweringEngine::new(ctx);
        let result = engine.lower_type(&arr_type);
        assert!(result.is_ok());
        let type_str = engine.type_map.get(&arr_type).unwrap();
        assert!(type_str.contains("!ch.array"));
        assert!(type_str.contains("?"));
    }

    #[test]
    fn test_union_lowering_with_trust() {
        use chimera_c_dialect::{CDeclaration, CFieldDecl, CUnionDecl};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        let float_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(float_type, CType::Float);

        let union_decl = CUnionDecl {
            id: DeclId(0),
            name: Some("IntOrFloat".to_string()),
            variants: vec![
                CFieldDecl {
                    name: "as_int".to_string(),
                    typ: int_type,
                    bitfield_width: None,
                    offset: 0,
                    size: 4,
                    align: 4,
                },
                CFieldDecl {
                    name: "as_float".to_string(),
                    typ: float_type,
                    bitfield_width: None,
                    offset: 0,
                    size: 4,
                    align: 4,
                },
            ],
            size: 4,
            align: 4,
            is_incomplete: false,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 16,
            },
        };

        ctx.add_declaration(CDeclaration::Union(union_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("union_fields = ["));
        assert!(output.contains("requires_trust = true"));
    }

    #[test]
    fn test_enum_lowering_with_constants() {
        use chimera_c_dialect::{CDeclaration, CEnumConstant, CEnumDecl};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: false });

        let enum_decl = CEnumDecl {
            id: DeclId(0),
            name: Some("Color".to_string()),
            underlying_type: int_type,
            constants: vec![
                CEnumConstant {
                    id: DeclId(1),
                    name: "RED".to_string(),
                    value: Some(0),
                    source_span: chimera_c_schema::SourceSpan {
                        file: "test.c".to_string(),
                        line: 1,
                        col: 1,
                        byte_offset: 0,
                        byte_length: 4,
                    },
                },
                CEnumConstant {
                    id: DeclId(2),
                    name: "GREEN".to_string(),
                    value: Some(1),
                    source_span: chimera_c_schema::SourceSpan {
                        file: "test.c".to_string(),
                        line: 2,
                        col: 1,
                        byte_offset: 4,
                        byte_length: 4,
                    },
                },
                CEnumConstant {
                    id: DeclId(3),
                    name: "BLUE".to_string(),
                    value: Some(2),
                    source_span: chimera_c_schema::SourceSpan {
                        file: "test.c".to_string(),
                        line: 3,
                        col: 1,
                        byte_offset: 8,
                        byte_length: 4,
                    },
                },
            ],
            is_incomplete: false,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 12,
            },
        };

        ctx.add_declaration(CDeclaration::Enum(enum_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("is_enum = true"));
        assert!(output.contains("constants = ["));
        assert!(output.contains("RED = 0"));
        assert!(output.contains("GREEN = 1"));
        assert!(output.contains("BLUE = 2"));
    }

    #[test]
    fn test_function_pointer_lowering() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let void_type = TypeRef(2);
        let fp_type = TypeRef(3);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(void_type, CType::Void);
        ctx.types.insert(
            fp_type,
            CType::FunctionPointer {
                params: vec![int_type, int_type],
                ret: Some(void_type),
                cconv: "cdecl".to_string(),
            },
        );

        let mut engine = CLoweringEngine::new(ctx);
        let result = engine.lower_type(&fp_type);
        assert!(result.is_ok());
        let type_str = engine.type_map.get(&fp_type).unwrap();
        assert!(type_str.contains("!ch.func"));
        assert!(type_str.contains("cconv"));
    }

    #[test]
    fn test_global_lowering_with_thread_local() {
        use chimera_c_dialect::{CDeclaration, CGlobalVarDecl, CStorageClass};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let global_decl = CGlobalVarDecl {
            id: DeclId(0),
            name: "thread_counter".to_string(),
            linkage: chimera_c_dialect::CDeclarationLinkage::Internal,
            storage_class: CStorageClass::Static,
            typ: int_type,
            initializer: None,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 16,
            },
            is_thread_local: true,
            is_static: true,
        };

        ctx.add_declaration(CDeclaration::GlobalVariable(global_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("thread_local = true"));
        assert!(output.contains("visibility = private"));
        assert!(output.contains("mutable = false"));
    }

    #[test]
    fn test_global_lowering_extern_uninitialized() {
        use chimera_c_dialect::{CDeclaration, CDeclarationLinkage, CGlobalVarDecl, CStorageClass};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        let ptr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            ptr_type,
            CType::Pointer {
                pointee: int_type,
                constness: 0,
                nullability: chimera_c_abi::PointerNullability::Nullable,
            },
        );

        let global_decl = CGlobalVarDecl {
            id: DeclId(0),
            name: "external_pointer".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::Extern,
            typ: ptr_type,
            initializer: None,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 16,
            },
            is_thread_local: false,
            is_static: false,
        };

        ctx.add_declaration(CDeclaration::GlobalVariable(global_decl));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("init_trust = uninitialized"));
    }

    #[test]
    fn test_error_convention_detection() {
        let ctx = CDialectContext::default();
        let mut engine = CLoweringEngine::new(ctx);

        assert_eq!(
            engine.detect_error_convention("open_file"),
            "may_error = true"
        );
        assert_eq!(
            engine.detect_error_convention("init_device"),
            "may_error = true"
        );
        assert_eq!(
            engine.detect_error_convention("connect_server"),
            "may_error = true"
        );
        assert_eq!(
            engine.detect_error_convention("try_alloc"),
            "may_error = true"
        );
        assert_eq!(engine.detect_error_convention("do_math"), "");
        assert_eq!(engine.detect_error_convention("get_size"), "");
    }

    #[test]
    fn test_function_lowering_with_error_effect() {
        use chimera_c_dialect::{CDeclaration, CDeclarationLinkage, CFunctionDecl, CStorageClass};
        use chimera_c_schema::DeclId;
        let mut ctx = CDialectContext::default();

        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "open_file".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: int_type,
            attributes: vec![],
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 16,
            },
            has_body: false,
            is_inline: false,
        };

        ctx.add_declaration(CDeclaration::Function(func));

        let result = lower_to_module(ctx);
        assert!(result.is_ok());
        let module = result.unwrap();
        let output = module.operations.join("\n");
        assert!(output.contains("may_error = true"));
    }

    #[test]
    fn test_macro_expansion_stack() {
        let mut stack = MacroExpansionStack::default();
        assert!(stack.is_empty());

        stack.push(MacroExpansion {
            macro_name: "MAX".to_string(),
            invocation_location: SourceLocation {
                file: "test.c".to_string(),
                line: 10,
                col: 5,
                byte_offset: 100,
            },
            definition_location: Some(SourceLocation {
                file: "defs.h".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
            }),
        });

        assert!(!stack.is_empty());
        assert_eq!(stack.expansions().len(), 1);
        assert_eq!(stack.expansions()[0].macro_name, "MAX");
    }

    #[test]
    fn test_macro_expansion_stack_pop() {
        let mut stack = MacroExpansionStack::default();
        stack.push(MacroExpansion {
            macro_name: "INNER".to_string(),
            invocation_location: SourceLocation {
                file: "test.c".to_string(),
                line: 5,
                col: 1,
                byte_offset: 50,
            },
            definition_location: None,
        });

        let popped = stack.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().macro_name, "INNER");
        assert!(stack.is_empty());
    }

    #[test]
    fn test_macro_expansion_stack_to_comment() {
        let mut stack = MacroExpansionStack::default();
        let comment = stack.to_chimera_comment();
        assert!(comment.is_empty());

        stack.push(MacroExpansion {
            macro_name: "DEBUG".to_string(),
            invocation_location: SourceLocation {
                file: "test.c".to_string(),
                line: 20,
                col: 10,
                byte_offset: 200,
            },
            definition_location: None,
        });

        let comment = stack.to_chimera_comment();
        assert!(comment.contains("DEBUG"));
        assert!(comment.contains("test.c"));
        assert!(comment.contains("20"));
        assert!(comment.contains("10"));
    }

    #[test]
    fn test_macro_provenance_in_lowered_module() {
        let mut module = CLoweredModule::default();
        module
            .macro_provenance
            .insert("MAX_VALUE".to_string(), MacroExpansionStack::default());
        module.include_chain.insert(
            "MAX_VALUE".to_string(),
            vec!["config.h".to_string(), "types.h".to_string()],
        );

        assert!(module.macro_provenance.contains_key("MAX_VALUE"));
        assert!(module.include_chain.contains_key("MAX_VALUE"));
        let chain = module.include_chain.get("MAX_VALUE").unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], "config.h");
    }

    #[test]
    fn test_add_macro_comment() {
        let mut module = CLoweredModule::default();
        let mut stack = MacroExpansionStack::default();
        stack.push(MacroExpansion {
            macro_name: "Wrapper".to_string(),
            invocation_location: SourceLocation {
                file: "main.c".to_string(),
                line: 5,
                col: 1,
                byte_offset: 50,
            },
            definition_location: None,
        });
        module
            .macro_provenance
            .insert("my_symbol".to_string(), stack);

        let op = "  func.external @\"c_my_symbol\"() -> ()";
        let result = module.add_macro_comment("my_symbol", op);
        assert!(result.contains(op));
        assert!(result.contains("Wrapper"));
    }

    #[test]
    fn test_add_macro_comment_no_provenance() {
        let module = CLoweredModule::default();
        let op = "  func.external @\"c_my_symbol\"() -> ()";
        let result = module.add_macro_comment("nonexistent", op);
        assert_eq!(result, op);
    }

    #[test]
    fn test_emit_metadata_empty() {
        let header = ArtifactHeader::new("test", "1.0");
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let metadata = emit_metadata(&cchmeta);
        assert_eq!(metadata.version.major, 1);
        assert_eq!(metadata.version.minor, 0);
        assert_eq!(metadata.version.patch, 0);
        assert!(metadata.imports.is_empty());
        assert!(metadata.layouts.is_empty());
        assert!(metadata.trust_assumptions.is_empty());
    }

    #[test]
    fn test_emit_metadata_with_abi_facts() {
        let header = ArtifactHeader::new("test", "1.0");
        let fact = CAbiFact {
            symbol: "my_func".to_string(),
            cconv: "c".to_string(),
            params: vec![AbiParamInfo {
                position: 0,
                passing: PassingConvention::Direct,
                by_val: true,
                align: 4,
                size: 4,
            }],
            ret: Some(AbiRetInfo {
                passing: PassingConvention::Direct,
                align: 4,
                size: 4,
            }),
            varargs: false,
            proof_hash: "hash123".to_string(),
        };
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![fact],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let metadata = emit_metadata(&cchmeta);
        assert_eq!(metadata.imports.len(), 1);
        assert_eq!(metadata.imports[0].symbol, "my_func");
        assert_eq!(metadata.imports[0].language, SourceLanguage::C);
    }

    #[test]
    fn test_emit_metadata_with_layout_facts() {
        let header = ArtifactHeader::new("test", "1.0");
        let layout_fact = CLayoutFact {
            type_name: "my_struct".to_string(),
            size: 8,
            align: 4,
            fields: vec![
                LayoutFieldFact {
                    name: "field1".to_string(),
                    offset: 0,
                    size: 4,
                    align: 4,
                },
                LayoutFieldFact {
                    name: "field2".to_string(),
                    offset: 4,
                    size: 4,
                    align: 4,
                },
            ],
            bitfields: vec![],
            is_packed: false,
            proof_hash: "hash456".to_string(),
        };
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![layout_fact],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let metadata = emit_metadata(&cchmeta);
        assert_eq!(metadata.layouts.len(), 1);
        assert_eq!(metadata.layouts[0].name, "my_struct");
        assert_eq!(metadata.layouts[0].size, 8);
        assert_eq!(metadata.layouts[0].fields.len(), 2);
        assert!(!metadata.layouts[0].is_packed);
    }

    #[test]
    fn test_emit_metadata_with_trust_assumptions() {
        let header = ArtifactHeader::new("test", "1.0");
        let trust = CTrustAssumption {
            kind: CTrustKind::LayoutComputedByClang,
            description: "Layout verified by Clang".to_string(),
            external_ref: Some("clang-17".to_string()),
            verified_by: Some("tool".to_string()),
        };
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![trust],
        };
        let metadata = emit_metadata(&cchmeta);
        assert_eq!(metadata.trust_assumptions.len(), 1);
        assert_eq!(
            metadata.trust_assumptions[0].description,
            "Layout verified by Clang"
        );
        assert_eq!(
            metadata.trust_assumptions[0].external_ref,
            Some("clang-17".to_string())
        );
    }

    #[test]
    fn test_convert_to_metadata() {
        let header = ArtifactHeader::new("test", "1.0");
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let result = convert_to_metadata(&cchmeta);
        assert_eq!(result.version.major, 1);
    }

    #[test]
    fn test_chmeta_roundtrip() {
        let header = ArtifactHeader::new("test", "1.0");
        let fact = CAbiFact {
            symbol: "roundtrip_func".to_string(),
            cconv: "c".to_string(),
            params: vec![],
            ret: None,
            varargs: false,
            proof_hash: "hash789".to_string(),
        };
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![fact],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let metadata = emit_metadata(&cchmeta);
        let json = serde_json::to_string(&metadata).unwrap();
        let parsed: Metadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].symbol, "roundtrip_func");
    }

    #[test]
    fn test_emit_object_file_native() {
        let header = ArtifactHeader::new("test", "1.0");
        let cchmeta = CchMeta {
            header,
            checksum: "abc123".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let payload = vec![0x01, 0x02, 0x03, 0x04];
        let obj = emit_c_object_file("x86_64-unknown-linux-gnu", payload.clone(), &cchmeta);
        assert_eq!(obj.header.target, "x86_64-unknown-linux-gnu");
        assert_eq!(obj.payload, payload);
        assert_eq!(obj.header.payload_kind, chimera_object::PayloadKind::Native);
    }

    #[test]
    fn test_emit_object_file_textual_ir() {
        let header = ArtifactHeader::new("test", "1.0");
        let cchmeta = CchMeta {
            header,
            checksum: "def456".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let ir_text = "func @main() -> ()";
        let obj = emit_c_ir_object_file("aarch64-unknown-linux-gnu", ir_text, &cchmeta);
        assert_eq!(obj.header.target, "aarch64-unknown-linux-gnu");
        assert_eq!(obj.payload, ir_text.as_bytes());
        assert_eq!(
            obj.header.payload_kind,
            chimera_object::PayloadKind::TextualIR
        );
    }

    #[test]
    fn test_emit_object_file_roundtrip() {
        let header = ArtifactHeader::new("test", "1.0");
        let fact = CAbiFact {
            symbol: "my_func".to_string(),
            cconv: "c".to_string(),
            params: vec![],
            ret: None,
            varargs: false,
            proof_hash: "hash789".to_string(),
        };
        let cchmeta = CchMeta {
            header,
            checksum: "ghi789".to_string(),
            declaration_provenance: vec![],
            c_abi_facts: vec![fact],
            layout_facts: vec![],
            macro_dependencies: vec![],
            include_dependencies: vec![],
            trust_assumptions: vec![],
        };
        let obj = emit_c_object_file("x86_64-unknown-linux-gnu", vec![], &cchmeta);
        // Verify the object has correct target and metadata
        assert_eq!(obj.header.target, "x86_64-unknown-linux-gnu");
        // Metadata should contain our import
        assert!(!obj.metadata.imports.is_empty());
        assert_eq!(obj.metadata.imports[0].symbol, "my_func");
        // Verify we can serialize it
        let bytes = obj.to_bytes();
        assert!(!bytes.is_empty());
        // Verify binary format starts with magic
        assert_eq!(&bytes[0..4], b"CHOB");
    }
}
