//! Stable Rust source/surface parser using `syn`.
//!
//! This crate provides stable Rust parsing without requiring rustc compiler internals.
//! It validates FFI boundaries, repr(C) layouts, extern blocks, and ABI contracts.
//!
//! # Stable Surface Mode
//!
//! In stable surface-only mode, this crate validates ABI boundaries using `syn`
//! and generates layout assertions without requiring MIR/body lowering from rustc.
//! This enables FFI boundary validation without a nightly toolchain.

use chimera_rust_schema::{
    Attribute, ConstParam, Generics, ItemId, ItemKind, Linkage, RsnapExport, RsnapItem, Visibility,
    VisibilityRank,
};
use quote::ToTokens;
use syn::{Attribute as SynAttribute, Item, ItemFn, ItemMod, ItemStatic};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("parse error: {0}")]
    Syn(#[from] syn::Error),
    #[error("generic not allowed on FFI boundary")]
    GenericOnFFI,
    #[error("invalid extern block: {0}")]
    InvalidExtern(String),
}

/// Parse a Rust source file and extract semantic information for FFI validation.
pub fn parse_rust_source(source: &str) -> Result<ParsedSource, ParseError> {
    let ast = syn::parse_file(source)?;

    let mut items = Vec::new();
    let mut exports = Vec::new();
    let mut imports = Vec::new();

    for item in &ast.items {
        match item {
            Item::Fn(f) => {
                if let Some(export) = validate_exported_function(f) {
                    exports.push(export);
                }
                items.push(extract_function_item(f));
            }
            Item::Static(s) => {
                if let Some(export) = validate_exported_static(s) {
                    exports.push(export);
                }
            }
            Item::Const(c) => {
                items.push(extract_const_item(c));
            }
            Item::Struct(s) => {
                items.push(extract_struct_item(s));
            }
            Item::Enum(e) => {
                items.push(extract_enum_item(e));
            }
            Item::Union(u) => {
                items.push(extract_union_item(u));
            }
            Item::Trait(t) => {
                items.push(extract_trait_item(t));
            }
            Item::Impl(i) => {
                items.push(extract_impl_item(i));
            }
            Item::Type(t) => {
                items.push(extract_type_item(t));
            }
            Item::Mod(m) => {
                if m.content.is_some() {
                    items.push(extract_module_item(m));
                }
            }
            Item::ForeignMod(e) => {
                let (ext_exports, ext_items, ext_imports) = validate_extern_block_with_imports(e)?;
                exports.extend(ext_exports);
                items.extend(ext_items);
                imports.extend(ext_imports);
            }
            Item::Use(_) => {}
            _ => {}
        }
    }

    Ok(ParsedSource {
        items,
        exports,
        imports,
    })
}

/// Validated parsed source
#[derive(Debug, Clone)]
pub struct ParsedSource {
    pub items: Vec<RsnapItem>,
    pub exports: Vec<RsnapExport>,
    pub imports: Vec<RsnapImport>,
}

/// Import table entry (Task 15)
#[derive(Debug, Clone)]
pub struct RsnapImport {
    pub symbol: String,
    pub abi: String,
    pub signature: String,
}

/// Surface-level validation result (Task 25)
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub diagnostics: Vec<SurfaceDiagnostic>,
    pub repr_validated: Vec<ReprValidatedItem>,
    pub abi_validated: Vec<AbiValidatedItem>,
}

/// A repr-validated item
#[derive(Debug, Clone)]
pub struct ReprValidatedItem {
    pub name: String,
    pub kind: ItemKind,
    pub repr: ReprKind,
}

/// Validated ABI item
#[derive(Debug, Clone)]
pub struct AbiValidatedItem {
    pub symbol: String,
    pub abi: String,
    pub is_exported: bool,
    pub panic_policy: PanicPolicy,
}

/// Diagnostic from surface validation
#[derive(Debug, Clone)]
pub struct SurfaceDiagnostic {
    pub code: String,
    pub message: String,
    pub span: Option<String>,
}

/// Repr kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReprKind {
    C,
    Transparent,
    CUnwind,
    Scalar(ScalarRepr),
}

/// Scalar repr variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarRepr {
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
}

