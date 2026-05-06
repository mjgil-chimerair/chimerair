//! Lower Zig dialect to ChimeraIR.
//!
//! This crate handles lowering from Zig dialect types to Chimera IR
//! textual format (.chimera) and/or Chimera MLIR.
//!
//! Task 20: Implement minimal Zig ChimeraIR emitter

use chimera_meta::{
    FieldLayout, Function, LayoutMetadata, Metadata, Module, PanicPolicy as MetaPanicPolicy,
    PanicPolicyMetadata, Signature, SourceLanguage, Version,
};
use chimera_object::{ObjectFile, PayloadKind, TrustLevel, TrustMetadata};
use serde::{Deserialize, Serialize};
use zigmera_dialect::{DialectFunction, DialectModule, Effect, PanicPolicy, ZigType, ZigTypeKind};

/// A ChimeraIR module from Zig lowering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraModule {
    pub name: String,
    pub items: Vec<ZigChimeraItem>,
    pub types: Vec<ZigChimeraTypeDef>,
    pub imports: Vec<ZigChimeraImport>,
}

/// Import from external Zig dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraImport {
    pub symbol: String,
    pub abi: String,
    pub signature: String,
}

/// ABI attributes for Zig exports (Task 21)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigAbiAttrs {
    pub source_lang: String,
    pub symbol: String,
    pub calling_convention: String,
    pub panic_policy: String,
    pub effect_set: Vec<String>,
    pub trust_level: String,
}

/// A lowered Zig item in ChimeraIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraItem {
    pub name: String,
    pub kind: ZigChimeraItemKind,
    pub abi: String,
    pub location: Option<ZigSourceLocation>,
    /// ABI metadata for exports (Task 21)
    pub abi_attrs: Option<ZigAbiAttrs>,
    /// Panic policy for functions (Task 24)
    pub panic_policy: Option<ZigChimeraPanicPolicy>,
    /// Effects for functions
    pub effects: Vec<String>,
    /// Inline eligibility for cross-language optimization (Task 34)
    #[serde(default)]
    pub inline_eligibility: InlineEligibility,
}

/// Inline eligibility for cross-language optimization (Task 34)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InlineEligibility {
    /// Function is eligible for inlining across language boundaries
    Eligible,
    /// Function should NOT be inlined (has side effects or external dependencies)
    Ineligible,
    /// Inline eligibility is unknown or requires more analysis
    Unknown,
}

impl Default for InlineEligibility {
    fn default() -> Self {
        InlineEligibility::Unknown
    }
}

impl InlineEligibility {
    /// Determine if this item is eligible for cross-language inlining.
    pub fn can_inline(&self) -> bool {
        matches!(self, InlineEligibility::Eligible)
    }
}

/// Panic policy for Zig functions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ZigChimeraPanicPolicy {
    /// Function never panics (e.g., nosuspend, extern "C")
    Never,
    /// Function allows unwinding
    AllowUnwind,
    /// Only bounds checks may fail
    BoundsCheckOnly,
}

impl From<&str> for ZigChimeraPanicPolicy {
    fn from(s: &str) -> Self {
        match s {
            "allow_unwind" => ZigChimeraPanicPolicy::AllowUnwind,
            "bounds_check_only" => ZigChimeraPanicPolicy::BoundsCheckOnly,
            _ => ZigChimeraPanicPolicy::Never,
        }
    }
}

impl From<ZigChimeraPanicPolicy> for String {
    fn from(p: ZigChimeraPanicPolicy) -> Self {
        match p {
            ZigChimeraPanicPolicy::Never => "Never".to_string(),
            ZigChimeraPanicPolicy::AllowUnwind => "AllowUnwind".to_string(),
            ZigChimeraPanicPolicy::BoundsCheckOnly => "BoundsCheckOnly".to_string(),
        }
    }
}

impl From<&PanicPolicy> for ZigChimeraPanicPolicy {
    fn from(p: &PanicPolicy) -> Self {
        match p {
            PanicPolicy::AllowUnwind => ZigChimeraPanicPolicy::AllowUnwind,
            PanicPolicy::NoUnwind => ZigChimeraPanicPolicy::Never,
            PanicPolicy::BoundsCheckOnly => ZigChimeraPanicPolicy::BoundsCheckOnly,
        }
    }
}

/// Source location for Zig items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigSourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Kind of ChimeraIR item from Zig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZigChimeraItemKind {
    Function {
        params: Vec<ZigChimeraType>,
        return_type: Box<ZigChimeraType>,
    },
    Global {
        ty: ZigChimeraType,
        is_exported: bool,
        is_thread_local: bool,
        callconv: String,
    },
    TypeDef {
        repr: Option<String>,
    },
}

/// A lowered type definition from Zig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraTypeDef {
    pub name: String,
    pub kind: ZigChimeraTypeDefKind,
    /// Layout facts for this type (Task 23)
    pub layout: Option<ZigChimeraLayoutFact>,
}

