//! Chimera Rust Adapter
//!
//! Validates Rust FFI boundaries: repr(C), extern blocks, no native types
//! crossing boundaries, and proper result/panic handling.
//!
//! # Safety
//!
//! This adapter validates unsafe Rust FFI code and ensures ABI safety.

use chimera_diagnostics::{Code, DiagnosticBag};
use chimera_meta::LayoutMetadata;
use quote::ToTokens;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Error domain for Rust adapter errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorDomain {
    None,
    Type,
    Ownership,
    Validation,
    Safety,
}

/// Rust adapter errors
#[derive(Debug, Clone)]
pub enum AdapterError {
    InvalidRepr(String),
    UnsupportedType(String),
    NativeTypeCrossesBoundary(String),
    MissingExternBlock(String),
    PanicPolicyViolation(String),
    ParseError(String),
}

/// Types that are NOT allowed to cross FFI boundaries in Rust
const FORBIDDEN_TYPES: &[&str] = &[
    "Vec", "String", "Box", "Rc", "Arc", "Cell", "RefCell", "Option", "Result", "HashMap",
    "HashSet", "BTreeMap", "BTreeSet",
];

/// Rust item representation for FFI validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RustItem {
    Struct {
        name: String,
        repr: String,
        fields: Vec<RustField>,
    },
    Enum {
        name: String,
        repr: String,
    },
    Union {
        name: String,
    },
    Function {
        name: String,
        sig: RustSignature,
    },
    ExternBlock {
        abi: String,
        items: Vec<RustItem>,
    },
    TypeAlias {
        name: String,
        typ: String,
    },
}

/// Rust field representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustField {
    pub name: String,
    pub typ: String,
}

/// Rust function signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustSignature {
    pub abi: String,
    pub params: Vec<String>,
    pub ret: Option<String>,
    pub panic_policy: PanicPolicy,
}

/// Panic policy for FFI functions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicPolicy {
    Abort,
    Catch,
    Unwind,
}

/// Parse a Rust source file and extract FFI items
pub fn parse_rust_source(source: &str) -> Result<Vec<RustItem>, AdapterError> {
    match syn::parse_file(source) {
        Ok(file) => Ok(file.items.into_iter().filter_map(parse_item).collect()),
        Err(e) => Err(AdapterError::ParseError(e.to_string())),
    }
}