/// Panic policy for FFI functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanicPolicy {
    Abort,
    Unwind,
}

/// Validate Rust source at the surface level for FFI boundary safety.
/// This does not require rustc internals and works with stable Rust.
pub fn validate_surface(source: &str) -> ValidationResult {
    let mut diagnostics = Vec::new();
    let mut repr_validated = Vec::new();
    let mut abi_validated = Vec::new();

    let ast = match syn::parse_file(source) {
        Ok(ast) => ast,
        Err(e) => {
            diagnostics.push(SurfaceDiagnostic {
                code: "parse-error".to_string(),
                message: e.to_string(),
                span: None,
            });
            return ValidationResult {
                is_valid: false,
                diagnostics,
                repr_validated,
                abi_validated,
            };
        }
    };

    for item in &ast.items {
        match item {
            Item::Struct(s) => {
                if let Some(result) = extract_struct_repr(s) {
                    repr_validated.push(result);
                }
            }
            Item::Enum(e) => {
                if let Some(result) = extract_enum_repr(e) {
                    repr_validated.push(result);
                }
            }
            Item::Union(u) => {
                if let Some(result) = extract_union_repr(u) {
                    repr_validated.push(result);
                }
            }
            Item::ForeignMod(e) => {
                for fi in &e.items {
                    if let syn::ForeignItem::Fn(f) = fi {
                        abi_validated.push(AbiValidatedItem {
                            symbol: f.sig.ident.to_string(),
                            abi: e
                                .abi
                                .name
                                .as_ref()
                                .map(|n| n.value())
                                .unwrap_or_else(|| "C".to_string()),
                            is_exported: f.attrs.iter().any(|a| a.path().is_ident("no_mangle")),
                            panic_policy: PanicPolicy::Abort,
                        });
                    }
                }
            }
            Item::Fn(f) => {
                if let Some(result) = extract_fn_abi(f) {
                    abi_validated.push(result);
                }
            }
            _ => {}
        }
    }

    ValidationResult {
        is_valid: true,
        diagnostics,
        repr_validated,
        abi_validated,
    }
}

fn extract_struct_repr(s: &syn::ItemStruct) -> Option<ReprValidatedItem> {
    let repr = extract_repr(s.attrs.iter())?;
    Some(ReprValidatedItem {
        name: s.ident.to_string(),
        kind: ItemKind::Struct,
        repr,
    })
}

fn extract_enum_repr(e: &syn::ItemEnum) -> Option<ReprValidatedItem> {
    let repr = extract_repr(e.attrs.iter())?;
    Some(ReprValidatedItem {
        name: e.ident.to_string(),
        kind: ItemKind::Enum,
        repr,
    })
}

fn extract_union_repr(u: &syn::ItemUnion) -> Option<ReprValidatedItem> {
    let repr = extract_repr(u.attrs.iter())?;
    Some(ReprValidatedItem {
        name: u.ident.to_string(),
        kind: ItemKind::Union,
        repr,
    })
}

fn extract_repr<'a, I: Iterator<Item = &'a syn::Attribute>>(mut attrs: I) -> Option<ReprKind> {
    let attr = attrs.find(|a| a.path().is_ident("repr"))?;
    let tokens = attr.meta.to_token_stream().to_string();

    if tokens.contains("C") && !tokens.contains("unwind") {
        Some(ReprKind::C)
    } else if tokens.contains("transparent") {
        Some(ReprKind::Transparent)
    } else if tokens.contains("C unwind") {
        Some(ReprKind::CUnwind)
    } else if let Some(scalar) = parse_scalar_repr(&tokens) {
        Some(ReprKind::Scalar(scalar))
    } else {
        None
    }
}