/// Layout fact for Zig type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraLayoutFact {
    pub size_bytes: u64,
    pub alignment_bytes: u64,
    pub abi_kind: String,
    pub field_layouts: Vec<ZigChimeraFieldLayout>,
}

/// Field layout for Zig types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigChimeraFieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u64,
}

/// Kind of type definition from Zig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZigChimeraTypeDefKind {
    Struct {
        fields: Vec<(String, ZigChimeraType)>,
    },
    Enum {
        variants: Vec<(String, Option<i64>, Vec<(String, ZigChimeraType)>)>,
    },
    Union {
        fields: Vec<(String, ZigChimeraType)>,
    },
    ErrorSet {
        errors: Vec<String>,
    },
}

/// ChimeraIR type from Zig lowering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZigChimeraType {
    Never,
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    F32,
    F64,
    Pointer(Box<ZigChimeraType>),
    Slice(Box<ZigChimeraType>),
    Array(Box<ZigChimeraType>, u64),
    Optional(Box<ZigChimeraType>),
    Error(Box<ZigChimeraType>),
    Struct {
        name: String,
        fields: Vec<(String, ZigChimeraType)>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
    Opaque,
}

impl Default for ZigChimeraType {
    fn default() -> Self {
        ZigChimeraType::Never
    }
}