fn parse_item(item: syn::Item) -> Option<RustItem> {
    match item {
        syn::Item::Struct(s) => {
            let repr = extract_repr_attr(&s.attrs, "repr");
            let fields = s
                .fields
                .iter()
                .filter_map(|f| {
                    let name = f.ident.as_ref()?.to_string();
                    let typ = type_to_string(&f.ty);
                    Some(RustField { name, typ })
                })
                .collect();
            Some(RustItem::Struct {
                name: s.ident.to_string(),
                repr,
                fields,
            })
        }
        syn::Item::Enum(e) => {
            let repr = extract_repr_attr(&e.attrs, "repr");
            Some(RustItem::Enum {
                name: e.ident.to_string(),
                repr,
            })
        }
        syn::Item::Union(u) => Some(RustItem::Union {
            name: u.ident.to_string(),
        }),
        syn::Item::Fn(f) => {
            let abi = extract_extern_attr(&f.attrs).unwrap_or_else(|| "C".to_string());
            let sig = RustSignature {
                abi,
                params: f
                    .sig
                    .inputs
                    .iter()
                    .map(|p| match p {
                        syn::FnArg::Typed(t) => type_to_string(&t.ty),
                        syn::FnArg::Receiver(_) => "self".to_string(),
                    })
                    .collect(),
                ret: Some(return_type_to_string(&f.sig.output)),
                panic_policy: extract_panic_policy(&f.attrs),
            };
            Some(RustItem::Function {
                name: f.sig.ident.to_string(),
                sig,
            })
        }
        syn::Item::ForeignMod(m) => {
            let abi = m
                .abi
                .name
                .as_ref()
                .map(|s| s.value())
                .unwrap_or_else(|| "C".to_string());
            let items: Vec<RustItem> = m
                .items
                .iter()
                .filter_map(|i| match i {
                    syn::ForeignItem::Fn(f) => {
                        let sig = RustSignature {
                            abi: abi.clone(),
                            params: f
                                .sig
                                .inputs
                                .iter()
                                .map(|p| match p {
                                    syn::FnArg::Typed(t) => type_to_string(&t.ty),
                                    syn::FnArg::Receiver(_) => "self".to_string(),
                                })
                                .collect(),
                            ret: Some(return_type_to_string(&f.sig.output)),
                            panic_policy: PanicPolicy::Abort,
                        };
                        Some(RustItem::Function {
                            name: f.sig.ident.to_string(),
                            sig,
                        })
                    }
                    syn::ForeignItem::Static(s) => Some(RustItem::TypeAlias {
                        name: s.ident.to_string(),
                        typ: type_to_string(&s.ty),
                    }),
                    syn::ForeignItem::Type(t) => Some(RustItem::TypeAlias {
                        name: t.ident.to_string(),
                        typ: "type".to_string(),
                    }),
                    _ => None,
                })
                .collect();
            Some(RustItem::ExternBlock { abi, items })
        }
        syn::Item::Type(t) => Some(RustItem::TypeAlias {
            name: t.ident.to_string(),
            typ: type_to_string(&t.ty),
        }),
        _ => None,
    }
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .iter()
            .map(|s| match &s.arguments {
                syn::PathArguments::None => s.ident.to_string(),
                syn::PathArguments::AngleBracketed(ab) => {
                    let args = ab
                        .args
                        .iter()
                        .map(|a| match a {
                            syn::GenericArgument::Type(t) => type_to_string(t),
                            syn::GenericArgument::Lifetime(lt) => lt.to_string(),
                            _ => "_".to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", s.ident, args)
                }
                syn::PathArguments::Parenthesized(_) => "fn(..)".to_string(),
            })
            .collect::<Vec<_>>()
            .join("::"),
        syn::Type::Reference(r) => {
            let mutability = if r.mutability.is_some() { "mut " } else { "" };
            format!("&{}{}", mutability, type_to_string(&r.elem))
        }
        syn::Type::Ptr(p) => {
            let mutability = if p.mutability.is_some() {
                "mut "
            } else {
                "const "
            };
            format!("*{}{}", mutability, type_to_string(&p.elem))
        }
        syn::Type::Array(a) => format!("[{}; N]", type_to_string(&a.elem)),
        syn::Type::Slice(s) => format!("[{}]", type_to_string(&s.elem)),
        syn::Type::Tuple(t) if t.elems.is_empty() => "()".to_string(),
        syn::Type::Tuple(t) => {
            let fields = t
                .elems
                .iter()
                .map(type_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", fields)
        }
        syn::Type::Never(_) => "!".to_string(),
        _ => "_".to_string(),
    }
}

fn return_type_to_string(ret: &syn::ReturnType) -> String {
    match ret {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => type_to_string(ty),
    }
}

fn extract_repr_attr(attrs: &[syn::Attribute], name: &str) -> String {
    attrs
        .iter()
        .find(|a| a.path().is_ident(name))
        .map(|a| {
            let tokens = a.meta.to_token_stream().to_string();
            tokens
                .trim_start_matches(&format!("{} ", name))
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string()
        })
        .unwrap_or_default()
}

fn extract_extern_attr(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|a| a.path().is_ident("extern"))
        .map(|a| a.meta.to_token_stream().to_string())
}

fn extract_panic_policy(attrs: &[syn::Attribute]) -> PanicPolicy {
    attrs
        .iter()
        .find(|a| a.path().is_ident("panic"))
        .and_then(|a| {
            let tokens = a.meta.to_token_stream().to_string();
            if tokens.contains("unwind") {
                Some(PanicPolicy::Unwind)
            } else {
                None
            }
        })
        .unwrap_or(PanicPolicy::Abort)
}

/// Rust adapter for validating FFI boundaries
pub struct RustAdapter {
    diagnostics: DiagnosticBag,
    valid_reprs: HashSet<String>,
    #[allow(dead_code)]
    extern_blocks: Vec<RustItem>,
    #[allow(dead_code)]
    structs: Vec<RustItem>,
}

impl RustAdapter {
    pub fn new() -> Self {
        let mut valid_reprs = HashSet::new();
        valid_reprs.insert("C".to_string());
        valid_reprs.insert("C unwind".to_string());
        Self {
            diagnostics: DiagnosticBag::new(),
            valid_reprs,
            extern_blocks: Vec::new(),
            structs: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    pub fn parse_source(&mut self, source: &str) -> Result<Vec<RustItem>, AdapterError> {
        let items = parse_rust_source(source)?;
        for item in &items {
            self.validate_item(item);
        }
        Ok(items)
    }

    fn validate_item(&mut self, item: &RustItem) {
        match item {
            RustItem::Struct { name, repr, fields } => {
                if !repr.is_empty() && !self.validate_struct_repr(name, repr) {}
                for field in fields {
                    if !self.validate_type_not_foreign(&field.typ) {}
                }
            }
            RustItem::Function { sig, .. } => {
                for param in &sig.params {
                    if !self.validate_type_not_foreign(param) {}
                }
                if let Some(ret) = &sig.ret {
                    if !self.validate_type_not_foreign(ret) {}
                }
            }
            _ => {}
        }
    }

    pub fn validate_struct_repr(&mut self, name: &str, repr: &str) -> bool {
        if repr.is_empty() {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!("struct {} has no repr, FFI requires #[repr(C)]", name),
            );
            return false;
        }
        // Accept both "C unwind" and "Cunwind" (cleaned)
        let repr_clean = repr.replace(['(', ')', ' '], "");
        let repr_normalized = repr.replace(" unwind", "").trim().to_string();
        if !self.valid_reprs.contains(&repr_clean)
            && !self.valid_reprs.contains(&repr_normalized)
            && !self.valid_reprs.contains(repr)
        {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!("struct {} has invalid repr({})", name, repr),
            );
            return false;
        }
        true
    }

    pub fn validate_type_not_foreign(&mut self, type_name: &str) -> bool {
        let clean = type_name.replace(' ', "");
        for forbidden in FORBIDDEN_TYPES {
            // Check if forbidden type appears as a word boundary match
            if clean.contains(forbidden)
                && (clean.starts_with(forbidden)
                    || clean.contains(&format!(" {}", forbidden))
                    || clean.contains(&format!("<{}", forbidden)))
            {
                self.diagnostics.error(
                    Code::TypeMismatch,
                    &format!(
                        "type {} contains forbidden FFI type {}",
                        type_name, forbidden
                    ),
                );
                return false;
            }
        }
        true
    }

    pub fn validate_layout(
        &mut self,
        rust_layout: &RustStructLayout,
        expected: &LayoutMetadata,
    ) -> bool {
        let mut valid = true;
        if rust_layout.size != expected.size {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has size {} but expected {}",
                    rust_layout.name, rust_layout.size, expected.size
                ),
            );
            valid = false;
        }
        if rust_layout.align != expected.align {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has alignment {} but expected {}",
                    rust_layout.name, rust_layout.align, expected.align
                ),
            );
            valid = false;
        }
        for expected_field in &expected.fields {
            if let Some(rust_field) = rust_layout
                .fields
                .iter()
                .find(|f| f.name == expected_field.name)
            {
                if rust_field.offset != expected_field.offset {
                    self.diagnostics.error(
                        Code::TypeMismatch,
                        &format!(
                            "field {} in {} has offset {} but expected {}",
                            expected_field.name,
                            rust_layout.name,
                            rust_field.offset,
                            expected_field.offset
                        ),
                    );
                    valid = false;
                }
            }
        }
        valid
    }

    pub fn map_panic_policy(policy: PanicPolicy) -> ErrorDomain {
        match policy {
            PanicPolicy::Abort => ErrorDomain::Validation,
            PanicPolicy::Catch => ErrorDomain::Ownership,
            PanicPolicy::Unwind => ErrorDomain::Safety,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustFieldLayout {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustStructLayout {
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub fields: Vec<RustFieldLayout>,
}

impl Default for RustAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::InvalidRepr(s) => write!(f, "Invalid repr: {}", s),
            AdapterError::UnsupportedType(s) => write!(f, "Unsupported type: {}", s),
            AdapterError::NativeTypeCrossesBoundary(s) => {
                write!(f, "Native type crosses boundary: {}", s)
            }
            AdapterError::MissingExternBlock(s) => write!(f, "Missing extern block: {}", s),
            AdapterError::PanicPolicyViolation(s) => write!(f, "Panic policy violation: {}", s),
            AdapterError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for AdapterError {}

impl fmt::Display for ErrorDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorDomain::None => write!(f, "none"),
            ErrorDomain::Type => write!(f, "type"),
            ErrorDomain::Ownership => write!(f, "ownership"),
            ErrorDomain::Validation => write!(f, "validation"),
            ErrorDomain::Safety => write!(f, "safety"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = RustAdapter::new();
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_valid_c_repr() {
        let mut adapter = RustAdapter::new();
        assert!(adapter.validate_struct_repr("Test", "C"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_invalid_repr() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_struct_repr("Test", "Rust"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_forbidden_type_vec() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Vec<u8>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_safe_type() {
        let mut adapter = RustAdapter::new();
        assert!(adapter.validate_type_not_foreign("u32"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_validate_layout_success() {
        let mut adapter = RustAdapter::new();
        let rust_layout = RustStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        assert!(adapter.validate_layout(&rust_layout, &expected));
    }

    #[test]
    fn test_validate_layout_size_mismatch() {
        let mut adapter = RustAdapter::new();
        let rust_layout = RustStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        assert!(!adapter.validate_layout(&rust_layout, &expected));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_panic_policy_mapping() {
        assert_eq!(
            RustAdapter::map_panic_policy(PanicPolicy::Abort),
            ErrorDomain::Validation
        );
        assert_eq!(
            RustAdapter::map_panic_policy(PanicPolicy::Unwind),
            ErrorDomain::Safety
        );
    }

    #[test]
    fn test_parse_rust_source_struct() {
        let source = r#"
            #[repr(C)]
            struct Point {
                x: i32,
                y: i32,
            }
        "#;
        let items = parse_rust_source(source).unwrap();
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], RustItem::Struct { .. }));
    }

    #[test]
    fn test_parse_rust_source_fn() {
        let source = r#"
            #[no_mangle]
            pub extern "C" fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        "#;
        let items = parse_rust_source(source).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_parse_rust_source_extern_block() {
        let source = r#"
            extern "C" {
                fn puts(s: *const u8) -> i32;
            }
        "#;
        let items = parse_rust_source(source).unwrap();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_validate_forbidden_result() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Result<T, E>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_validate_forbidden_string() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("String"));
        assert!(adapter.has_errors());
    }

    // C.61: Tests for every disallowed native type

    #[test]
    fn test_c61_forbidden_box() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Box<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_rc() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Rc<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_arc() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Arc<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_cell() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Cell<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_refcell() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("RefCell<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_option() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("Option<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_hashmap() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("HashMap<K, V>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_hashset() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("HashSet<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_btreemap() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("BTreeMap<K, V>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_forbidden_btreeset() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_type_not_foreign("BTreeSet<T>"));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_allowed_types() {
        let mut adapter = RustAdapter::new();
        // These should all be allowed
        assert!(adapter.validate_type_not_foreign("i32"));
        assert!(adapter.validate_type_not_foreign("u64"));
        assert!(adapter.validate_type_not_foreign("f32"));
        assert!(adapter.validate_type_not_foreign("f64"));
        assert!(adapter.validate_type_not_foreign("*const u8"));
        assert!(adapter.validate_type_not_foreign("*mut u8"));
        assert!(adapter.validate_type_not_foreign("bool"));
        assert!(adapter.validate_type_not_foreign("u8"));
        assert!(adapter.validate_type_not_foreign("u16"));
        assert!(adapter.validate_type_not_foreign("u32"));
        assert!(adapter.validate_type_not_foreign("u128"));
        assert!(adapter.validate_type_not_foreign("i8"));
        assert!(adapter.validate_type_not_foreign("i16"));
        assert!(adapter.validate_type_not_foreign("i32"));
        assert!(adapter.validate_type_not_foreign("i64"));
        assert!(adapter.validate_type_not_foreign("i128"));
        assert!(adapter.validate_type_not_foreign("usize"));
        assert!(adapter.validate_type_not_foreign("isize"));
        assert!(adapter.validate_type_not_foreign("char"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_c61_repr_required() {
        let mut adapter = RustAdapter::new();
        assert!(!adapter.validate_struct_repr("TestStruct", ""));
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_c61_repr_c_unwind() {
        let mut adapter = RustAdapter::new();
        assert!(adapter.validate_struct_repr("TestStruct", "C unwind"));
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_c61_extern_block_with_no_mangle() {
        // Functions with both extern and #[no_mangle] ARE FFI-safe
        let source = r#"
            #[no_mangle]
            pub extern "C" fn bar() -> i32 { 42 }
        "#;
        let items = parse_rust_source(source).unwrap();
        // This function has both extern and #[no_mangle], so it's C FFI
        let is_ffi = items.iter().any(|item| match item {
            RustItem::Function { sig, .. } => sig.abi.contains("C"),
            _ => false,
        });
        assert!(is_ffi, "Function with extern should be FFI");
    }
}