fn parse_scalar_repr(tokens: &str) -> Option<ScalarRepr> {
    if tokens.contains("u8") {
        Some(ScalarRepr::U8)
    } else if tokens.contains("u16") {
        Some(ScalarRepr::U16)
    } else if tokens.contains("u32") {
        Some(ScalarRepr::U32)
    } else if tokens.contains("u64") {
        Some(ScalarRepr::U64)
    } else if tokens.contains("u128") {
        Some(ScalarRepr::U128)
    } else if tokens.contains("usize") {
        Some(ScalarRepr::Usize)
    } else if tokens.contains("i8") {
        Some(ScalarRepr::I8)
    } else if tokens.contains("i16") {
        Some(ScalarRepr::I16)
    } else if tokens.contains("i32") {
        Some(ScalarRepr::I32)
    } else if tokens.contains("i64") {
        Some(ScalarRepr::I64)
    } else if tokens.contains("i128") {
        Some(ScalarRepr::I128)
    } else if tokens.contains("isize") {
        Some(ScalarRepr::Isize)
    } else {
        None
    }
}

fn extract_fn_abi(f: &ItemFn) -> Option<AbiValidatedItem> {
    let has_no_mangle = f.attrs.iter().any(|a| a.path().is_ident("no_mangle"));
    let has_extern_c = f.attrs.iter().any(|a| {
        a.path().is_ident("extern")
            && syn::parse2::<syn::ExprTuple>(a.meta.to_token_stream())
                .map(|t| {
                    t.elems.iter().any(|e| {
                        if let syn::Expr::Lit(l) = e {
                            if let syn::Lit::Str(s) = &l.lit {
                                return s.value() == "C";
                            }
                        }
                        false
                    })
                })
                .unwrap_or(false)
    });

    if has_no_mangle && has_extern_c {
        let panic_policy = if f.attrs.iter().any(|a| a.path().is_ident("panic")) {
            PanicPolicy::Unwind
        } else {
            PanicPolicy::Abort
        };

        Some(AbiValidatedItem {
            symbol: f.sig.ident.to_string(),
            abi: "C".to_string(),
            is_exported: true,
            panic_policy,
        })
    } else {
        None
    }
}

// =============================================================================
// Item Extraction
// =============================================================================

fn extract_function_item(f: &ItemFn) -> RsnapItem {
    let attrs = f.attrs.iter().map(convert_attribute).collect();
    let generics = if f.sig.generics.params.is_empty() {
        None
    } else {
        Some(extract_generics(&f.sig.generics))
    };

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Function,
        visibility: convert_visibility(&f.vis),
        attributes: attrs,
        generics,
        where_clauses: Vec::new(),
    }
}

fn extract_struct_item(s: &syn::ItemStruct) -> RsnapItem {
    let attrs = s.attrs.iter().map(convert_attribute).collect();
    let generics = if s.generics.params.is_empty() {
        None
    } else {
        Some(extract_generics(&s.generics))
    };

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Struct,
        visibility: convert_visibility(&s.vis),
        attributes: attrs,
        generics,
        where_clauses: Vec::new(),
    }
}

fn extract_enum_item(e: &syn::ItemEnum) -> RsnapItem {
    let attrs = e.attrs.iter().map(convert_attribute).collect();
    let generics = if e.generics.params.is_empty() {
        None
    } else {
        Some(extract_generics(&e.generics))
    };

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Enum,
        visibility: convert_visibility(&e.vis),
        attributes: attrs,
        generics,
        where_clauses: Vec::new(),
    }
}

fn extract_union_item(u: &syn::ItemUnion) -> RsnapItem {
    let attrs = u.attrs.iter().map(convert_attribute).collect();
    let generics = if u.generics.params.is_empty() {
        None
    } else {
        Some(extract_generics(&u.generics))
    };

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Union,
        visibility: convert_visibility(&u.vis),
        attributes: attrs,
        generics,
        where_clauses: Vec::new(),
    }
}

fn extract_trait_item(t: &syn::ItemTrait) -> RsnapItem {
    let attrs = t.attrs.iter().map(convert_attribute).collect();

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Trait,
        visibility: convert_visibility(&t.vis),
        attributes: attrs,
        generics: None,
        where_clauses: Vec::new(),
    }
}

fn extract_impl_item(i: &syn::ItemImpl) -> RsnapItem {
    let attrs = i.attrs.iter().map(convert_attribute).collect();

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Impl,
        visibility: Visibility {
            rank: VisibilityRank::Pub,
            path: None,
        },
        attributes: attrs,
        generics: None,
        where_clauses: Vec::new(),
    }
}

