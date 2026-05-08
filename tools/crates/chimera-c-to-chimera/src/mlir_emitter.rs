//! Chimera C to MLIR emission module.
//!
//! Emits real Chimera MLIR ops/types from C dialect representations
//! using compiler-core MLIR definitions.
//!
//! Task 101: Emit Chimera MLIR from C

use chimera_c_dialect::{CDeclaration, CDialectContext, CType};
use chimera_c_schema::{CchMeta, TypeRef};
use chimera_meta::{Metadata, Signature, SourceLanguage};
use std::collections::HashMap;

/// Result type for MLIR emission operations
pub type Result<T> = std::result::Result<T, MlirEmissionError>;

/// MLIR emission errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum MlirEmissionError {
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("emission failed: {0}")]
    EmissionFailed(String),
    #[error("missing context: {0}")]
    MissingContext(String),
}

/// Target information for MLIR emission
#[derive(Debug, Clone)]
pub struct MlirTargetInfo {
    pub triple: String,
    pub pointer_width: u32,
    pub data_layout: String,
}

/// Default target info for x86_64 Linux
impl Default for MlirTargetInfo {
    fn default() -> Self {
        Self {
            triple: "x86_64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            data_layout: "E-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
                .to_string(),
        }
    }
}

impl MlirTargetInfo {
    /// Create target info from triple string
    pub fn from_triple(triple: &str) -> Self {
        let pointer_width = if triple.contains("64") { 64 } else { 32 };
        Self {
            triple: triple.to_string(),
            pointer_width,
            data_layout: Self::default_data_layout(triple),
        }
    }

    /// Generate default data layout for target
    fn default_data_layout(triple: &str) -> String {
        if triple.contains("x86_64") {
            "E-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128".to_string()
        } else if triple.contains("aarch64") {
            "E-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".to_string()
        } else if triple.contains("wasm") {
            "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128".to_string()
        } else {
            "E-m:e-p:32:32-p64:64-i64:64-n8:16:32:64-S128".to_string()
        }
    }

    /// Get pointer size in bytes
    pub fn pointer_size_bytes(&self) -> u32 {
        self.pointer_width / 8
    }
}

/// C to MLIR emission context
#[derive(Debug, Clone)]
pub struct CMlirEmitter {
    ctx: CDialectContext,
    target: MlirTargetInfo,
    symbol_map: HashMap<String, String>,
    type_map: HashMap<TypeRef, String>,
    next_id: u64,
    operations: Vec<String>,
    emitted_functions: Vec<String>,
}

impl CMlirEmitter {
    /// Create a new MLIR emitter
    pub fn new(ctx: CDialectContext) -> Self {
        Self {
            ctx,
            target: MlirTargetInfo::default(),
            symbol_map: HashMap::new(),
            type_map: HashMap::new(),
            next_id: 0,
            operations: Vec::new(),
            emitted_functions: Vec::new(),
        }
    }

    /// Create with custom target
    pub fn with_target(mut self, target: MlirTargetInfo) -> Self {
        self.target = target;
        self
    }