/// Lower a Zig dialect module to ChimeraIR
pub fn lower_zig_module(module: &DialectModule) -> ZigChimeraModule {
    let mut items = Vec::new();
    let mut types = Vec::new();
    let mut imports = Vec::new();

    // Lower functions (exports)
    for func in &module.functions {
        let effects = extract_zig_effects(func);
        let panic_policy = infer_zig_panic_policy(func);

        let params: Vec<ZigChimeraType> = func
            .params
            .iter()
            .map(|type_id| lower_zig_type_id(*type_id, module))
            .collect();

        let return_type = func
            .return_type
            .map(|type_id| lower_zig_type_id(type_id, module))
            .unwrap_or(ZigChimeraType::Never);

        items.push(ZigChimeraItem {
            name: func.name.clone(),
            kind: ZigChimeraItemKind::Function {
                params,
                return_type: Box::new(return_type),
            },
            abi: func.callconv.clone(),
            location: None,
            abi_attrs: build_zig_abi_attrs(func, &effects),
            panic_policy: Some(panic_policy),
            effects: effects.clone(),
            inline_eligibility: compute_inline_eligibility(func, &effects, &panic_policy),
        });
    }

    // Lower types
    for ty in &module.types {
        if let Some(type_def) = lower_zig_type(ty, module) {
            types.push(type_def);
        }
    }

    // Lower extern imports (Task 22)
    for ext_fn in &module.extern_fns {
        let params: Vec<ZigChimeraType> = ext_fn
            .params
            .iter()
            .map(|type_id| lower_zig_type_id(*type_id, module))
            .collect();

        let return_type = ext_fn
            .return_type
            .map(|type_id| lower_zig_type_id(type_id, module))
            .unwrap_or(ZigChimeraType::Never);

        imports.push(ZigChimeraImport {
            symbol: ext_fn.name.clone(),
            abi: ext_fn.abi.clone(),
            signature: format!(
                "({}) -> {}",
                params
                    .iter()
                    .map(zig_chimera_type_to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                zig_chimera_type_to_string(&return_type)
            ),
        });
    }

    ZigChimeraModule {
        name: module.name.clone(),
        items,
        types,
        imports,
    }
}

/// Extract effects from a Zig function
fn extract_zig_effects(func: &DialectFunction) -> Vec<String> {
    let mut effects = Vec::new();

    // Check for async/await by looking at blocks
    for block in &func.blocks {
        for inst in &block.instructions {
            match inst.op {
                zigmera_dialect::ZigOp::Await => {
                    effects.push("async_await".to_string());
                }
                zigmera_dialect::ZigOp::SuspendFrame => {
                    effects.push("suspend".to_string());
                }
                zigmera_dialect::ZigOp::Resume => {
                    effects.push("resume".to_string());
                }
                zigmera_dialect::ZigOp::Unreachable => {
                    effects.push("panic".to_string());
                }
                zigmera_dialect::ZigOp::Invoke => {
                    effects.push("panic".to_string());
                }
                _ => {}
            }
        }
    }

    if effects.is_empty() {
        effects.push("none".to_string());
    }

    effects
}

/// Infer panic policy from Zig function context
fn infer_zig_panic_policy(func: &DialectFunction) -> ZigChimeraPanicPolicy {
    // export fn and extern fn default to NoUnwind
    if func.is_exported {
        return ZigChimeraPanicPolicy::Never;
    }

    match func.callconv.as_str() {
        "C" => ZigChimeraPanicPolicy::Never,
        "Opaque" => ZigChimeraPanicPolicy::Never,
        _ => ZigChimeraPanicPolicy::Never,
    }
}

/// Build ABI attributes for a Zig export (Task 21)
fn build_zig_abi_attrs(func: &DialectFunction, effects: &[String]) -> Option<ZigAbiAttrs> {
    if !func.is_exported {
        return None;
    }

    let trust_level = if func.callconv == "C" {
        "Trusted"
    } else {
        "TCB"
    }
    .to_string();

    Some(ZigAbiAttrs {
        source_lang: "zig".to_string(),
        symbol: func.name.clone(),
        calling_convention: func.callconv.clone(),
        panic_policy: String::from(infer_zig_panic_policy(func)),
        effect_set: effects.to_vec(),
        trust_level,
    })
}

/// Compute inline eligibility for cross-language optimization (Task 34)
fn compute_inline_eligibility(
    func: &DialectFunction,
    effects: &[String],
    panic_policy: &ZigChimeraPanicPolicy,
) -> InlineEligibility {
    // Functions with async/await, suspend, or resume effects cannot be inlined
    // across language boundaries safely
    if effects
        .iter()
        .any(|e| matches!(e.as_str(), "async_await" | "suspend" | "resume"))
    {
        return InlineEligibility::Ineligible;
    }

    // Functions that can panic (beyond simple bounds checks) are not safe to inline
    // across language boundaries unless the caller can handle the panic
    if *panic_policy == ZigChimeraPanicPolicy::AllowUnwind {
        return InlineEligibility::Ineligible;
    }

    // Simple C-compatible export functions are eligible for inlining
    if func.is_exported && func.callconv == "C" {
        return InlineEligibility::Eligible;
    }

    InlineEligibility::Unknown
}

/// Lower a Zig type ID to ChimeraIR type
fn lower_zig_type_id(type_id: u64, module: &DialectModule) -> ZigChimeraType {
    module
        .types
        .iter()
        .find(|t| t.id == type_id)
        .map(|t| lower_zig_type_to_chimera_with_module(t, module))
        .unwrap_or(ZigChimeraType::Opaque)
}

/// Lower a ZigType to ChimeraIR type (with module for lookups)
pub fn lower_zig_type_to_chimera(ty: &ZigType) -> ZigChimeraType {
    lower_zig_type_to_chimera_with_module(ty, &DialectModule::new("".to_string(), "".to_string()))
}

/// Lower a ZigType to ChimeraIR type with module context
fn lower_zig_type_to_chimera_with_module(ty: &ZigType, module: &DialectModule) -> ZigChimeraType {
    match &ty.kind {
        ZigTypeKind::Int { width, signed } => match (*width, *signed) {
            (8, true) => ZigChimeraType::I8,
            (16, true) => ZigChimeraType::I16,
            (32, true) => ZigChimeraType::I32,
            (64, true) => ZigChimeraType::I64,
            (128, true) => ZigChimeraType::I128,
            (8, false) => ZigChimeraType::U8,
            (16, false) => ZigChimeraType::U16,
            (32, false) => ZigChimeraType::U32,
            (64, false) => ZigChimeraType::U64,
            (128, false) => ZigChimeraType::U128,
            _ => ZigChimeraType::Opaque,
        },
        ZigTypeKind::Float { width } => match *width {
            32 => ZigChimeraType::F32,
            64 => ZigChimeraType::F64,
            _ => ZigChimeraType::Opaque,
        },
        ZigTypeKind::Bool => ZigChimeraType::Bool,
        ZigTypeKind::Void => ZigChimeraType::Never,
        ZigTypeKind::Pointer => ZigChimeraType::Pointer(Box::new(ZigChimeraType::Opaque)),
        ZigTypeKind::Opaque => ZigChimeraType::Opaque,
        ZigTypeKind::ErrorSet { .. } => ZigChimeraType::Opaque,
        ZigTypeKind::ErrorUnion { .. } => ZigChimeraType::Error(Box::new(ZigChimeraType::Opaque)),
        ZigTypeKind::Optional { inner } => {
            let inner_type = lower_zig_type_id(*inner, module);
            ZigChimeraType::Optional(Box::new(inner_type))
        }
        ZigTypeKind::Slice { elem_type } => {
            let elem = lower_zig_type_id(*elem_type, module);
            ZigChimeraType::Slice(Box::new(elem))
        }
        ZigTypeKind::Array { elem_type, len } => {
            let elem = lower_zig_type_id(*elem_type, module);
            ZigChimeraType::Array(Box::new(elem), *len)
        }
        ZigTypeKind::Struct {
            field_types,
            field_names,
            ..
        } => {
            let fields: Vec<(String, ZigChimeraType)> = field_names
                .iter()
                .zip(field_types.iter())
                .map(|(name, type_id)| (name.clone(), lower_zig_type_id(*type_id, module)))
                .collect();
            ZigChimeraType::Struct {
                name: format!("struct_{}", ty.id),
                fields,
            }
        }
        ZigTypeKind::Enum { variants, .. } => ZigChimeraType::Enum {
            name: format!("enum_{}", ty.id),
            variants: variants.clone(),
        },
        ZigTypeKind::Union { variants } => {
            // Union lowered as struct with variants
            ZigChimeraType::Struct {
                name: format!("union_{}", ty.id),
                fields: variants
                    .iter()
                    .map(|(name, type_id)| (name.clone(), lower_zig_type_id(*type_id, module)))
                    .collect(),
            }
        }
        ZigTypeKind::Fn { .. } => ZigChimeraType::Opaque,
        ZigTypeKind::Vector { .. } => ZigChimeraType::Opaque,
        ZigTypeKind::Type => ZigChimeraType::Opaque,
    }
}

/// Get the size of a Zig type by ID
fn zig_type_size(type_id: u64, module: &DialectModule) -> u64 {
    module
        .types
        .iter()
        .find(|t| t.id == type_id)
        .map(|t| t.size_bytes)
        .unwrap_or(0)
}

/// Get the alignment of a Zig type by ID
fn zig_type_alignment(type_id: u64, module: &DialectModule) -> u64 {
    module
        .types
        .iter()
        .find(|t| t.id == type_id)
        .map(|t| t.align_bytes)
        .unwrap_or(1)
}

/// Lower a ZigType to a ChimeraIR type definition
fn lower_zig_type(ty: &ZigType, module: &DialectModule) -> Option<ZigChimeraTypeDef> {
    match &ty.kind {
        ZigTypeKind::Struct {
            field_types,
            field_names,
            field_offsets,
            packed,
            ..
        } => {
            let fields: Vec<(String, ZigChimeraType)> = field_names
                .iter()
                .zip(field_types.iter())
                .map(|(name, type_id)| {
                    let chimera_type = lower_zig_type_id(*type_id, module);
                    (name.clone(), chimera_type)
                })
                .collect();

            let field_layouts: Vec<ZigChimeraFieldLayout> = field_names
                .iter()
                .zip(field_offsets.iter())
                .zip(field_types.iter())
                .map(|((name, offset), type_id)| {
                    let field_size = zig_type_size(*type_id, module);
                    let field_align = zig_type_alignment(*type_id, module);
                    ZigChimeraFieldLayout {
                        name: name.clone(),
                        offset: *offset,
                        size: field_size,
                        alignment: field_align,
                    }
                })
                .collect();

            Some(ZigChimeraTypeDef {
                name: format!("T{}", ty.id),
                kind: ZigChimeraTypeDefKind::Struct { fields },
                layout: Some(ZigChimeraLayoutFact {
                    size_bytes: ty.size_bytes,
                    alignment_bytes: ty.align_bytes,
                    abi_kind: if *packed { "Packed" } else { "Struct" }.to_string(),
                    field_layouts,
                }),
            })
        }
        ZigTypeKind::Enum {
            tag_type: _,
            variants,
        } => Some(ZigChimeraTypeDef {
            name: format!("T{}", ty.id),
            kind: ZigChimeraTypeDefKind::Enum {
                variants: variants
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (name.clone(), Some(i as i64), vec![]))
                    .collect(),
            },
            layout: Some(ZigChimeraLayoutFact {
                size_bytes: ty.size_bytes,
                alignment_bytes: ty.align_bytes,
                abi_kind: "Scalar".to_string(),
                field_layouts: vec![],
            }),
        }),
        ZigTypeKind::Union { variants } => Some(ZigChimeraTypeDef {
            name: format!("T{}", ty.id),
            kind: ZigChimeraTypeDefKind::Union {
                fields: variants
                    .iter()
                    .map(|(name, type_id)| (name.clone(), lower_zig_type_id(*type_id, module)))
                    .collect(),
            },
            layout: Some(ZigChimeraLayoutFact {
                size_bytes: ty.size_bytes,
                alignment_bytes: ty.align_bytes,
                abi_kind: "Union".to_string(),
                field_layouts: vec![],
            }),
        }),
        ZigTypeKind::ErrorSet { errors } => Some(ZigChimeraTypeDef {
            name: format!("T{}", ty.id),
            kind: ZigChimeraTypeDefKind::ErrorSet {
                errors: errors.clone(),
            },
            layout: None,
        }),
        _ => None,
    }
}