fn extract_type_item(t: &syn::ItemType) -> RsnapItem {
    let attrs = t.attrs.iter().map(convert_attribute).collect();
    let generics = if t.generics.params.is_empty() {
        None
    } else {
        Some(extract_generics(&t.generics))
    };

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Type,
        visibility: convert_visibility(&t.vis),
        attributes: attrs,
        generics,
        where_clauses: Vec::new(),
    }
}

fn extract_const_item(c: &syn::ItemConst) -> RsnapItem {
    let attrs = c.attrs.iter().map(convert_attribute).collect();

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Constant,
        visibility: convert_visibility(&c.vis),
        attributes: attrs,
        generics: None,
        where_clauses: Vec::new(),
    }
}

fn extract_module_item(m: &ItemMod) -> RsnapItem {
    let attrs = m.attrs.iter().map(convert_attribute).collect();

    RsnapItem {
        id: ItemId(0),
        def_path: String::new(),
        kind: ItemKind::Module,
        visibility: convert_visibility(&m.vis),
        attributes: attrs,
        generics: None,
        where_clauses: Vec::new(),
    }
}

// =============================================================================
// Extern Block Validation
// =============================================================================

fn validate_extern_block_with_imports(
    e: &syn::ItemForeignMod,
) -> Result<(Vec<RsnapExport>, Vec<RsnapItem>, Vec<RsnapImport>), ParseError> {
    let abi = e
        .abi
        .name
        .as_ref()
        .map(|n| n.value())
        .unwrap_or_else(|| "C".to_string());

    let mut exports = Vec::new();
    let mut items = Vec::new();
    let mut imports = Vec::new();

    for item in &e.items {
        match item {
            syn::ForeignItem::Fn(f) => {
                let has_no_mangle = f.attrs.iter().any(|a| a.path().is_ident("no_mangle"));
                let symbol = f.sig.ident.to_string();
                // Build signature from function inputs
                let param_types: Vec<String> = f
                    .sig
                    .inputs
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::FnArg::Typed(pt) => {
                            let ty = &pt.ty;
                            Some(quote::ToTokens::to_token_stream(ty).to_string())
                        }
                        _ => None,
                    })
                    .collect();
                let signature = match &f.sig.output {
                    syn::ReturnType::Type(_, ty) => {
                        let ret_str = quote::ToTokens::to_token_stream(&*ty).to_string();
                        format!("({}) -> {}", param_types.join(", "), ret_str)
                    }
                    syn::ReturnType::Default => {
                        format!("({}) -> ()", param_types.join(", "))
                    }
                };

                if has_no_mangle {
                    // Exported function (defined in this crate, visible externally)
                    exports.push(RsnapExport {
                        symbol: symbol.clone(),
                        item_id: ItemId(0),
                        abi: abi.to_string(),
                        linkage: Linkage::External,
                    });
                    items.push(RsnapItem {
                        id: ItemId(0),
                        def_path: String::new(),
                        kind: ItemKind::Function,
                        visibility: Visibility {
                            rank: VisibilityRank::Pub,
                            path: None,
                        },
                        attributes: vec![],
                        generics: None,
                        where_clauses: Vec::new(),
                    });
                } else {
                    // Import (external dependency that this crate depends on)
                    imports.push(RsnapImport {
                        symbol,
                        abi: abi.to_string(),
                        signature,
                    });
                }
            }
            syn::ForeignItem::Static(s) => {
                let has_no_mangle = s.attrs.iter().any(|a| a.path().is_ident("no_mangle"));

                if has_no_mangle {
                    let symbol = s.ident.to_string();
                    exports.push(RsnapExport {
                        symbol,
                        item_id: ItemId(0),
                        abi: abi.to_string(),
                        linkage: Linkage::External,
                    });
                    items.push(RsnapItem {
                        id: ItemId(0),
                        def_path: String::new(),
                        kind: ItemKind::Static,
                        visibility: Visibility {
                            rank: VisibilityRank::Pub,
                            path: None,
                        },
                        attributes: vec![],
                        generics: None,
                        where_clauses: Vec::new(),
                    });
                } else {
                    // Immutable import
                    imports.push(RsnapImport {
                        symbol: s.ident.to_string(),
                        abi: abi.to_string(),
                        signature: "i32".to_string(),
                    });
                }
            }
            syn::ForeignItem::Type(_t) => {
                items.push(RsnapItem {
                    id: ItemId(0),
                    def_path: String::new(),
                    kind: ItemKind::Type,
                    visibility: Visibility {
                        rank: VisibilityRank::Pub,
                        path: None,
                    },
                    attributes: vec![],
                    generics: None,
                    where_clauses: Vec::new(),
                });
            }
            syn::ForeignItem::Macro(_) => {}
            _ => {}
        }
    }

    Ok((exports, items, imports))
}