    /// Generate unique ID
    fn fresh_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("ch_{id}")
    }

    /// Mangle C name to MLIR-compatible name
    fn mangle_name(&mut self, name: &str) -> String {
        let sanitized = name.replace(['.', '$', '#', ':'], "_");
        if let Some(existing) = self.symbol_map.get(name) {
            return existing.clone();
        }
        let ir_name = format!("c_{sanitized}");
        self.symbol_map.insert(name.to_string(), ir_name.clone());
        ir_name
    }

    /// Emit the complete MLIR module
    pub fn emit_module(&mut self) -> Result<String> {
        // Clear any previous state
        self.operations.clear();
        self.emitted_functions.clear();

        // Emit module header with target and source language
        self.emit_module_header()?;

        // Lower all declarations
        let declarations: Vec<_> = self.ctx.declarations.values().cloned().collect();
        for decl in declarations {
            self.emit_declaration(&decl)?;
        }

        // Close module
        self.emit_module_footer()?;

        Ok(self.operations.join("\n"))
    }

    /// Emit module header with target configuration
    fn emit_module_header(&mut self) -> Result<()> {
        self.operations.push(format!(
            "chimera.module @\"c_module\" {{\n  target = \"{}\"\n  source_lang = \"c\"\n  data_layout = \"{}\"",
            self.target.triple,
            self.target.data_layout
        ));
        Ok(())
    }

    /// Emit module footer
    fn emit_module_footer(&mut self) -> Result<()> {
        self.operations.push("}".to_string());
        Ok(())
    }

    /// Emit a single declaration
    fn emit_declaration(&mut self, decl: &CDeclaration) -> Result<()> {
        match decl {
            CDeclaration::Function(func) => self.emit_function(func),
            CDeclaration::Struct(s) => self.emit_struct_type(s),
            CDeclaration::Union(u) => self.emit_union_type(u),
            CDeclaration::Enum(e) => self.emit_enum_type(e),
            CDeclaration::Typedef(t) => self.emit_typedef(t),
            CDeclaration::GlobalVariable(var) => self.emit_global(var),
            _ => Ok(()), // Skip enum constants and macros for now
        }
    }

    /// Emit a function as chimera.func operation
    fn emit_function(&mut self, func: &chimera_c_dialect::CFunctionDecl) -> Result<()> {
        let ir_name = self.mangle_name(&func.name);

        // Skip if already emitted (avoid duplicates)
        if self.emitted_functions.contains(&ir_name) {
            return Ok(());
        }
        self.emitted_functions.push(ir_name.clone());

        // Get function type
        self.emit_function_type(func)?;

        // Determine linkage
        let _linkage = match func.linkage {
            chimera_c_dialect::CDeclarationLinkage::External => "external",
            chimera_c_dialect::CDeclarationLinkage::Internal => "private",
            chimera_c_dialect::CDeclarationLinkage::Weak => "weak",
            _ => "external",
        };

        // Detect effects
        let effects = self.detect_effects(func);

        // Build function signature
        let mut param_types = Vec::new();
        let mut param_names = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            self.lower_type(&param.typ)?;
            let ptype = self
                .type_map
                .get(&param.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            param_types.push(ptype);
            let pname = if param.name.is_empty() {
                format!("arg_{}", i)
            } else {
                param.name.clone()
            };
            param_names.push(pname);
        }

        let ret_type = self
            .type_map
            .get(&func.return_type)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        // Build operation with inline format matching Ops.td
        let params_formatted: Vec<String> = param_names
            .iter()
            .zip(param_types.iter())
            .map(|(n, t)| format!("{}: {}", n, t))
            .collect();

        let mut func_op = format!(
            "  chimera.func @{ir_name}({params}) -> {ret} {{",
            ir_name = ir_name,
            params = params_formatted.join(", "),
            ret = ret_type
        );

        // Add effects if detected
        if !effects.is_empty() {
            func_op.push_str(&format!("  [{effects}]"));
        }

        func_op.push_str(" }");

        self.operations.push(func_op);
        Ok(())
    }

    /// Emit function type string
    fn emit_function_type(&mut self, func: &chimera_c_dialect::CFunctionDecl) -> Result<String> {
        self.lower_type(&func.return_type)?;
        let ret_str = self
            .type_map
            .get(&func.return_type)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        let mut param_types = Vec::new();
        for param in &func.params {
            self.lower_type(&param.typ)?;
            let ptype_str = self
                .type_map
                .get(&param.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            param_types.push(ptype_str);
        }

        Ok(format!("({}) -> {}", param_types.join(", "), ret_str))
    }

    /// Detect effects from function name/attributes
    fn detect_effects(&self, func: &chimera_c_dialect::CFunctionDecl) -> String {
        let mut effects = Vec::new();

        // Check name patterns for common effect patterns
        if func.name.ends_with("_err")
            || func.name.ends_with("_result")
            || func.name.starts_with("try_")
            || func.name.contains("open")
            || func.name.contains("init")
            || func.name.contains("connect")
        {
            effects.push("may_error");
        }

        if func.name.contains("alloc") || func.name.contains("malloc") || func.name.contains("free")
        {
            effects.push("may_alloc");
            effects.push("may_dealloc");
        }

        if effects.is_empty() {
            String::new()
        } else {
            effects.join(", ")
        }
    }

    /// Compute struct size and alignment from fields
    fn compute_struct_layout(s: &chimera_c_dialect::CStructDecl) -> (u64, u32) {
        let mut max_align: u32 = 1;
        let mut total_size: u64 = 0;
        for field in &s.fields {
            max_align = max_align.max(field.align);
            total_size = field.offset + field.size;
        }
        (total_size, max_align)
    }

    /// Emit a struct type as chimera.type operation
    fn emit_struct_type(&mut self, s: &chimera_c_dialect::CStructDecl) -> Result<()> {
        let name = s.name.as_deref().unwrap_or("unnamed_struct");
        let ir_name = self.mangle_name(name);

        if s.is_incomplete {
            self.operations
                .push(format!("  chimera.type @\"{ir_name}\" {{ opaque }}"));
            return Ok(());
        }

        // Build field list
        let mut field_strs = Vec::new();
        for field in &s.fields {
            self.lower_type(&field.typ)?;
            let ftype_str = self
                .type_map
                .get(&field.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            let field_str = format!("{name}: {typ}", name = field.name, typ = ftype_str);
            field_strs.push(field_str);
        }

        let fields_str = field_strs.join(", ");
        let packed_attr = if s.is_packed { " packed" } else { "" };

        // Compute layout from fields
        let (size, align) = Self::compute_struct_layout(s);

        self.operations.push(format!(
            "  chimera.type @\"{ir_name}\"{packed} {{ fields = [{fields}], size = {size}, align = {align} }}",
            ir_name = ir_name,
            packed = packed_attr,
            fields = fields_str,
            size = size,
            align = align
        ));
        Ok(())
    }

    /// Emit a union type
    fn emit_union_type(&mut self, u: &chimera_c_dialect::CUnionDecl) -> Result<()> {
        let name = u.name.as_deref().unwrap_or("unnamed_union");
        let ir_name = self.mangle_name(name);

        let mut variant_strs = Vec::new();
        for variant in &u.variants {
            self.lower_type(&variant.typ)?;
            let vtype_str = self
                .type_map
                .get(&variant.typ)
                .cloned()
                .unwrap_or_else(|| "()".to_string());
            variant_strs.push(format!(
                "{name}: {typ}",
                name = variant.name,
                typ = vtype_str
            ));
        }

        let variants_str = variant_strs.join(", ");

        self.operations.push(format!(
            "  chimera.type @\"{ir_name}\" {{ union, variants = [{variants}], size = {size}, align = {align} }}",
            ir_name = ir_name,
            variants = variants_str,
            size = u.size,
            align = u.align
        ));
        Ok(())
    }

    /// Emit an enum type
    fn emit_enum_type(&mut self, e: &chimera_c_dialect::CEnumDecl) -> Result<()> {
        let name = e.name.as_deref().unwrap_or("unnamed_enum");
        let ir_name = self.mangle_name(name);

        self.lower_type(&e.underlying_type)?;
        let underlying_str = self
            .type_map
            .get(&e.underlying_type)
            .cloned()
            .unwrap_or_else(|| "i32".to_string());

        let mut const_strs = Vec::new();
        for c in &e.constants {
            let value_str = c.value.map_or("0".to_string(), |v| v.to_string());
            const_strs.push(format!(
                "{name} = {value}",
                name = c.name,
                value = value_str
            ));
        }
        let consts_str = const_strs.join(", ");

        self.operations.push(format!(
            "  chimera.type @\"{ir_name}\" {{ enum, underlying = {underlying}, constants = [{consts}] }}",
            ir_name = ir_name,
            underlying = underlying_str,
            consts = consts_str
        ));
        Ok(())
    }

    /// Emit a typedef
    fn emit_typedef(&mut self, t: &chimera_c_dialect::CTypedefDecl) -> Result<()> {
        let ir_name = self.mangle_name(&t.name);
        self.lower_type(&t.underlying_type)?;
        let source_str = self
            .type_map
            .get(&t.underlying_type)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        self.operations.push(format!(
            "  chimera.typedef @\"{ir_name}\" = {source}",
            ir_name = ir_name,
            source = source_str
        ));
        Ok(())
    }

    /// Emit a global variable
    fn emit_global(&mut self, var: &chimera_c_dialect::CGlobalVarDecl) -> Result<()> {
        let ir_name = self.mangle_name(&var.name);
        self.lower_type(&var.typ)?;
        let vtype_str = self
            .type_map
            .get(&var.typ)
            .cloned()
            .unwrap_or_else(|| "()".to_string());

        let linkage = match var.linkage {
            chimera_c_dialect::CDeclarationLinkage::External => "external",
            chimera_c_dialect::CDeclarationLinkage::Internal => "private",
            _ => "external",
        };

        let thread_local_str = if var.is_thread_local {
            " thread_local"
        } else {
            ""
        };

        self.operations.push(format!(
            "  chimera.global @\"{ir_name}\"{thread} {linkage} : {typ}",
            ir_name = ir_name,
            thread = thread_local_str,
            linkage = linkage,
            typ = vtype_str
        ));
        Ok(())
    }

    /// Lower a type reference to MLIR type string
    fn lower_type(&mut self, type_ref: &TypeRef) -> Result<()> {
        if self.type_map.contains_key(type_ref) {
            return Ok(());
        }

        // Get type from context
        let typ = self
            .ctx
            .get_type(type_ref)
            .ok_or_else(|| MlirEmissionError::MissingContext(format!("type ref {:?}", type_ref)))?;

        let type_str = match typ {
            CType::Void => "()".to_string(),
            CType::Bool => "i1".to_string(),
            CType::Char { is_signed } => if *is_signed { "si8" } else { "ui8" }.to_string(),
            CType::Short { is_signed } => if *is_signed { "si16" } else { "ui16" }.to_string(),
            CType::Int { is_signed } => if *is_signed { "si32" } else { "ui32" }.to_string(),
            CType::Long { is_signed } => if *is_signed { "si64" } else { "ui64" }.to_string(),
            CType::LongLong { is_signed } => if *is_signed { "si64" } else { "ui64" }.to_string(),
            CType::Float => "f32".to_string(),
            CType::Double => "f64".to_string(),
            CType::LongDouble => "f80".to_string(),
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
                    _ => "raw",
                };
                format!("!ch.ptr<{base_str}, {null_str}>")
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
                    "!ch.func<({params}) -> {ret_str}, cconv = {cconv}>",
                    params = param_strs.join(", "),
                    ret_str = ret_str,
                    cconv = cconv
                )
            }
            CType::Struct(tr) | CType::Union(tr) | CType::Enum(tr) | CType::Typedef(tr) => self
                .type_map
                .get(tr)
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
}