/// Convert Zig ChimeraIR module to text format
pub fn to_chimera_text(module: &ZigChimeraModule) -> String {
    let mut output = String::new();
    output.push_str(&format!("module @{} {{\n", module.name));

    // Output imports
    for import in &module.imports {
        output.push_str(&format!(
            "  import:{} @{} {} {}\n",
            import.symbol, import.symbol, import.abi, import.signature
        ));
    }

    // Output type definitions
    for ty in &module.types {
        output.push_str(&format!("  type @{} = ", ty.name));
        match &ty.kind {
            ZigChimeraTypeDefKind::Struct { fields } => {
                output.push_str("{\n");
                for (fname, fty) in fields {
                    output.push_str(&format!(
                        "    {}: {},\n",
                        fname,
                        zig_chimera_type_to_string(fty)
                    ));
                }
                output.push_str("  }\n");
            }
            ZigChimeraTypeDefKind::Enum { variants } => {
                output.push_str("enum {\n");
                for (vname, disc, _) in variants {
                    let disc_str = disc.map(|d| format!(" = {}", d)).unwrap_or_default();
                    output.push_str(&format!("    {}{},\n", vname, disc_str));
                }
                output.push_str("}\n");
            }
            ZigChimeraTypeDefKind::Union { fields } => {
                output.push_str("union {\n");
                for (fname, fty) in fields {
                    output.push_str(&format!(
                        "    {}: {},\n",
                        fname,
                        zig_chimera_type_to_string(fty)
                    ));
                }
                output.push_str("  }\n");
            }
            ZigChimeraTypeDefKind::ErrorSet { errors } => {
                output.push_str("error {\n");
                for e in errors {
                    output.push_str(&format!("    {},\n", e));
                }
                output.push_str("}\n");
            }
        }
    }

    // Output items
    for item in &module.items {
        let panic_str = item
            .panic_policy
            .map(|p| format!(" // panic: {:?}", p))
            .unwrap_or_default();

        // Show export symbol if available (Task 21)
        let symbol_str = item
            .abi_attrs
            .as_ref()
            .map(|attrs| format!("[symbol={}]", attrs.symbol))
            .unwrap_or_default();

        // Show inline eligibility (Task 34)
        let inline_str = match item.inline_eligibility {
            InlineEligibility::Eligible => " // inline: eligible".to_string(),
            InlineEligibility::Ineligible => " // inline: ineligible".to_string(),
            InlineEligibility::Unknown => String::new(),
        };

        output.push_str(&format!(
            "  {} @{} {}{}{} ",
            item.abi, item.name, symbol_str, panic_str, inline_str
        ));
        match &item.kind {
            ZigChimeraItemKind::Function {
                params,
                return_type,
            } => {
                output.push_str("(");
                output.push_str(
                    &params
                        .iter()
                        .map(zig_chimera_type_to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                output.push_str(&format!(
                    ") -> {} {{...}}\n",
                    zig_chimera_type_to_string(return_type)
                ));
            }
            ZigChimeraItemKind::Global {
                ty, is_exported, ..
            } => {
                output.push_str(&format!(
                    ": {} {}\n",
                    zig_chimera_type_to_string(ty),
                    if *is_exported { "export" } else { "const" }
                ));
            }
            ZigChimeraItemKind::TypeDef { .. } => {
                output.push_str("type\n");
            }
        }
    }

    output.push_str("}\n");
    output
}

/// Convert Zig Chimera type to string
pub fn zig_chimera_type_to_string(t: &ZigChimeraType) -> String {
    match t {
        ZigChimeraType::Never => "!void".to_string(),
        ZigChimeraType::Bool => "i1".to_string(),
        ZigChimeraType::I8 => "i8".to_string(),
        ZigChimeraType::I16 => "i16".to_string(),
        ZigChimeraType::I32 => "i32".to_string(),
        ZigChimeraType::I64 => "i64".to_string(),
        ZigChimeraType::I128 => "i128".to_string(),
        ZigChimeraType::Isize => "isize".to_string(),
        ZigChimeraType::U8 => "u8".to_string(),
        ZigChimeraType::U16 => "u16".to_string(),
        ZigChimeraType::U32 => "u32".to_string(),
        ZigChimeraType::U64 => "u64".to_string(),
        ZigChimeraType::U128 => "u128".to_string(),
        ZigChimeraType::Usize => "usize".to_string(),
        ZigChimeraType::F32 => "f32".to_string(),
        ZigChimeraType::F64 => "f64".to_string(),
        ZigChimeraType::Pointer(p) => format!("ptr<{}>", zig_chimera_type_to_string(p)),
        ZigChimeraType::Slice(t) => format!("[]{}>", zig_chimera_type_to_string(t)),
        ZigChimeraType::Array(t, size) => format!("[{} x {}]", size, zig_chimera_type_to_string(t)),
        ZigChimeraType::Optional(t) => format!("!?{}>", zig_chimera_type_to_string(t)),
        ZigChimeraType::Error(t) => format!("!error<{}>", zig_chimera_type_to_string(t)),
        ZigChimeraType::Struct { name, fields } => {
            format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, zig_chimera_type_to_string(t)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ZigChimeraType::Enum { name, variants } => {
            format!("enum<{}>", variants.join(", "))
        }
        ZigChimeraType::Opaque => "!ch.opaque".to_string(),
    }
}

/// Convert Zig ChimeraIR module to metadata document
pub fn to_zig_chimera_meta(module: &ZigChimeraModule, crate_name: &str, target: &str) -> Metadata {
    let mut meta = Metadata::default();
    meta.version = Version::new(0, 1, 0);
    meta.module = Some(Module {
        name: module.name.clone(),
        target: target.to_string(),
        source_lang: SourceLanguage::Zig,
    });

    for item in &module.items {
        if matches!(item.kind, ZigChimeraItemKind::Function { .. }) {
            let mut func = Function {
                name: item.name.clone(),
                export: true,
                cconv: Some(item.abi.clone()),
                ..Default::default()
            };

            // Check for panic effects
            if item.effects.iter().any(|e| e.contains("panic")) {
                meta.panic_policy = Some(PanicPolicyMetadata {
                    policy: MetaPanicPolicy::Abort,
                    catches: vec![],
                    aborts: vec![],
                });
            }

            meta.functions.push(func);
        }
    }

    for ty_def in &module.types {
        let layout = LayoutMetadata {
            name: ty_def.name.clone(),
            size: ty_def.layout.as_ref().map(|l| l.size_bytes).unwrap_or(0),
            align: ty_def
                .layout
                .as_ref()
                .map(|l| l.alignment_bytes)
                .unwrap_or(0),
            fields: vec![],
            is_packed: false,
        };
        meta.layouts.push(layout);
    }

    meta
}

/// Convert Zig ChimeraIR module to object file
pub fn to_zig_chimera_object(
    module: &ZigChimeraModule,
    crate_name: &str,
    target: &str,
) -> ObjectFile {
    let meta = to_zig_chimera_meta(module, crate_name, target);
    let trust = TrustMetadata::new(TrustLevel::Generated);
    let payload = serde_json::to_string_pretty(&module)
        .unwrap_or_default()
        .into_bytes();
    ObjectFile::new_with_trust(
        target.to_string(),
        payload,
        PayloadKind::TextualIR,
        meta,
        trust,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zigmera_dialect::{Block, DialectFunction, DialectModule};

    #[test]
    fn test_zig_chimera_type_primitives() {
        assert_eq!(ZigChimeraType::I32, ZigChimeraType::I32);
        assert_eq!(ZigChimeraType::U64, ZigChimeraType::U64);
        assert_eq!(ZigChimeraType::Bool, ZigChimeraType::Bool);
    }

    #[test]
    fn test_zig_chimera_type_default() {
        let t = ZigChimeraType::default();
        assert_eq!(t, ZigChimeraType::Never);
    }

    #[test]
    fn test_zig_chimera_panic_policy_from_str() {
        assert_eq!(
            ZigChimeraPanicPolicy::from("allow_unwind"),
            ZigChimeraPanicPolicy::AllowUnwind
        );
        assert_eq!(
            ZigChimeraPanicPolicy::from("bounds_check_only"),
            ZigChimeraPanicPolicy::BoundsCheckOnly
        );
        assert_eq!(
            ZigChimeraPanicPolicy::from("unknown"),
            ZigChimeraPanicPolicy::Never
        );
    }

    #[test]
    fn test_zig_chimera_item_function() {
        let item = ZigChimeraItem {
            name: "add".to_string(),
            kind: ZigChimeraItemKind::Function {
                params: vec![ZigChimeraType::I32, ZigChimeraType::I32],
                return_type: Box::new(ZigChimeraType::I32),
            },
            abi: "C".to_string(),
            location: None,
            abi_attrs: None,
            panic_policy: Some(ZigChimeraPanicPolicy::Never),
            effects: vec!["none".to_string()],
            inline_eligibility: InlineEligibility::Eligible,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("add"));
        assert!(json.contains("I32"));
    }

    #[test]
    fn test_zig_chimera_module() {
        let module = ZigChimeraModule {
            name: "test_zig".to_string(),
            items: vec![ZigChimeraItem {
                name: "test_fn".to_string(),
                kind: ZigChimeraItemKind::Function {
                    params: vec![ZigChimeraType::U64],
                    return_type: Box::new(ZigChimeraType::U64),
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ZigChimeraPanicPolicy::Never),
                effects: vec![],
                inline_eligibility: InlineEligibility::Eligible,
            }],
            types: vec![],
            imports: vec![],
        };
        let json = serde_json::to_string(&module).unwrap();
        assert!(json.contains("test_zig"));
        assert!(json.contains("test_fn"));
    }

    #[test]
    fn test_zig_chimera_type_struct() {
        let ty = ZigChimeraType::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), ZigChimeraType::I32),
                ("y".to_string(), ZigChimeraType::I32),
            ],
        };
        let s = zig_chimera_type_to_string(&ty);
        assert!(s.contains("x"));
        assert!(s.contains("y"));
        assert!(s.contains("i32"));
    }

    #[test]
    fn test_zig_chimera_type_optional() {
        let ty = ZigChimeraType::Optional(Box::new(ZigChimeraType::I32));
        let s = zig_chimera_type_to_string(&ty);
        assert!(s.contains("!"));
    }

    #[test]
    fn test_zig_chimera_type_array() {
        let ty = ZigChimeraType::Array(Box::new(ZigChimeraType::U8), 10);
        let s = zig_chimera_type_to_string(&ty);
        assert!(s.contains("[10"));
        assert!(s.contains("u8"));
    }

    #[test]
    fn test_zig_abi_attrs_export() {
        let attrs = ZigAbiAttrs {
            source_lang: "zig".to_string(),
            symbol: "my_export".to_string(),
            calling_convention: "C".to_string(),
            panic_policy: "Never".to_string(),
            effect_set: vec!["none".to_string()],
            trust_level: "Trusted".to_string(),
        };
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains("zig"));
        assert!(json.contains("my_export"));
        assert!(json.contains("C"));
        assert!(json.contains("Trusted"));
    }

    #[test]
    fn test_to_chimera_text() {
        let module = ZigChimeraModule {
            name: "test_module".to_string(),
            items: vec![ZigChimeraItem {
                name: "my_fn".to_string(),
                kind: ZigChimeraItemKind::Function {
                    params: vec![ZigChimeraType::I32],
                    return_type: Box::new(ZigChimeraType::I64),
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ZigChimeraPanicPolicy::Never),
                effects: vec![],
                inline_eligibility: InlineEligibility::Eligible,
            }],
            types: vec![],
            imports: vec![],
        };
        let text = to_chimera_text(&module);
        assert!(text.contains("module @test_module"));
        assert!(text.contains("my_fn"));
    }

    #[test]
    fn test_zig_chimera_layout_fact() {
        let layout = ZigChimeraLayoutFact {
            size_bytes: 8,
            alignment_bytes: 4,
            abi_kind: "Aggregate".to_string(),
            field_layouts: vec![
                ZigChimeraFieldLayout {
                    name: "x".to_string(),
                    offset: 0,
                    size: 4,
                    alignment: 4,
                },
                ZigChimeraFieldLayout {
                    name: "y".to_string(),
                    offset: 4,
                    size: 4,
                    alignment: 4,
                },
            ],
        };
        let json = serde_json::to_string(&layout).unwrap();
        assert!(json.contains("8"));
        assert!(json.contains("Aggregate"));
        assert!(json.contains("x"));
    }

    #[test]
    fn test_zig_chimera_type_def_struct() {
        let type_def = ZigChimeraTypeDef {
            name: "Point".to_string(),
            kind: ZigChimeraTypeDefKind::Struct {
                fields: vec![
                    ("x".to_string(), ZigChimeraType::I32),
                    ("y".to_string(), ZigChimeraType::I32),
                ],
            },
            layout: Some(ZigChimeraLayoutFact {
                size_bytes: 8,
                alignment_bytes: 4,
                abi_kind: "Struct".to_string(),
                field_layouts: vec![
                    ZigChimeraFieldLayout {
                        name: "x".to_string(),
                        offset: 0,
                        size: 4,
                        alignment: 4,
                    },
                    ZigChimeraFieldLayout {
                        name: "y".to_string(),
                        offset: 4,
                        size: 4,
                        alignment: 4,
                    },
                ],
            }),
        };
        let json = serde_json::to_string(&type_def).unwrap();
        assert!(json.contains("Point"));
        assert!(json.contains("x"));
        assert!(json.contains("y"));
        // Verify layout is included
        assert!(json.contains("8")); // size_bytes
        assert!(json.contains("4")); // alignment_bytes
        assert!(json.contains("offset"));
    }

    #[test]
    fn test_zig_chimera_import() {
        let import = ZigChimeraImport {
            symbol: "external_add".to_string(),
            abi: "C".to_string(),
            signature: "(i32, i32) -> i32".to_string(),
        };
        let json = serde_json::to_string(&import).unwrap();
        assert!(json.contains("external_add"));
        assert!(json.contains("C"));
    }

    #[test]
    fn test_zig_chimera_panic_policy_serialization() {
        let policy = ZigChimeraPanicPolicy::AllowUnwind;
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("AllowUnwind"));

        let policy2 = ZigChimeraPanicPolicy::BoundsCheckOnly;
        let json2 = serde_json::to_string(&policy2).unwrap();
        assert!(json2.contains("BoundsCheckOnly"));
    }

    #[test]
    fn test_extract_zig_effects_basic() {
        let func = DialectFunction::new("test".to_string(), 1, 0);
        let effects = extract_zig_effects(&func);
        assert!(effects.contains(&"none".to_string()));
    }

    #[test]
    fn test_to_zig_chimera_meta() {
        let module = ZigChimeraModule {
            name: "test_zig".to_string(),
            items: vec![ZigChimeraItem {
                name: "test_fn".to_string(),
                kind: ZigChimeraItemKind::Function {
                    params: vec![ZigChimeraType::I32],
                    return_type: Box::new(ZigChimeraType::I64),
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ZigChimeraPanicPolicy::Never),
                effects: vec![],
                inline_eligibility: InlineEligibility::Eligible,
            }],
            types: vec![ZigChimeraTypeDef {
                name: "TestStruct".to_string(),
                kind: ZigChimeraTypeDefKind::Struct {
                    fields: vec![
                        ("x".to_string(), ZigChimeraType::I32),
                        ("y".to_string(), ZigChimeraType::I32),
                    ],
                },
                layout: Some(ZigChimeraLayoutFact {
                    size_bytes: 8,
                    alignment_bytes: 4,
                    abi_kind: "Aggregate".to_string(),
                    field_layouts: vec![],
                }),
            }],
            imports: vec![],
        };

        let meta = to_zig_chimera_meta(&module, "test_zig", "x86_64-unknown-linux-gnu");
        assert!(meta.module.is_some());
        let mod_info = meta.module.unwrap();
        assert_eq!(mod_info.name, "test_zig");
        assert_eq!(mod_info.source_lang, SourceLanguage::Zig);
        assert!(meta.functions.len() >= 1);
        assert!(meta.layouts.len() >= 1);
    }

    #[test]
    fn test_to_zig_chimera_object() {
        let module = ZigChimeraModule {
            name: "test_zig".to_string(),
            items: vec![ZigChimeraItem {
                name: "test_fn".to_string(),
                kind: ZigChimeraItemKind::Function {
                    params: vec![ZigChimeraType::I32],
                    return_type: Box::new(ZigChimeraType::I64),
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ZigChimeraPanicPolicy::Never),
                effects: vec![],
                inline_eligibility: InlineEligibility::Eligible,
            }],
            types: vec![],
            imports: vec![],
        };

        let obj = to_zig_chimera_object(&module, "test_zig", "x86_64-unknown-linux-gnu");
        assert_eq!(obj.header.magic, *b"CHOB");
        assert_eq!(obj.header.payload_kind, PayloadKind::TextualIR);
        assert!(!obj.payload.is_empty());
        let payload_str = String::from_utf8_lossy(&obj.payload);
        assert!(payload_str.contains("test_fn"));
        assert!(obj.trust.is_some());
    }

    #[test]
    fn test_zig_chimera_type_enums() {
        let ty = ZigChimeraType::Enum {
            name: "Color".to_string(),
            variants: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        };
        let s = zig_chimera_type_to_string(&ty);
        assert!(s.contains("Red"));
        assert!(s.contains("Green"));
        assert!(s.contains("Blue"));
    }

    #[test]
    fn test_zig_chimera_type_error() {
        let ty = ZigChimeraType::Error(Box::new(ZigChimeraType::I32));
        let s = zig_chimera_type_to_string(&ty);
        assert!(s.contains("error"));
        assert!(s.contains("i32"));
    }

    #[test]
    fn test_zig_chimera_type_def_enum() {
        let type_def = ZigChimeraTypeDef {
            name: "Color".to_string(),
            kind: ZigChimeraTypeDefKind::Enum {
                variants: vec![
                    ("Red".to_string(), Some(0), vec![]),
                    ("Green".to_string(), Some(1), vec![]),
                    ("Blue".to_string(), Some(2), vec![]),
                ],
            },
            layout: Some(ZigChimeraLayoutFact {
                size_bytes: 1,
                alignment_bytes: 1,
                abi_kind: "Scalar".to_string(),
                field_layouts: vec![],
            }),
        };
        let json = serde_json::to_string(&type_def).unwrap();
        assert!(json.contains("Color"));
        assert!(json.contains("Red"));
        // Verify layout is included
        assert!(json.contains("1")); // size_bytes
        assert!(json.contains("Scalar")); // abi_kind
    }

    #[test]
    fn test_zig_chimera_type_def_error_set() {
        let type_def = ZigChimeraTypeDef {
            name: "MyError".to_string(),
            kind: ZigChimeraTypeDefKind::ErrorSet {
                errors: vec!["ErrOne".to_string(), "ErrTwo".to_string()],
            },
            layout: None,
        };
        let json = serde_json::to_string(&type_def).unwrap();
        assert!(json.contains("MyError"));
        assert!(json.contains("ErrOne"));
        assert!(json.contains("ErrTwo"));
    }

    #[test]
    fn test_zig_chimera_type_def_union() {
        let type_def = ZigChimeraTypeDef {
            name: "MyUnion".to_string(),
            kind: ZigChimeraTypeDefKind::Union {
                fields: vec![
                    ("int_val".to_string(), ZigChimeraType::I32),
                    ("float_val".to_string(), ZigChimeraType::F64),
                ],
            },
            layout: Some(ZigChimeraLayoutFact {
                size_bytes: 8,
                alignment_bytes: 8,
                abi_kind: "Union".to_string(),
                field_layouts: vec![
                    ZigChimeraFieldLayout {
                        name: "int_val".to_string(),
                        offset: 0,
                        size: 4,
                        alignment: 4,
                    },
                    ZigChimeraFieldLayout {
                        name: "float_val".to_string(),
                        offset: 0,
                        size: 8,
                        alignment: 8,
                    },
                ],
            }),
        };
        let json = serde_json::to_string(&type_def).unwrap();
        assert!(json.contains("MyUnion"));
        assert!(json.contains("int_val"));
        assert!(json.contains("float_val"));
        // Verify layout is included
        assert!(json.contains("8")); // size_bytes
        assert!(json.contains("Union")); // abi_kind
    }
}