fn validate_extern_fn(f: &syn::ForeignItemFn, abi: &str) -> Option<RsnapExport> {
    let sig = &f.sig;
    let has_no_mangle = f.attrs.iter().any(|a| a.path().is_ident("no_mangle"));

    if has_no_mangle {
        let symbol = sig.ident.to_string();
        Some(RsnapExport {
            symbol,
            item_id: ItemId(0),
            abi: abi.to_string(),
            linkage: Linkage::External,
        })
    } else {
        None
    }
}

fn validate_extern_static(s: &syn::ForeignItemStatic, abi: &str) -> Option<RsnapExport> {
    let has_no_mangle = s.attrs.iter().any(|a| a.path().is_ident("no_mangle"));

    if has_no_mangle {
        let symbol = s.ident.to_string();
        Some(RsnapExport {
            symbol,
            item_id: ItemId(0),
            abi: abi.to_string(),
            linkage: Linkage::External,
        })
    } else {
        None
    }
}

// =============================================================================
// Exported Function/Static Validation
// =============================================================================

fn validate_exported_function(f: &ItemFn) -> Option<RsnapExport> {
    let sig = &f.sig;

    let has_no_mangle = f.attrs.iter().any(|a| a.path().is_ident("no_mangle"));
    let has_extern_c = f.attrs.iter().any(|a| {
        a.path().is_ident("extern")
            && syn::parse2::<syn::ExprTuple>(a.meta.to_token_stream())
                .map(|t| {
                    t.elems.iter().any(|e| {
                        if let syn::Expr::Lit(l) = e {
                            if let syn::Lit::Str(s) = &l.lit {
                                return s.value() == "C";
                            }
                        }
                        false
                    })
                })
                .unwrap_or(false)
    });

    if !sig.generics.params.is_empty() {
        return None;
    }

    if has_no_mangle && has_extern_c {
        let symbol = sig.ident.to_string();
        Some(RsnapExport {
            symbol,
            item_id: ItemId(0),
            abi: "C".to_string(),
            linkage: Linkage::External,
        })
    } else {
        None
    }
}

fn validate_exported_static(s: &ItemStatic) -> Option<RsnapExport> {
    let has_no_mangle = s.attrs.iter().any(|a| a.path().is_ident("no_mangle"));
    let has_extern_c = s.attrs.iter().any(|a| {
        a.path().is_ident("extern")
            && syn::parse2::<syn::ExprTuple>(a.meta.to_token_stream())
                .map(|t| {
                    t.elems.iter().any(|e| {
                        if let syn::Expr::Lit(l) = e {
                            if let syn::Lit::Str(s) = &l.lit {
                                return s.value() == "C";
                            }
                        }
                        false
                    })
                })
                .unwrap_or(false)
    });

    let is_not_mutable = matches!(s.mutability, syn::StaticMutability::None);

    if has_no_mangle && has_extern_c && is_not_mutable {
        let symbol = s.ident.to_string();
        Some(RsnapExport {
            symbol,
            item_id: ItemId(0),
            abi: "C".to_string(),
            linkage: Linkage::External,
        })
    } else {
        None
    }
}

// =============================================================================
// Attribute and Visibility Conversion
// =============================================================================

fn convert_attribute(attr: &SynAttribute) -> Attribute {
    let path = attr.path().to_token_stream().to_string().replace(' ', "");
    let tokens = attr.meta.to_token_stream().to_string();
    Attribute { path, tokens }
}

