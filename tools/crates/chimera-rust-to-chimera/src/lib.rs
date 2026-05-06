//! Lower Rust dialect to ChimeraIR.
//!
//! This crate handles lowering from Rust dialect types to Chimera IR
//! textual format (.chimera) and/or Chimera MLIR.

use chimera_meta::{
    FieldLayout, Function, LayoutMetadata, Metadata, Module, PanicPolicy, PanicPolicyMetadata,
    Signature, SourceLanguage, Version,
};
use chimera_object::{ObjectFile, PayloadKind, TrustLevel, TrustMetadata};
use chimera_rust_dialect::{
    BorrowKind, DialectContext, EnumDialect, EnumRepr, FieldDialect, FnDialect, FnEffects,
    ItemDialect, ItemKind, Lifetime, SafetyMode, StaticDialect, StructDialect, StructRepr,
    TypeDialect, UnionDialect, VariantDialect,
};
use chimera_rust_layout::{AbiKind, LayoutFact};
use chimera_rust_mir_import::NormalizedMirBody;
use chimera_rust_schema::ItemId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraModule {
    pub name: String,
    pub items: Vec<ChimeraItem>,
    pub types: Vec<ChimeraTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraItem {
    pub name: String,
    pub kind: ChimeraItemKind,
    pub abi: String,
    pub location: Option<SourceLocation>,
    pub abi_attrs: Option<RustAbiAttrs>,
    /// Panic policy for functions (Task 17)
    pub panic_policy: Option<ChimeraPanicPolicy>,
}

/// Panic policy for ChimeraIR functions (Task 17)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChimeraPanicPolicy {
    /// Function never panics
    Never,
    /// Function panics by aborting (extern "C" default)
    Abort,
    /// Function panics by unwinding
    Unwind,
    /// Function catches panics
    Catch,
}