/// Emit MLIR from C dialect context
pub fn emit_mlir(ctx: CDialectContext) -> Result<String> {
    let mut emitter = CMlirEmitter::new(ctx);
    emitter.emit_module()
}

/// Emit MLIR with custom target
pub fn emit_mlir_with_target(ctx: CDialectContext, target: MlirTargetInfo) -> Result<String> {
    let mut emitter = CMlirEmitter::new(ctx).with_target(target);
    emitter.emit_module()
}

/// Convert CchMeta to common Metadata with MLIR emission info
pub fn mlir_to_metadata(cchmeta: &CchMeta) -> Metadata {
    let mut metadata = Metadata::default();
    metadata.version = chimera_meta::Version::new(1, 0, 0);

    // Convert C ABI facts to imports
    for fact in &cchmeta.c_abi_facts {
        let import = chimera_meta::ImportMetadata {
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

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_c_dialect::{
        CDeclaration, CDeclarationLinkage, CDialectContext, CFieldDecl, CFunctionDecl,
        CStorageClass, CStructDecl,
    };
    use chimera_c_schema::{DeclId, TypeRef};

    #[test]
    fn test_emitter_new() {
        let ctx = CDialectContext::default();
        let emitter = CMlirEmitter::new(ctx);
        assert_eq!(emitter.next_id, 0);
    }

    #[test]
    fn test_emitter_with_target() {
        let ctx = CDialectContext::default();
        let target = MlirTargetInfo {
            triple: "aarch64-unknown-linux-gnu".to_string(),
            pointer_width: 64,
            data_layout: "E-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128".to_string(),
        };
        let emitter = CMlirEmitter::new(ctx).with_target(target.clone());
        assert_eq!(emitter.target.triple, "aarch64-unknown-linux-gnu");
    }

    #[test]
    fn test_mlir_target_info_default() {
        let target = MlirTargetInfo::default();
        assert_eq!(target.pointer_width, 64);
        assert!(target.data_layout.contains("E"));
    }

    #[test]
    fn test_mlir_target_info_from_triple_x86_64() {
        let target = MlirTargetInfo::from_triple("x86_64-unknown-linux-gnu");
        assert_eq!(target.pointer_width, 64);
        assert!(target.data_layout.contains("p270"));
    }

    #[test]
    fn test_mlir_target_info_from_triple_aarch64() {
        let target = MlirTargetInfo::from_triple("aarch64-unknown-linux-gnu");
        assert_eq!(target.pointer_width, 64);
        assert!(target.data_layout.contains("i64"));
    }

    #[test]
    fn test_mlir_target_info_from_triple_wasm() {
        let target = MlirTargetInfo::from_triple("wasm32-unknown-unknown");
        assert_eq!(target.pointer_width, 32);
        assert!(target.data_layout.contains("e-m:e"));
    }

    #[test]
    fn test_pointer_size_bytes() {
        let target = MlirTargetInfo::default();
        assert_eq!(target.pointer_size_bytes(), 8);
    }

    #[test]
    fn test_fresh_id() {
        let ctx = CDialectContext::default();
        let mut emitter = CMlirEmitter::new(ctx);
        assert_eq!(emitter.fresh_id(), "ch_0");
        assert_eq!(emitter.fresh_id(), "ch_1");
        assert_eq!(emitter.fresh_id(), "ch_2");
    }

    #[test]
    fn test_mangle_name() {
        let ctx = CDialectContext::default();
        let mut emitter = CMlirEmitter::new(ctx);
        assert_eq!(emitter.mangle_name("foo"), "c_foo");
        assert_eq!(emitter.mangle_name("bar"), "c_bar");
    }

    #[test]
    fn test_mangle_name_deduplication() {
        let ctx = CDialectContext::default();
        let mut emitter = CMlirEmitter::new(ctx);
        assert_eq!(emitter.mangle_name("foo"), "c_foo");
        assert_eq!(emitter.mangle_name("foo"), "c_foo");
    }

    #[test]
    fn test_mangle_name_special_chars() {
        let ctx = CDialectContext::default();
        let mut emitter = CMlirEmitter::new(ctx);
        assert_eq!(emitter.mangle_name("foo.bar"), "c_foo_bar");
        assert_eq!(emitter.mangle_name("ns::func"), "c_ns__func");
    }

    #[test]
    fn test_emit_mlir_empty_module() {
        let ctx = CDialectContext::default();
        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("chimera.module"));
        assert!(mlir.contains("source_lang = \"c\""));
    }

    #[test]
    fn test_emit_mlir_with_target() {
        let ctx = CDialectContext::default();
        let target = MlirTargetInfo::from_triple("aarch64-unknown-linux-gnu");
        let result = emit_mlir_with_target(ctx, target);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("aarch64-unknown-linux-gnu"));
    }

    #[test]
    fn test_emit_function() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "my_func".to_string(),
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
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        };
        ctx.add_declaration(CDeclaration::Function(func));

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("chimera.func"));
        assert!(mlir.contains("c_my_func"));
    }

    #[test]
    fn test_emit_function_with_params() {
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

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "process".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![
                chimera_c_dialect::CParamDecl {
                    name: "input".to_string(),
                    typ: ptr_type,
                    has_default: false,
                },
                chimera_c_dialect::CParamDecl {
                    name: "count".to_string(),
                    typ: int_type,
                    has_default: false,
                },
            ],
            return_type: int_type,
            attributes: vec![],
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 32,
            },
            has_body: false,
            is_inline: false,
        };
        ctx.add_declaration(CDeclaration::Function(func));

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("input:"));
        assert!(mlir.contains("count:"));
    }

    #[test]
    fn test_emit_struct_type() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

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

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("chimera.type"));
        assert!(mlir.contains("c_Point"));
        assert!(mlir.contains("fields = ["));
    }

    #[test]
    fn test_emit_struct_type_packed() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let struct_decl = CStructDecl {
            id: DeclId(0),
            name: Some("Packed".to_string()),
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

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("packed"));
    }

    #[test]
    fn test_emit_global() {
        use chimera_c_dialect::CGlobalVarDecl;
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let global_decl = CGlobalVarDecl {
            id: DeclId(0),
            name: "global_counter".to_string(),
            linkage: CDeclarationLinkage::Internal,
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
            is_thread_local: false,
            is_static: true,
        };

        ctx.add_declaration(CDeclaration::GlobalVariable(global_decl));

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("chimera.global"));
        assert!(mlir.contains("c_global_counter"));
    }

    #[test]
    fn test_emit_global_thread_local() {
        use chimera_c_dialect::CGlobalVarDecl;
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        let global_decl = CGlobalVarDecl {
            id: DeclId(0),
            name: "tls_counter".to_string(),
            linkage: CDeclarationLinkage::Internal,
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

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("thread_local"));
    }

    #[test]
    fn test_detect_effects_error_func() {
        let ctx = CDialectContext::default();
        let emitter = CMlirEmitter::new(ctx);

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "open_file".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(1),
            attributes: vec![],
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        };

        let effects = emitter.detect_effects(&func);
        assert!(effects.contains("may_error"));
    }

    #[test]
    fn test_detect_effects_alloc_func() {
        let ctx = CDialectContext::default();
        let emitter = CMlirEmitter::new(ctx);

        let func = CFunctionDecl {
            id: DeclId(0),
            name: "my_malloc".to_string(),
            linkage: CDeclarationLinkage::External,
            storage_class: CStorageClass::None,
            calling_convention: "cdecl".to_string(),
            params: vec![],
            return_type: TypeRef(1),
            attributes: vec![],
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        };

        let effects = emitter.detect_effects(&func);
        assert!(effects.contains("may_alloc"));
        assert!(effects.contains("may_dealloc"));
    }

    #[test]
    fn test_emit_pointer_type() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        let ptr_type = TypeRef(2);
        ctx.types.insert(int_type, CType::Int { is_signed: true });
        ctx.types.insert(
            ptr_type,
            CType::Pointer {
                pointee: int_type,
                constness: 0,
                nullability: chimera_c_abi::PointerNullability::NonNull,
            },
        );

        let mut emitter = CMlirEmitter::new(ctx);
        let result = emitter.lower_type(&ptr_type);
        assert!(result.is_ok());
        let type_str = emitter.type_map.get(&ptr_type).unwrap();
        assert!(type_str.contains("!ch.ptr"));
        assert!(type_str.contains("nonnull"));
    }

    #[test]
    fn test_emit_array_type() {
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

        let mut emitter = CMlirEmitter::new(ctx);
        let result = emitter.lower_type(&arr_type);
        assert!(result.is_ok());
        let type_str = emitter.type_map.get(&arr_type).unwrap();
        assert!(type_str.contains("!ch.array"));
        assert!(type_str.contains("10"));
    }

    #[test]
    fn test_emit_function_pointer_type() {
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

        let mut emitter = CMlirEmitter::new(ctx);
        let result = emitter.lower_type(&fp_type);
        assert!(result.is_ok());
        let type_str = emitter.type_map.get(&fp_type).unwrap();
        assert!(type_str.contains("!ch.func"));
        assert!(type_str.contains("cconv"));
    }

    #[test]
    fn test_emit_incomplete_struct() {
        let mut ctx = CDialectContext::default();

        let struct_decl = CStructDecl {
            id: DeclId(0),
            name: Some("IncompleteStruct".to_string()),
            fields: vec![],
            is_packed: false,
            pack_align: None,
            is_incomplete: true,
            source_span: chimera_c_schema::SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 0,
            },
        };

        ctx.add_declaration(CDeclaration::Struct(struct_decl));

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("opaque"));
    }

    #[test]
    fn test_mlir_to_metadata() {
        use chimera_c_schema::{AbiParamInfo, ArtifactHeader, CAbiFact, PassingConvention};
        let header = ArtifactHeader::new("test", "1.0");
        let fact = CAbiFact {
            symbol: "test_func".to_string(),
            cconv: "c".to_string(),
            params: vec![AbiParamInfo {
                position: 0,
                passing: PassingConvention::Direct,
                by_val: true,
                align: 4,
                size: 4,
            }],
            ret: None,
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

        let metadata = mlir_to_metadata(&cchmeta);
        assert_eq!(metadata.imports.len(), 1);
        assert_eq!(metadata.imports[0].symbol, "test_func");
        assert_eq!(metadata.imports[0].language, SourceLanguage::C);
    }

    #[test]
    fn test_emit_module_with_multiple_declarations() {
        let mut ctx = CDialectContext::default();
        let int_type = TypeRef(1);
        ctx.types.insert(int_type, CType::Int { is_signed: true });

        // Add a function
        let func = CFunctionDecl {
            id: DeclId(0),
            name: "add".to_string(),
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
                byte_length: 10,
            },
            has_body: false,
            is_inline: false,
        };
        ctx.add_declaration(CDeclaration::Function(func));

        // Add a struct
        let struct_decl = CStructDecl {
            id: DeclId(1),
            name: Some("Vec2".to_string()),
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
                line: 5,
                col: 1,
                byte_offset: 40,
                byte_length: 32,
            },
        };
        ctx.add_declaration(CDeclaration::Struct(struct_decl));

        let result = emit_mlir(ctx);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("chimera.func"));
        assert!(mlir.contains("chimera.type"));
        assert!(mlir.contains("c_add"));
        assert!(mlir.contains("c_Vec2"));
    }

    #[test]
    fn test_emit_data_layout_in_module() {
        let ctx = CDialectContext::default();
        let target = MlirTargetInfo::from_triple("x86_64-unknown-linux-gnu");
        let result = emit_mlir_with_target(ctx, target);
        assert!(result.is_ok());
        let mlir = result.unwrap();
        assert!(mlir.contains("data_layout"));
    }
}