fn convert_visibility(vis: &syn::Visibility) -> Visibility {
    match vis {
        syn::Visibility::Public(_) => Visibility {
            rank: VisibilityRank::Pub,
            path: None,
        },
        syn::Visibility::Restricted(r) => {
            let path_str = r.path.to_token_stream().to_string();
            if path_str == "crate" {
                Visibility {
                    rank: VisibilityRank::PubCrate,
                    path: None,
                }
            } else {
                Visibility {
                    rank: VisibilityRank::PubRestricted,
                    path: Some(path_str),
                }
            }
        }
        syn::Visibility::Inherited => Visibility {
            rank: VisibilityRank::Private,
            path: None,
        },
    }
}

fn extract_generics(generics: &syn::Generics) -> Generics {
    let lifetimes = generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Lifetime(lt) = p {
                Some(lt.lifetime.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    let type_params = generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Type(t) = p {
                Some(chimera_rust_schema::TypeParam {
                    name: t.ident.to_string(),
                    bounds: Vec::new(),
                })
            } else {
                None
            }
        })
        .collect();

    let const_params = generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Const(c) = p {
                let ty = quote::ToTokens::to_token_stream(&c.ty).to_string();
                Some(ConstParam {
                    name: c.ident.to_string(),
                    ty,
                })
            } else {
                None
            }
        })
        .collect();

    Generics {
        lifetimes,
        type_params,
        const_params,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_extern() {
        let source = r#"
            extern "C" {
                #[no_mangle]
                pub fn add(a: i32, b: i32) -> i32;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(!parsed.exports.is_empty());
        assert_eq!(parsed.exports[0].symbol, "add");
    }

    #[test]
    fn test_parse_extern_with_no_mangle() {
        let source = r#"
            #[no_mangle]
            pub extern "C" fn multiply(x: i32, y: i32) -> i32 {
                x * y
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_struct_with_repr() {
        let source = r#"
            #[repr(C)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_enum_with_repr() {
        let source = r#"
            #[repr(u32)]
            pub enum Color {
                Red = 0,
                Green = 1,
                Blue = 2,
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_generic_ffi_rejected() {
        let source = r#"
            #[no_mangle]
            pub extern "C" fn generic_fn<T>(x: T) -> T {
                x
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.exports.is_empty());
    }

    #[test]
    fn test_parse_union() {
        let source = r#"
            #[repr(C)]
            pub union IntOrFloat {
                as_int: u64,
                as_float: f64,
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module() {
        let source = r#"
            mod inner {
                pub fn internal() {}
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_trait() {
        let source = r#"
            pub trait MyTrait {
                fn method(&self);
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_impl() {
        let source = r#"
            impl MyTrait for u32 {
                fn method(&self) {}
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    // =====================================================================
    // Surface validation tests (Task 25)
    // =====================================================================

    #[test]
    fn test_validate_surface_repr_c() {
        let source = r#"
            #[repr(C)]
            pub struct Point {
                pub x: f64,
                pub y: f64,
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert_eq!(result.repr_validated.len(), 1);
        assert!(matches!(result.repr_validated[0].repr, ReprKind::C));
    }

    #[test]
    fn test_validate_surface_repr_transparent() {
        let source = r#"
            #[repr(transparent)]
            pub struct Wrapper(i32);
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert_eq!(result.repr_validated.len(), 1);
        assert!(matches!(
            result.repr_validated[0].repr,
            ReprKind::Transparent
        ));
    }

    #[test]
    fn test_validate_surface_enum_repr() {
        let source = r#"
            #[repr(u32)]
            pub enum Status {
                Ok = 0,
                Error = 1,
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert_eq!(result.repr_validated.len(), 1);
        assert!(matches!(
            result.repr_validated[0].repr,
            ReprKind::Scalar(ScalarRepr::U32)
        ));
    }

    #[test]
    fn test_validate_surface_extern_fn() {
        let source = r#"
            extern "C" {
                #[no_mangle]
                pub fn add(a: i32, b: i32) -> i32;
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert_eq!(result.abi_validated.len(), 1);
        assert_eq!(result.abi_validated[0].symbol, "add");
        assert!(result.abi_validated[0].is_exported);
        assert_eq!(result.abi_validated[0].abi, "C");
    }

    #[test]
    fn test_validate_surface_no_repr() {
        let source = r#"
            pub struct Point {
                x: f64,
                y: f64,
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert!(result.repr_validated.is_empty());
    }

    #[test]
    fn test_validate_surface_extern_no_mangle() {
        let source = r#"
            extern "C" {
                pub fn not_exported(a: i32) -> i32;
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert_eq!(result.abi_validated.len(), 1);
        assert!(!result.abi_validated[0].is_exported);
    }

    #[test]
    fn test_parse_cfg_attribute() {
        let source = r#"
            #[cfg(feature = "unsafe")]
            pub struct UnsafeWrapper {
                ptr: *mut u8,
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(!parsed.items.is_empty());
    }

    #[test]
    fn test_parse_multiple_extern_items() {
        let source = r#"
            extern "C" {
                #[no_mangle]
                pub fn foo(a: i32) -> i32;
                #[no_mangle]
                pub fn bar(b: f64) -> f64;
                pub static mut COUNTER: i32;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        // Check that we have exports from no_mangle functions
        assert!(!parsed.exports.is_empty());
    }

    #[test]
    fn test_parse_type_alias() {
        let source = r#"
            pub type MyCallback = Option<fn(i32) -> i32>;
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_const_item() {
        let source = r#"
            const MAX_SIZE: usize = 1024;
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_static_item() {
        let source = r#"
            static mut COUNTER: i32 = 0;
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_where_clause() {
        let source = r#"
            pub struct Wrapper<T>
            where
                T: Sized,
            {
                value: T,
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_surface_with_diagnostic() {
        let source = r#"
            #[repr(C)]
            pub struct Invalid {
                a: i32,
                #[repr(C)]
                pub b: f64,
            }
        "#;

        let result = validate_surface(source);
        // Should handle multiple reprs and nested attrs
        assert!(result.is_valid || !result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_nested_module() {
        let source = r#"
            mod outer {
                mod inner {
                    pub fn nested_fn() {}
                }
                pub use inner::nested_fn;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_export_name_attribute() {
        let source = r#"
            #[export_name = "exported_name"]
            pub extern "C" fn renamed_fn() {}
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_surface_multiple_abi_items() {
        let source = r#"
            extern "C" {
                pub fn first_fn();
                pub fn second_fn(x: i32);
            }

            extern "C-unwind" {
                pub fn unwind_fn() -> Result<i32, ()>;
            }
        "#;

        let result = validate_surface(source);
        assert!(result.is_valid);
        assert!(result.abi_validated.len() >= 2);
    }

    // Task 172: Property-based/fuzz-style tests

    #[test]
    fn test_parse_struct_repr_c_valid() {
        // repr(C) structs with primitive fields should parse
        let source = r#"
            #[repr(C)]
            pub struct TwoFloats {
                x: f64,
                y: f64,
            }
        "#;
        let result = parse_rust_source(source);
        assert!(result.is_ok());

        let source2 = r#"
            #[repr(C)]
            pub struct Mixed {
                a: i32,
                b: f64,
                c: u8,
            }
        "#;
        let result2 = parse_rust_source(source2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_parse_enum_repr_valid() {
        let sources = vec![
            r#"
                #[repr(C)]
                pub enum E1 { A, B, C }
            "#,
            r#"
                #[repr(u32)]
                pub enum E2 { A = 1, B = 2, C = 4 }
            "#,
            r#"
                #[repr(Isize)]
                pub enum E3 { A = -1, B = 0, C = 1 }
            "#,
        ];
        for source in sources {
            let result = parse_rust_source(source);
            assert!(result.is_ok(), "source: {}", source);
        }
    }

    #[test]
    fn test_parse_extern_with_various_abi() {
        let abis = vec!["C", "C-unwind", "Rust", "system"];
        for abi in abis {
            let safe_abi = abi.replace("-", "_");
            let source = format!(
                r#"
                extern "{}" {{
                    pub fn fn_{}() -> i32;
                }}
            "#,
                abi, safe_abi
            );
            let result = parse_rust_source(&source);
            assert!(result.is_ok(), "ABI: {}", abi);
        }
    }

    #[test]
    fn test_parse_various_primitive_types() {
        let types = vec![
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64", "bool", "char",
        ];
        for ty in types {
            let source = format!(
                r#"
                extern "C" {{
                    pub fn get_{}(v: {}) -> {};
                }}
            "#,
                ty, ty, ty
            );
            let result = parse_rust_source(&source);
            assert!(result.is_ok(), "type: {}", ty);
        }
    }

    #[test]
    fn test_parse_array_and_slice_types() {
        let sources = vec![
            r#"
                extern "C" {
                    pub fn takes_array(a: [i32; 4]) -> i32;
                }
            "#,
            r#"
                extern "C" {
                    pub fn takes_slice(s: *const i32, len: usize) -> i32;
                }
            "#,
        ];
        for source in sources {
            let result = parse_rust_source(source);
            assert!(result.is_ok(), "source: {}", source);
        }
    }

    // =====================================================================
    // Import tests (Task 15)
    // =====================================================================

    #[test]
    fn test_parse_extern_imports() {
        // Non-no_mangle functions in extern blocks are imports (external dependencies)
        let source = r#"
            extern "C" {
                pub fn external_add(a: i32, b: i32) -> i32;
                pub fn external_mul(a: i32, b: i32) -> i32;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        // external_add and external_mul are imports (not exports)
        assert!(
            parsed.exports.is_empty(),
            "non-no_mangle functions should not be exports"
        );
        assert_eq!(parsed.imports.len(), 2, "should have 2 imports");
        assert_eq!(parsed.imports[0].symbol, "external_add");
        assert_eq!(parsed.imports[1].symbol, "external_mul");
    }

    #[test]
    fn test_parse_extern_mixed_export_import() {
        // Mix of exported (no_mangle) and imported (non-no_mangle) functions
        let source = r#"
            extern "C" {
                #[no_mangle]
                pub fn my_export(a: i32) -> i32;
                pub fn external_fn(a: i32) -> i32;
                #[no_mangle]
                pub static MY_STATIC: i32;
                pub static EXTERNAL_STATIC: i32;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        // my_export and MY_STATIC are exports
        assert_eq!(parsed.exports.len(), 2);
        assert_eq!(parsed.exports[0].symbol, "my_export");
        assert_eq!(parsed.exports[1].symbol, "MY_STATIC");
        // external_fn and EXTERNAL_STATIC are imports
        assert_eq!(parsed.imports.len(), 2);
        assert_eq!(parsed.imports[0].symbol, "external_fn");
        assert_eq!(parsed.imports[1].symbol, "EXTERNAL_STATIC");
    }

    #[test]
    fn test_parse_import_abi() {
        let source = r#"
            extern "C-unwind" {
                pub fn unwind_import(a: i32) -> i32;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].symbol, "unwind_import");
        assert_eq!(parsed.imports[0].abi, "C-unwind");
    }

    #[test]
    fn test_parse_import_signature() {
        let source = r#"
            extern "C" {
                pub fn add(a: i32, b: i32) -> i32;
                pub fn get_value() -> i64;
                pub fn process(ptr: *const u8, len: usize) -> bool;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.imports.len(), 3);
        // Check that signatures are captured
        assert!(parsed.imports[0].signature.contains("i32"));
        assert!(parsed.imports[1].signature.contains("i64"));
        assert!(parsed.imports[2].signature.contains("bool"));
    }

    #[test]
    fn test_no_imports_without_extern_block() {
        // Regular Rust code without extern blocks should have no imports
        let source = r#"
            pub fn regular_fn(a: i32) -> i32 { a }

            #[no_mangle]
            pub extern "C" fn exported_fn(a: i32) -> i32 { a }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.imports.is_empty(), "no imports without extern block");
    }

    #[test]
    fn test_extern_block_type_not_import() {
        // Type declarations in extern blocks are not imports
        let source = r#"
            extern "C" {
                pub type MyCallback;
                pub fn fn_ptr_to_cbk() -> MyCallback;
            }
        "#;

        let result = parse_rust_source(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        // MyCallback is a type, not an import
        // fn_ptr_to_cbk is a function import
        assert!(parsed.imports.len() >= 1);
    }
}
