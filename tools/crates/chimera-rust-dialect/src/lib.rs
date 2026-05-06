//! Rust semantic dialect for type/ownership/lifetime modeling.
//!
//! This crate models Rust's semantic concepts before lowering to ChimeraIR.

use chimera_rust_mir_import::{
    NormalizedBorrowKind, NormalizedMirBody, NormalizedPrimitiveType, NormalizedTypeDef,
};
use chimera_rust_schema::ItemId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod call_lowering;
pub mod cfg;
pub mod diagnostics;
pub mod drop_lowering;

pub use call_lowering::{lower_call, lower_intrinsic, CallKind, CallOp};
pub use cfg::{BasicBlock, BlockTerminator, BranchTarget, ControlFlowGraph, SwitchCase};
pub use diagnostics::{DiagnosticKind, DialectDiagnostic, FeatureMatrix, UnsupportedFeature};
pub use drop_lowering::{lower_cleanup_path, lower_drop, DropGlue, DropOp, DropOrder};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DialectContext {
    pub items: HashMap<ItemId, ItemDialect>,
    pub type_cache: HashMap<ItemId, TypeDialect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDialect {
    pub item_id: ItemId,
    pub name: String,
    pub kind: ItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    Struct(StructDialect),
    Enum(EnumDialect),
    Union(UnionDialect),
    Fn(FnDialect),
    Const,
    Static(StaticDialect),
    TypeAlias,
    Trait,
    Impl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrinsicKind {
    Sized,
    Unsized,
    Copy,
    Sync,
    Send,
    Drop,
    Fn,
    Pointer,
    Slice,
    Str,
    Trait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDialect {
    pub fields: Vec<FieldDialect>,
    pub repr: StructRepr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDialect {
    pub name: String,
    pub ty: TypeDialect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StructRepr {
    C,
    Transparent,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDialect {
    pub variants: Vec<VariantDialect>,
    pub repr: EnumRepr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDialect {
    pub name: String,
    pub discriminant: Option<i64>,
    pub fields: Vec<FieldDialect>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnumRepr {
    C,
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionDialect {
    pub fields: Vec<FieldDialect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDialect {
    pub params: Vec<TypeDialect>,
    pub return_type: Box<TypeDialect>,
    pub effects: FnEffects,
    pub abi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FnEffects {
    pub may_panic: bool,
    pub may_alloc: bool,
    pub may_ffi: bool,
    pub may_unsafe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaticDialect {
    pub ty: TypeDialect,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorrowKind {
    Shared,
    Mutable,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lifetime {
    Static,
    Elided,
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyMode {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TypeDialect {
    Never,
    Unit,
    Bool,
    Char,
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
    Str,
    Slice(Box<TypeDialect>),
    Array(Box<TypeDialect>, u64),
    Tuple(Vec<TypeDialect>),
    Reference(Box<TypeRef>),
    Ptr(Box<TypeRef>),
    FnPtr(Box<FnPtrDialect>),
    Adt(ItemId, Vec<TypeDialect>),
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypeRef {
    pub pointee: Box<TypeDialect>,
    pub borrow_kind: BorrowKind,
    pub lifetime: Lifetime,
    pub is_const: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FnPtrDialect {
    pub params: Vec<TypeDialect>,
    pub return_type: Box<TypeDialect>,
    pub abi: String,
}

pub fn model_type_def(type_def: &NormalizedTypeDef) -> TypeDialect {
    match type_def {
        NormalizedTypeDef::Primitive(p) => model_primitive_type(p),
        NormalizedTypeDef::Struct { .. } => TypeDialect::Error,
        NormalizedTypeDef::Enum { .. } => TypeDialect::Error,
        NormalizedTypeDef::Union { .. } => TypeDialect::Error,
        NormalizedTypeDef::Tuple(elems) => {
            TypeDialect::Tuple(elems.iter().map(|t| model_type_ref(*t)).collect())
        }
        NormalizedTypeDef::Array(elem, len) => {
            TypeDialect::Array(Box::new(model_type_ref(*elem)), *len)
        }
        NormalizedTypeDef::Slice(elem) => TypeDialect::Slice(Box::new(model_type_ref(*elem))),
        NormalizedTypeDef::Ref(pointee, borrow_kind) => TypeDialect::Reference(Box::new(TypeRef {
            pointee: Box::new(model_type_ref(*pointee)),
            borrow_kind: match borrow_kind {
                NormalizedBorrowKind::Shared => BorrowKind::Shared,
                NormalizedBorrowKind::Mut => BorrowKind::Mutable,
                NormalizedBorrowKind::Shallow => BorrowKind::Shared,
                NormalizedBorrowKind::TwoPhaseMut => BorrowKind::Mutable,
            },
            lifetime: Lifetime::Elided,
            is_const: false,
        })),
        NormalizedTypeDef::RawPtr(pointee, _mutable) => TypeDialect::Ptr(Box::new(TypeRef {
            pointee: Box::new(model_type_ref(*pointee)),
            borrow_kind: BorrowKind::Unique,
            lifetime: Lifetime::Elided,
            is_const: false,
        })),
        NormalizedTypeDef::FnPtr { params, ret } => TypeDialect::FnPtr(Box::new(FnPtrDialect {
            params: params.iter().map(|p| model_type_ref(*p)).collect(),
            return_type: Box::new(model_type_ref(*ret)),
            abi: "Rust".to_string(),
        })),
    }
}

pub fn model_type_ref(_type_ref: chimera_rust_mir_import::StableTypeRef) -> TypeDialect {
    TypeDialect::Error
}

fn model_primitive_type(p: &NormalizedPrimitiveType) -> TypeDialect {
    match p {
        NormalizedPrimitiveType::Never => TypeDialect::Never,
        NormalizedPrimitiveType::Bool => TypeDialect::Bool,
        NormalizedPrimitiveType::Char => TypeDialect::Char,
        NormalizedPrimitiveType::I8 => TypeDialect::I8,
        NormalizedPrimitiveType::I16 => TypeDialect::I16,
        NormalizedPrimitiveType::I32 => TypeDialect::I32,
        NormalizedPrimitiveType::I64 => TypeDialect::I64,
        NormalizedPrimitiveType::I128 => TypeDialect::I128,
        NormalizedPrimitiveType::Isize => TypeDialect::Isize,
        NormalizedPrimitiveType::U8 => TypeDialect::U8,
        NormalizedPrimitiveType::U16 => TypeDialect::U16,
        NormalizedPrimitiveType::U32 => TypeDialect::U32,
        NormalizedPrimitiveType::U64 => TypeDialect::U64,
        NormalizedPrimitiveType::U128 => TypeDialect::U128,
        NormalizedPrimitiveType::Usize => TypeDialect::Usize,
        NormalizedPrimitiveType::F32 => TypeDialect::F32,
        NormalizedPrimitiveType::F64 => TypeDialect::F64,
        NormalizedPrimitiveType::Str => TypeDialect::Str,
        NormalizedPrimitiveType::Unit => TypeDialect::Tuple(vec![]),
    }
}

pub fn verify_dialect(dialect: &DialectContext) -> Result<(), DialectError> {
    for (item_id, item) in &dialect.items {
        verify_item(item).map_err(|e| DialectError::ItemError(*item_id))?;
    }
    Ok(())
}

fn verify_item(item: &ItemDialect) -> Result<(), DialectError> {
    match &item.kind {
        ItemKind::Struct(s) => verify_struct(s),
        ItemKind::Enum(e) => verify_enum(e),
        ItemKind::Fn(f) => verify_fn(f),
        _ => Ok(()),
    }
}

fn verify_struct(s: &StructDialect) -> Result<(), DialectError> {
    if s.fields.is_empty() {
        return Err(DialectError::EmptyStruct);
    }
    Ok(())
}

fn verify_enum(e: &EnumDialect) -> Result<(), DialectError> {
    if e.variants.is_empty() {
        return Err(DialectError::EmptyEnum);
    }
    Ok(())
}

fn verify_fn(f: &FnDialect) -> Result<(), DialectError> {
    if f.effects.may_panic && f.effects.may_unsafe {
        return Err(DialectError::UnsafePanicCombination);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DialectError {
    #[error("item error")]
    ItemError(ItemId),
    #[error("empty struct")]
    EmptyStruct,
    #[error("empty enum")]
    EmptyEnum,
    #[error("unsafe and panic combination")]
    UnsafePanicCombination,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_effects_default() {
        let effects = FnEffects::default();
        assert!(!effects.may_panic);
        assert!(!effects.may_alloc);
        assert!(!effects.may_ffi);
        assert!(!effects.may_unsafe);
    }

    #[test]
    fn test_struct_repr_equality() {
        assert_eq!(StructRepr::C, StructRepr::C);
        assert_eq!(StructRepr::Transparent, StructRepr::Transparent);
        assert_ne!(StructRepr::C, StructRepr::Transparent);
    }

    #[test]
    fn test_lifetime_equality() {
        assert_eq!(Lifetime::Static, Lifetime::Static);
        assert_eq!(Lifetime::Elided, Lifetime::Elided);
        assert_eq!(
            Lifetime::Named("'a".to_string()),
            Lifetime::Named("'a".to_string())
        );
    }

    #[test]
    fn test_type_dialect_primitives() {
        assert_eq!(TypeDialect::I32, TypeDialect::I32);
        assert_eq!(TypeDialect::U64, TypeDialect::U64);
        assert_eq!(TypeDialect::Bool, TypeDialect::Bool);
        assert_eq!(TypeDialect::F64, TypeDialect::F64);
    }

    #[test]
    fn test_borrow_kind_equality() {
        assert_eq!(BorrowKind::Shared, BorrowKind::Shared);
        assert_eq!(BorrowKind::Mutable, BorrowKind::Mutable);
        assert_eq!(BorrowKind::Unique, BorrowKind::Unique);
    }

    #[test]
    fn test_safety_mode_equality() {
        assert_eq!(SafetyMode::Safe, SafetyMode::Safe);
        assert_eq!(SafetyMode::Unsafe, SafetyMode::Unsafe);
    }

    #[test]
    fn test_dialect_context_default() {
        let ctx = DialectContext::default();
        assert!(ctx.items.is_empty());
        assert!(ctx.type_cache.is_empty());
    }

    #[test]
    fn test_verify_empty_struct_fails() {
        let s = StructDialect {
            fields: vec![],
            repr: StructRepr::Rust,
        };
        let result = verify_struct(&s);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_empty_enum_fails() {
        let e = EnumDialect {
            variants: vec![],
            repr: EnumRepr::U32,
        };
        let result = verify_enum(&e);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_fn_unsafe_panic_fails() {
        let f = FnDialect {
            params: vec![],
            return_type: Box::new(TypeDialect::Unit),
            effects: FnEffects {
                may_panic: true,
                may_unsafe: true,
                ..Default::default()
            },
            abi: "Rust".to_string(),
        };
        let result = verify_fn(&f);
        assert!(result.is_err());
    }

    #[test]
    fn test_fn_dialect_serialization() {
        let f = FnDialect {
            params: vec![TypeDialect::I32],
            return_type: Box::new(TypeDialect::Bool),
            effects: FnEffects::default(),
            abi: "C".to_string(),
        };
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("I32"));
        assert!(json.contains("C"));
    }

    #[test]
    fn test_item_dialect_serialization() {
        let item = ItemDialect {
            item_id: ItemId(42),
            name: "test_fn".to_string(),
            kind: ItemKind::Fn(FnDialect {
                params: vec![],
                return_type: Box::new(TypeDialect::Unit),
                effects: FnEffects::default(),
                abi: "Rust".to_string(),
            }),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test_fn"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_type_dialect_reference() {
        let ref_type = TypeDialect::Reference(Box::new(TypeRef {
            pointee: Box::new(TypeDialect::I32),
            borrow_kind: BorrowKind::Shared,
            lifetime: Lifetime::Elided,
            is_const: true,
        }));
        let json = serde_json::to_string(&ref_type).unwrap();
        assert!(json.contains("Reference"));
    }

    #[test]
    fn test_type_dialect_fnptr() {
        let fnptr = TypeDialect::FnPtr(Box::new(FnPtrDialect {
            params: vec![TypeDialect::I32, TypeDialect::Bool],
            return_type: Box::new(TypeDialect::Unit),
            abi: "C".to_string(),
        }));
        let json = serde_json::to_string(&fnptr).unwrap();
        assert!(json.contains("FnPtr"));
        assert!(json.contains("C"));
    }

    #[test]
    fn test_model_primitive_type() {
        assert_eq!(
            model_primitive_type(&NormalizedPrimitiveType::I32),
            TypeDialect::I32
        );
        assert_eq!(
            model_primitive_type(&NormalizedPrimitiveType::Bool),
            TypeDialect::Bool
        );
        assert_eq!(
            model_primitive_type(&NormalizedPrimitiveType::Str),
            TypeDialect::Str
        );
        assert_eq!(
            model_primitive_type(&NormalizedPrimitiveType::Unit),
            TypeDialect::Tuple(vec![])
        );
    }
}