impl From<&str> for ChimeraPanicPolicy {
    fn from(s: &str) -> Self {
        match s {
            "abort" => ChimeraPanicPolicy::Abort,
            "unwind" => ChimeraPanicPolicy::Unwind,
            "catch" => ChimeraPanicPolicy::Catch,
            _ => ChimeraPanicPolicy::Never,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustAbiAttrs {
    pub source_lang: String,
    pub crate_name: String,
    pub symbol: String,
    pub calling_convention: String,
    pub layout_hash: String,
    pub panic_policy: String,
    pub effect_set: Vec<String>,
    pub trust_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub span_start: u32,
    pub span_end: u32,
    pub provenance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChimeraItemKind {
    Function {
        params: Vec<ChimeraType>,
        return_type: Box<ChimeraType>,
        effects: Vec<String>,
        body: Option<String>,
    },
    Global {
        ty: ChimeraType,
        is_mutable: bool,
        is_thread_local: bool,
    },
    TypeDef {
        repr: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraTypeDef {
    pub name: String,
    pub kind: ChimeraTypeDefKind,
    /// Layout facts for this type (Task 16)
    pub layout: Option<ChimeraLayoutFact>,
}

/// Layout fact for ChimeraIR type definitions (Task 16)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraLayoutFact {
    pub size_bytes: u64,
    pub alignment_bytes: u64,
    pub abi_kind: String,
    pub field_layouts: Vec<ChimeraFieldLayout>,
}

impl From<&LayoutFact> for ChimeraLayoutFact {
    fn from(lay: &LayoutFact) -> Self {
        use chimera_rust_layout::Fields;
        let field_layouts = lay
            .fields
            .as_ref()
            .map(|Fields(fs)| {
                fs.iter()
                    .map(|f| ChimeraFieldLayout {
                        name: f.name.clone(),
                        offset: f.offset,
                        size: f.size,
                        alignment: f.alignment,
                    })
                    .collect()
            })
            .unwrap_or_default();

        ChimeraLayoutFact {
            size_bytes: lay.size_bytes,
            alignment_bytes: lay.alignment_bytes,
            abi_kind: format!("{:?}", lay.abi_kind),
            field_layouts,
        }
    }
}

/// Field layout in ChimeraIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChimeraFieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub alignment: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChimeraTypeDefKind {
    Struct {
        fields: Vec<(String, ChimeraType)>,
    },
    Enum {
        variants: Vec<(String, Option<i64>, Vec<(String, ChimeraType)>)>,
    },
    Union {
        fields: Vec<(String, ChimeraType)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChimeraType {
    Never,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Pointer(Box<ChimeraType>),
    FunctionPointer {
        params: Vec<ChimeraType>,
        return_type: Box<ChimeraType>,
        abi: String,
    },
    Struct {
        name: String,
        fields: Vec<(String, ChimeraType)>,
    },
    Enum {
        name: String,
        variants: Vec<String>,
    },
    Array(Box<ChimeraType>, u64),
    Slice(Box<ChimeraType>),
    Tuple(Vec<ChimeraType>),
    String,
    Str,
    Result {
        ok: Box<ChimeraType>,
        err: Box<ChimeraType>,
    },
    Owned(Box<ChimeraType>),
    Borrow(Box<ChimeraType>),
    Handle {
        drop_fn: String,
    },
    Opaque,
}

impl Default for ChimeraType {
    fn default() -> Self {
        ChimeraType::Never
    }
}

pub fn lower_dialect(dialect: &DialectContext) -> ChimeraModule {
    let mut items = Vec::new();
    let mut types = Vec::new();

    for (item_id, item) in &dialect.items {
        match lower_item(item_id, item) {
            LoweredResult::Item(item) => items.push(item),
            LoweredResult::Type(ty) => types.push(ty),
        }
    }

    ChimeraModule {
        name: "rust_module".to_string(),
        items,
        types,
    }
}

enum LoweredResult {
    Item(ChimeraItem),
    Type(ChimeraTypeDef),
}

fn lower_item(item_id: &ItemId, item: &ItemDialect) -> LoweredResult {
    match &item.kind {
        ItemKind::Fn(fn_dialect) => LoweredResult::Item(lower_fn(item.name.clone(), fn_dialect)),
        ItemKind::Struct(s) => LoweredResult::Type(lower_struct(item.name.clone(), s)),
        ItemKind::Enum(e) => LoweredResult::Type(lower_enum(item.name.clone(), e)),
        ItemKind::Union(u) => LoweredResult::Type(lower_union(item.name.clone(), u)),
        ItemKind::Static(s) => LoweredResult::Item(lower_static(item.name.clone(), s)),
        _ => LoweredResult::Item(ChimeraItem {
            name: item.name.clone(),
            kind: ChimeraItemKind::TypeDef { repr: None },
            abi: "Rust".to_string(),
            location: None,
            abi_attrs: None,
            panic_policy: None,
        }),
    }
}

fn lower_fn(name: String, f: &FnDialect) -> ChimeraItem {
    // Compute panic policy from ABI and effects (Task 17)
    let panic_policy = if f.effects.may_panic {
        match f.abi.as_str() {
            "C" | "system" => Some(ChimeraPanicPolicy::Abort),
            "C-unwind" => Some(ChimeraPanicPolicy::Unwind),
            "Rust" => Some(ChimeraPanicPolicy::Never), // Rust panics are caught
            _ => Some(ChimeraPanicPolicy::Abort),
        }
    } else {
        Some(ChimeraPanicPolicy::Never)
    };

    ChimeraItem {
        name,
        kind: ChimeraItemKind::Function {
            params: f.params.iter().map(|t| lower_type(t)).collect(),
            return_type: Box::new(lower_type(&f.return_type)),
            effects: effects_to_strings(&f.effects),
            body: None,
        },
        abi: f.abi.clone(),
        location: None,
        abi_attrs: None,
        panic_policy,
    }
}

fn lower_struct(name: String, s: &StructDialect) -> ChimeraTypeDef {
    // Compute layout facts for the struct (Task 16)
    let mut field_layouts = Vec::new();
    let mut offset = 0u64;
    let mut max_align = 1u64;

    for f in &s.fields {
        let field_size = type_size(&f.ty);
        let field_align = type_alignment(&f.ty);
        // Align offset to field alignment
        offset = (offset + field_align - 1) & !(field_align - 1);
        field_layouts.push(ChimeraFieldLayout {
            name: f.name.clone(),
            offset,
            size: field_size,
            alignment: field_align,
        });
        offset += field_size;
        max_align = max_align.max(field_align);
    }

    let size = (offset + max_align - 1) & !(max_align - 1);

    ChimeraTypeDef {
        name,
        kind: ChimeraTypeDefKind::Struct {
            fields: s
                .fields
                .iter()
                .map(|f| (f.name.clone(), lower_type(&f.ty)))
                .collect(),
        },
        layout: Some(ChimeraLayoutFact {
            size_bytes: size,
            alignment_bytes: max_align,
            abi_kind: format!(
                "{:?}",
                if s.fields.len() == 1 {
                    AbiKind::Scalar
                } else {
                    AbiKind::Aggregate
                }
            ),
            field_layouts,
        }),
    }
}

fn lower_enum(name: String, e: &EnumDialect) -> ChimeraTypeDef {
    // Compute layout for enum (use largest variant)
    let mut max_size = 0u64;
    let mut max_align = 1u64;
    for v in &e.variants {
        for f in &v.fields {
            let sz = type_size(&f.ty);
            let al = type_alignment(&f.ty);
            max_size = max_size.max(sz);
            max_align = max_align.max(al);
        }
    }
    let size = (max_size + max_align - 1) & !(max_align - 1);

    ChimeraTypeDef {
        name,
        kind: ChimeraTypeDefKind::Enum {
            variants: e
                .variants
                .iter()
                .map(|v| {
                    (
                        v.name.clone(),
                        v.discriminant,
                        v.fields
                            .iter()
                            .map(|f| (f.name.clone(), lower_type(&f.ty)))
                            .collect(),
                    )
                })
                .collect(),
        },
        layout: Some(ChimeraLayoutFact {
            size_bytes: size,
            alignment_bytes: max_align,
            abi_kind: "Scalar".to_string(),
            field_layouts: vec![],
        }),
    }
}

fn lower_union(name: String, u: &UnionDialect) -> ChimeraTypeDef {
    // Union layout: size is the max of field sizes, alignment is max of field alignments
    let mut max_size = 0u64;
    let mut max_align = 1u64;
    for f in &u.fields {
        let sz = type_size(&f.ty);
        let al = type_alignment(&f.ty);
        max_size = max_size.max(sz);
        max_align = max_align.max(al);
    }

    ChimeraTypeDef {
        name,
        kind: ChimeraTypeDefKind::Union {
            fields: u
                .fields
                .iter()
                .map(|f| (f.name.clone(), lower_type(&f.ty)))
                .collect(),
        },
        layout: Some(ChimeraLayoutFact {
            size_bytes: max_size,
            alignment_bytes: max_align,
            abi_kind: "Aggregate".to_string(),
            field_layouts: vec![],
        }),
    }
}

fn lower_static(name: String, s: &StaticDialect) -> ChimeraItem {
    ChimeraItem {
        name,
        kind: ChimeraItemKind::Global {
            ty: lower_type(&s.ty),
            is_mutable: s.is_mutable,
            is_thread_local: false,
        },
        abi: "Rust".to_string(),
        location: None,
        abi_attrs: None,
        panic_policy: None,
    }
}

/// Get the size of a type in bytes
fn type_size(t: &TypeDialect) -> u64 {
    match t {
        TypeDialect::Never => 0,
        TypeDialect::Unit => 0,
        TypeDialect::Bool => 1,
        TypeDialect::Char => 4,
        TypeDialect::I8 | TypeDialect::U8 => 1,
        TypeDialect::I16 | TypeDialect::U16 => 2,
        TypeDialect::I32 | TypeDialect::U32 => 4,
        TypeDialect::I64 | TypeDialect::U64 => 8,
        TypeDialect::I128 | TypeDialect::U128 => 16,
        TypeDialect::Isize | TypeDialect::Usize => 8,
        TypeDialect::F32 => 4,
        TypeDialect::F64 => 8,
        TypeDialect::Str => 0,       // unsized
        TypeDialect::Slice(_) => 16, // ptr + len
        TypeDialect::Array(t, size) => type_size(t) * size,
        TypeDialect::Tuple(ts) => ts.iter().map(type_size).sum(),
        TypeDialect::Reference(_) => 8,
        TypeDialect::Ptr(_) => 8,
        TypeDialect::FnPtr(_) => 16, // ptr + env
        TypeDialect::Adt(_, _) => 8, // pointer-sized for now
        TypeDialect::Error => 0,
    }
}

/// Get the alignment of a type in bytes
fn type_alignment(t: &TypeDialect) -> u64 {
    match t {
        TypeDialect::Never => 1,
        TypeDialect::Unit => 1,
        TypeDialect::Bool => 1,
        TypeDialect::Char => 4,
        TypeDialect::I8 | TypeDialect::U8 => 1,
        TypeDialect::I16 | TypeDialect::U16 => 2,
        TypeDialect::I32 | TypeDialect::U32 => 4,
        TypeDialect::I64 | TypeDialect::U64 => 8,
        TypeDialect::I128 | TypeDialect::U128 => 16,
        TypeDialect::Isize | TypeDialect::Usize => 8,
        TypeDialect::F32 => 4,
        TypeDialect::F64 => 8,
        TypeDialect::Str => 1,
        TypeDialect::Slice(_) => 8,
        TypeDialect::Array(t, _) => type_alignment(t),
        TypeDialect::Tuple(ts) => ts.iter().map(type_alignment).max().unwrap_or(1),
        TypeDialect::Reference(_) => 8,
        TypeDialect::Ptr(_) => 8,
        TypeDialect::FnPtr(_) => 8,
        TypeDialect::Adt(_, _) => 8,
        TypeDialect::Error => 1,
    }
}

pub fn lower_type(t: &TypeDialect) -> ChimeraType {
    match t {
        TypeDialect::Never => ChimeraType::Never,
        TypeDialect::Bool => ChimeraType::Bool,
        TypeDialect::I8 => ChimeraType::I8,
        TypeDialect::I16 => ChimeraType::I16,
        TypeDialect::I32 => ChimeraType::I32,
        TypeDialect::I64 => ChimeraType::I64,
        TypeDialect::I128 => ChimeraType::I128,
        TypeDialect::Isize => ChimeraType::Isize,
        TypeDialect::U8 => ChimeraType::U8,
        TypeDialect::U16 => ChimeraType::U16,
        TypeDialect::U32 => ChimeraType::U32,
        TypeDialect::U64 => ChimeraType::U64,
        TypeDialect::U128 => ChimeraType::U128,
        TypeDialect::Usize => ChimeraType::Usize,
        TypeDialect::F32 => ChimeraType::F32,
        TypeDialect::F64 => ChimeraType::F64,
        TypeDialect::Str => ChimeraType::Str,
        TypeDialect::Slice(t) => ChimeraType::Slice(Box::new(lower_type(t))),
        TypeDialect::Array(t, size) => ChimeraType::Array(Box::new(lower_type(t)), *size),
        TypeDialect::Tuple(ts) => ChimeraType::Tuple(ts.iter().map(lower_type).collect()),
        TypeDialect::Reference(r) => ChimeraType::Pointer(Box::new(lower_type(&r.pointee))),
        TypeDialect::Ptr(r) => ChimeraType::Pointer(Box::new(lower_type(&r.pointee))),
        TypeDialect::FnPtr(fp) => ChimeraType::FunctionPointer {
            params: fp.params.iter().map(lower_type).collect(),
            return_type: Box::new(lower_type(&fp.return_type)),
            abi: fp.abi.clone(),
        },
        TypeDialect::Adt(def_id, args) => ChimeraType::Struct {
            name: format!("adt_{}", def_id.0),
            fields: args
                .iter()
                .map(|t| ("field".to_string(), lower_type(t)))
                .collect(),
        },
        TypeDialect::Error => ChimeraType::Never,
        _ => ChimeraType::Never,
    }
}

fn effects_to_strings(e: &FnEffects) -> Vec<String> {
    let mut effects = Vec::new();
    if e.may_panic {
        effects.push("may_panic".to_string());
    }
    if e.may_alloc {
        effects.push("may_alloc".to_string());
    }
    if e.may_ffi {
        effects.push("may_ffi".to_string());
    }
    if e.may_unsafe {
        effects.push("may_unsafe".to_string());
    }
    effects
}

pub fn lower_mir_body(body: &NormalizedMirBody) -> String {
    let mut output = format!("fn @{} {{\n", body.def_path);

    for (i, local) in body.locals.iter().enumerate() {
        let kind = if local.is_return_slot {
            "return_slot"
        } else if local.is_arg {
            "arg"
        } else {
            "local"
        };
        output.push_str(&format!("  %{}: ty_{} -- {}\n", i, local.ty.0, kind));
    }

    for block in &body.blocks {
        output.push_str(&format!("block_{}:\n", block.index));
        for stmt in &block.statements {
            output.push_str(&format!("  {:?}\n", stmt));
        }
        output.push_str(&format!("  {:?}\n", block.terminator));
    }

    output.push_str("}\n");
    output
}

pub fn to_chimera_text(module: &ChimeraModule) -> String {
    let mut output = String::new();
    output.push_str(&format!("module @{} {{\n", module.name));

    for ty in &module.types {
        output.push_str(&format!("  type @{} = ", ty.name));
        match &ty.kind {
            ChimeraTypeDefKind::Struct { fields } => {
                output.push_str("{\n");
                for (fname, fty) in fields {
                    output.push_str(&format!(
                        "    {}: {},\n",
                        fname,
                        chimera_type_to_string(fty)
                    ));
                }
                output.push_str("  }\n");
            }
            ChimeraTypeDefKind::Enum { variants } => {
                output.push_str(&format!("enum {{\n"));
                for (vname, disc, _) in variants {
                    let disc_str = disc.map(|d| format!(" = {}", d)).unwrap_or_default();
                    output.push_str(&format!("    {}{},\n", vname, disc_str));
                }
                output.push_str("}\n");
            }
            ChimeraTypeDefKind::Union { fields } => {
                output.push_str("union {\n");
                for (fname, fty) in fields {
                    output.push_str(&format!(
                        "    {}: {},\n",
                        fname,
                        chimera_type_to_string(fty)
                    ));
                }
                output.push_str("  }\n");
            }
        }
    }

    for item in &module.items {
        output.push_str(&format!("  {} @{} ", item.abi, item.name));
        match &item.kind {
            ChimeraItemKind::Function {
                params,
                return_type,
                effects,
                body,
            } => {
                output.push_str("(");
                output.push_str(
                    &params
                        .iter()
                        .map(chimera_type_to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                output.push_str(&format!(") -> {} {{", chimera_type_to_string(return_type)));
                if !effects.is_empty() {
                    output.push_str(&format!(" // {}", effects.join(", ")));
                }
                if let Some(body) = body {
                    if body.trim().is_empty() {
                        output.push_str(" }\n");
                    } else {
                        output.push('\n');
                        for line in body.lines() {
                            output.push_str("    ");
                            output.push_str(line);
                            output.push('\n');
                        }
                        output.push_str("  }\n");
                    }
                } else {
                    output.push_str(" ... }\n");
                }
            }
            ChimeraItemKind::Global { ty, is_mutable, .. } => {
                output.push_str(&format!(
                    ": {} {}\n",
                    chimera_type_to_string(ty),
                    if *is_mutable { "var" } else { "const" }
                ));
            }
            ChimeraItemKind::TypeDef { .. } => {
                output.push_str("type\n");
            }
        }
    }

    output.push_str("}\n");
    output
}

/// Convert Rust lowering to Chimera metadata document (Task 129)
pub fn to_chimera_meta(module: &ChimeraModule, crate_name: &str, target: &str) -> Metadata {
    let mut meta = Metadata::default();
    meta.version = Version::new(0, 1, 0);
    meta.module = Some(Module {
        name: module.name.clone(),
        target: target.to_string(),
        source_lang: SourceLanguage::Rust,
    });

    // Convert functions
    for item in &module.items {
        if matches!(item.kind, ChimeraItemKind::Function { .. }) {
            let effects: Vec<_> = match &item.kind {
                ChimeraItemKind::Function { effects, .. } => effects.clone(),
                _ => vec![],
            };

            let mut func = Function {
                name: item.name.clone(),
                export: true,
                cconv: Some(item.abi.clone()),
                signature: None,
                ..Default::default()
            };

            // Check for may_panic effect
            if effects.iter().any(|e| e.contains("may_panic")) {
                meta.panic_policy = Some(PanicPolicyMetadata {
                    policy: PanicPolicy::Abort,
                    catches: vec![],
                    aborts: vec![],
                });
            }

            meta.functions.push(func);
        }
    }

    // Convert type layouts
    for ty_def in &module.types {
        let mut layout = LayoutMetadata {
            name: ty_def.name.clone(),
            size: 0,  // Would need actual layout from rustc
            align: 8, // Default alignment
            fields: vec![],
            is_packed: false,
        };

        if let ChimeraTypeDefKind::Struct { fields } = &ty_def.kind {
            for (fname, fty) in fields {
                layout.fields.push(FieldLayout {
                    name: fname.clone(),
                    offset: 0, // Would need actual layout
                    typ: chimera_type_to_string(fty),
                    size: 0,
                    align: 8,
                });
            }
        }

        meta.layouts.push(layout);
    }

    meta
}

/// Convert Rust lowering to Chimera object file (Task 130)
pub fn to_chimera_object(module: &ChimeraModule, crate_name: &str, target: &str) -> ObjectFile {
    let meta = to_chimera_meta(module, crate_name, target);
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

fn chimera_type_to_string(t: &ChimeraType) -> String {
    match t {
        ChimeraType::Never => "!void".to_string(),
        ChimeraType::Bool => "i1".to_string(),
        ChimeraType::I8 => "i8".to_string(),
        ChimeraType::I16 => "i16".to_string(),
        ChimeraType::I32 => "i32".to_string(),
        ChimeraType::I64 => "i64".to_string(),
        ChimeraType::I128 => "i128".to_string(),
        ChimeraType::Isize => "isize".to_string(),
        ChimeraType::U8 => "u8".to_string(),
        ChimeraType::U16 => "u16".to_string(),
        ChimeraType::U32 => "u32".to_string(),
        ChimeraType::U64 => "u64".to_string(),
        ChimeraType::U128 => "u128".to_string(),
        ChimeraType::Usize => "usize".to_string(),
        ChimeraType::F32 => "f32".to_string(),
        ChimeraType::F64 => "f64".to_string(),
        ChimeraType::Pointer(p) => format!("ptr<{}>", chimera_type_to_string(p)),
        ChimeraType::FunctionPointer {
            params,
            return_type,
            abi,
        } => {
            format!(
                "fn<{}>({}) -> {}",
                abi,
                params
                    .iter()
                    .map(chimera_type_to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                chimera_type_to_string(return_type)
            )
        }
        ChimeraType::Struct { name, fields } => {
            format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, chimera_type_to_string(t)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ChimeraType::Enum { name, variants } => {
            format!("enum<{}>", variants.join(", "))
        }
        ChimeraType::Array(t, size) => format!("[{} x {}]", size, chimera_type_to_string(t)),
        ChimeraType::Slice(t) => format!("[]{}>", chimera_type_to_string(t)),
        ChimeraType::Tuple(ts) => {
            format!(
                "({})",
                ts.iter()
                    .map(chimera_type_to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ChimeraType::String => "!ch.string".to_string(),
        ChimeraType::Str => "!ch.str".to_string(),
        ChimeraType::Result { ok, err } => {
            format!(
                "!ch.result<{}, {}>",
                chimera_type_to_string(ok),
                chimera_type_to_string(err)
            )
        }
        ChimeraType::Owned(t) => format!("!ch.owned<{}>", chimera_type_to_string(t)),
        ChimeraType::Borrow(t) => format!("!ch.borrow<{}>", chimera_type_to_string(t)),
        ChimeraType::Handle { drop_fn } => {
            format!("!ch.handle<drop = {}>", drop_fn)
        }
        ChimeraType::Opaque => "!ch.opaque".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_rust_dialect::*;

    #[test]
    fn test_chimera_type_primitives() {
        assert_eq!(ChimeraType::I32, ChimeraType::I32);
        assert_eq!(ChimeraType::U64, ChimeraType::U64);
        assert_eq!(ChimeraType::Bool, ChimeraType::Bool);
    }

    #[test]
    fn test_chimera_type_default() {
        let t = ChimeraType::default();
        assert_eq!(t, ChimeraType::Never);
    }

    #[test]
    fn test_lower_type_primitive() {
        assert_eq!(lower_type(&TypeDialect::I32), ChimeraType::I32);
        assert_eq!(lower_type(&TypeDialect::U64), ChimeraType::U64);
        assert_eq!(lower_type(&TypeDialect::Bool), ChimeraType::Bool);
        assert_eq!(lower_type(&TypeDialect::F64), ChimeraType::F64);
    }

    #[test]
    fn test_lower_type_reference() {
        let ref_type = TypeDialect::Reference(Box::new(chimera_rust_dialect::TypeRef {
            pointee: Box::new(TypeDialect::I32),
            borrow_kind: BorrowKind::Shared,
            lifetime: Lifetime::Elided,
            is_const: false,
        }));
        assert_eq!(
            lower_type(&ref_type),
            ChimeraType::Pointer(Box::new(ChimeraType::I32))
        );
    }

    #[test]
    fn test_lower_type_ptr() {
        let ptr_type = TypeDialect::Ptr(Box::new(chimera_rust_dialect::TypeRef {
            pointee: Box::new(TypeDialect::U8),
            borrow_kind: BorrowKind::Unique,
            lifetime: Lifetime::Elided,
            is_const: true,
        }));
        assert_eq!(
            lower_type(&ptr_type),
            ChimeraType::Pointer(Box::new(ChimeraType::U8))
        );
    }

    #[test]
    fn test_lower_type_slice() {
        assert_eq!(
            lower_type(&TypeDialect::Slice(Box::new(TypeDialect::I8))),
            ChimeraType::Slice(Box::new(ChimeraType::I8))
        );
    }

    #[test]
    fn test_lower_type_array() {
        assert_eq!(
            lower_type(&TypeDialect::Array(Box::new(TypeDialect::I32), 10)),
            ChimeraType::Array(Box::new(ChimeraType::I32), 10)
        );
    }

    #[test]
    fn test_lower_type_tuple() {
        assert_eq!(
            lower_type(&TypeDialect::Tuple(vec![
                TypeDialect::I32,
                TypeDialect::Bool
            ])),
            ChimeraType::Tuple(vec![ChimeraType::I32, ChimeraType::Bool])
        );
    }

    #[test]
    fn test_chimera_type_result() {
        let t = ChimeraType::Result {
            ok: Box::new(ChimeraType::U64),
            err: Box::new(ChimeraType::I32),
        };
        assert_eq!(chimera_type_to_string(&t), "!ch.result<u64, i32>");
    }

    #[test]
    fn test_chimera_type_owned() {
        let t = ChimeraType::Owned(Box::new(ChimeraType::U8));
        assert_eq!(chimera_type_to_string(&t), "!ch.owned<u8>");
    }

    #[test]
    fn test_chimera_type_borrow() {
        let t = ChimeraType::Borrow(Box::new(ChimeraType::Str));
        assert_eq!(chimera_type_to_string(&t), "!ch.borrow<!ch.str>");
    }

    #[test]
    fn test_chimera_type_handle() {
        let t = ChimeraType::Handle {
            drop_fn: "my_drop".to_string(),
        };
        assert_eq!(chimera_type_to_string(&t), "!ch.handle<drop = my_drop>");
    }

    #[test]
    fn test_chimera_type_opaque() {
        let t = ChimeraType::Opaque;
        assert_eq!(chimera_type_to_string(&t), "!ch.opaque");
    }

    #[test]
    fn test_chimera_type_string() {
        assert_eq!(chimera_type_to_string(&ChimeraType::String), "!ch.string");
        assert_eq!(chimera_type_to_string(&ChimeraType::Str), "!ch.str");
    }

    #[test]
    fn test_chimera_item_fn() {
        let item = ChimeraItem {
            name: "test_fn".to_string(),
            kind: ChimeraItemKind::Function {
                params: vec![ChimeraType::I32, ChimeraType::Bool],
                return_type: Box::new(ChimeraType::I64),
                effects: vec!["may_panic".to_string()],
                body: None,
            },
            abi: "C".to_string(),
            location: Some(SourceLocation {
                file: "test.rs".to_string(),
                line: 10,
                column: 1,
                span_start: 100,
                span_end: 150,
                provenance: Some("macro expansion".to_string()),
            }),
            abi_attrs: Some(RustAbiAttrs {
                source_lang: "rust".to_string(),
                crate_name: "test_crate".to_string(),
                symbol: "_ZN4test12test_fn17h1234567890abcdefE".to_string(),
                calling_convention: "C".to_string(),
                layout_hash: "abc123".to_string(),
                panic_policy: "abort".to_string(),
                effect_set: vec!["may_panic".to_string()],
                trust_level: "TCB".to_string(),
            }),
            panic_policy: Some(ChimeraPanicPolicy::Abort),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test_fn"));
        assert!(json.contains("may_panic"));
        assert!(json.contains("test.rs"));
        assert!(json.contains("test_crate"));
    }

    #[test]
    fn test_chimera_module() {
        let module = ChimeraModule {
            name: "test".to_string(),
            items: vec![ChimeraItem {
                name: "global_var".to_string(),
                kind: ChimeraItemKind::Global {
                    ty: ChimeraType::I32,
                    is_mutable: true,
                    is_thread_local: false,
                },
                abi: "Rust".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: None,
            }],
            types: vec![],
        };
        let json = serde_json::to_string(&module).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("global_var"));
    }

    #[test]
    fn test_effects_to_strings() {
        let effects = FnEffects {
            may_panic: true,
            may_alloc: true,
            may_ffi: false,
            may_unsafe: true,
        };
        let strings = effects_to_strings(&effects);
        assert!(strings.contains(&"may_panic".to_string()));
        assert!(strings.contains(&"may_alloc".to_string()));
        assert!(strings.contains(&"may_unsafe".to_string()));
        assert!(!strings.contains(&"may_ffi".to_string()));
    }

    #[test]
    fn test_chimera_type_to_string() {
        assert_eq!(chimera_type_to_string(&ChimeraType::I32), "i32");
        assert_eq!(chimera_type_to_string(&ChimeraType::Bool), "i1");
        assert_eq!(chimera_type_to_string(&ChimeraType::Never), "!void");
    }

    #[test]
    fn test_to_chimera_text() {
        let module = ChimeraModule {
            name: "test_module".to_string(),
            items: vec![ChimeraItem {
                name: "my_fn".to_string(),
                kind: ChimeraItemKind::Function {
                    params: vec![ChimeraType::I32],
                    return_type: Box::new(ChimeraType::I64),
                    effects: vec![],
                    body: None,
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: None,
            }],
            types: vec![],
        };
        let text = to_chimera_text(&module);
        assert!(text.contains("module @test_module"));
        assert!(text.contains("my_fn"));
    }

    #[test]
    fn test_to_chimera_text_with_body() {
        let module = ChimeraModule {
            name: "test_module".to_string(),
            items: vec![ChimeraItem {
                name: "my_fn".to_string(),
                kind: ChimeraItemKind::Function {
                    params: vec![ChimeraType::I32],
                    return_type: Box::new(ChimeraType::I32),
                    effects: vec!["may_ffi".to_string()],
                    body: Some("ret 0".to_string()),
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: None,
            }],
            types: vec![],
        };
        let text = to_chimera_text(&module);
        assert!(text.contains("ret 0"));
        assert!(!text.contains("... }"));
    }

    #[test]
    fn test_lower_fn_dialect() {
        let fn_dialect = FnDialect {
            params: vec![TypeDialect::I32, TypeDialect::Bool],
            return_type: Box::new(TypeDialect::Unit),
            effects: FnEffects::default(),
            abi: "Rust".to_string(),
        };
        let item = lower_fn("add".to_string(), &fn_dialect);
        assert_eq!(item.name, "add");
        match item.kind {
            ChimeraItemKind::Function {
                params,
                return_type,
                ..
            } => {
                assert_eq!(params.len(), 2);
            }
            _ => panic!("expected Function"),
        }
    }

    #[test]
    fn test_lower_struct_dialect() {
        let struct_dialect = StructDialect {
            fields: vec![
                FieldDialect {
                    name: "x".to_string(),
                    ty: TypeDialect::I32,
                },
                FieldDialect {
                    name: "y".to_string(),
                    ty: TypeDialect::I32,
                },
            ],
            repr: StructRepr::C,
        };
        let ty_def = lower_struct("Point".to_string(), &struct_dialect);
        assert_eq!(ty_def.name, "Point");
        match ty_def.kind {
            ChimeraTypeDefKind::Struct { fields } => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn test_lower_enum_dialect() {
        let enum_dialect = EnumDialect {
            variants: vec![
                VariantDialect {
                    name: "A".to_string(),
                    discriminant: Some(0),
                    fields: vec![],
                },
                VariantDialect {
                    name: "B".to_string(),
                    discriminant: Some(1),
                    fields: vec![FieldDialect {
                        name: "value".to_string(),
                        ty: TypeDialect::I32,
                    }],
                },
            ],
            repr: EnumRepr::U32,
        };
        let ty_def = lower_enum("MyEnum".to_string(), &enum_dialect);
        assert_eq!(ty_def.name, "MyEnum");
        match ty_def.kind {
            ChimeraTypeDefKind::Enum { variants } => {
                assert_eq!(variants.len(), 2);
            }
            _ => panic!("expected Enum"),
        }
    }

    #[test]
    fn test_chimera_item_serialization() {
        let item = ChimeraItem {
            name: "test_item".to_string(),
            kind: ChimeraItemKind::TypeDef {
                repr: Some("C".to_string()),
            },
            abi: "Rust".to_string(),
            location: None,
            abi_attrs: None,
            panic_policy: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test_item"));
    }

    #[test]
    fn test_rust_abi_attrs_serialization() {
        let attrs = RustAbiAttrs {
            source_lang: "rust".to_string(),
            crate_name: "my_crate".to_string(),
            symbol: "_ZN7my_crate12test_fn17hedef123456789aE".to_string(),
            calling_convention: "C-unwind".to_string(),
            layout_hash: "def456".to_string(),
            panic_policy: "unwind".to_string(),
            effect_set: vec!["may_panic".to_string(), "may_alloc".to_string()],
            trust_level: "Trusted".to_string(),
        };
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("my_crate"));
        assert!(json.contains("C-unwind"));
        assert!(json.contains("unwind"));
    }

    #[test]
    fn test_chimera_type_def_serialization() {
        let ty_def = ChimeraTypeDef {
            name: "TestType".to_string(),
            kind: ChimeraTypeDefKind::Struct {
                fields: vec![
                    ("field1".to_string(), ChimeraType::I32),
                    ("field2".to_string(), ChimeraType::Bool),
                ],
            },
            layout: None,
        };
        let json = serde_json::to_string(&ty_def).unwrap();
        assert!(json.contains("TestType"));
        assert!(json.contains("field1"));
    }

    #[test]
    fn test_to_chimera_meta() {
        let module = ChimeraModule {
            name: "test_crate".to_string(),
            items: vec![ChimeraItem {
                name: "test_fn".to_string(),
                kind: ChimeraItemKind::Function {
                    params: vec![ChimeraType::I32],
                    return_type: Box::new(ChimeraType::I64),
                    effects: vec!["may_panic".to_string()],
                    body: None,
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ChimeraPanicPolicy::Abort),
            }],
            types: vec![ChimeraTypeDef {
                name: "TestStruct".to_string(),
                kind: ChimeraTypeDefKind::Struct {
                    fields: vec![
                        ("x".to_string(), ChimeraType::I32),
                        ("y".to_string(), ChimeraType::I32),
                    ],
                },
                layout: None,
            }],
        };

        let meta = to_chimera_meta(&module, "test_crate", "x86_64-unknown-linux-gnu");
        assert!(meta.module.is_some());
        let mod_info = meta.module.unwrap();
        assert_eq!(mod_info.name, "test_crate");
        assert_eq!(mod_info.source_lang, SourceLanguage::Rust);
        assert!(meta.functions.len() >= 1);
        assert!(meta.layouts.len() >= 1);
        assert!(meta.panic_policy.is_some());
    }

    #[test]
    fn test_to_chimera_object() {
        let module = ChimeraModule {
            name: "test_crate".to_string(),
            items: vec![ChimeraItem {
                name: "test_fn".to_string(),
                kind: ChimeraItemKind::Function {
                    params: vec![ChimeraType::I32],
                    return_type: Box::new(ChimeraType::I64),
                    effects: vec![],
                    body: None,
                },
                abi: "C".to_string(),
                location: None,
                abi_attrs: None,
                panic_policy: Some(ChimeraPanicPolicy::Abort),
            }],
            types: vec![],
        };

        let obj = to_chimera_object(&module, "test_crate", "x86_64-unknown-linux-gnu");
        assert_eq!(obj.header.magic, *b"CHOB");
        assert_eq!(obj.header.payload_kind, PayloadKind::TextualIR);
        assert!(!obj.payload.is_empty());
        let payload_str = String::from_utf8_lossy(&obj.payload);
        assert!(payload_str.contains("test_fn"));
        assert!(obj.trust.is_some());
    }

    #[test]
    fn test_chimera_panic_policy_serialization() {
        let policy = ChimeraPanicPolicy::Abort;
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("Abort"));

        let policy2 = ChimeraPanicPolicy::Unwind;
        let json2 = serde_json::to_string(&policy2).unwrap();
        assert!(json2.contains("Unwind"));
    }

    #[test]
    fn test_chimera_panic_policy_from_str() {
        assert_eq!(ChimeraPanicPolicy::from("abort"), ChimeraPanicPolicy::Abort);
        assert_eq!(
            ChimeraPanicPolicy::from("unwind"),
            ChimeraPanicPolicy::Unwind
        );
        assert_eq!(ChimeraPanicPolicy::from("catch"), ChimeraPanicPolicy::Catch);
        assert_eq!(
            ChimeraPanicPolicy::from("unknown"),
            ChimeraPanicPolicy::Never
        );
    }

    #[test]
    fn test_chimera_item_with_panic_policy() {
        let item = ChimeraItem {
            name: "panic_fn".to_string(),
            kind: ChimeraItemKind::Function {
                params: vec![ChimeraType::I32],
                return_type: Box::new(ChimeraType::I32),
                effects: vec!["may_panic".to_string()],
                body: None,
            },
            abi: "C".to_string(),
            location: None,
            abi_attrs: None,
            panic_policy: Some(ChimeraPanicPolicy::Abort),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("panic_fn"));
        assert!(json.contains("Abort"));
    }
}
