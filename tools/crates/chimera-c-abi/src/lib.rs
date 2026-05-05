//! Chimera C ABI extraction and lowering crate.
//!
//! Computes semantic ABI and physical ABI for C constructs:
//! - Functions, pointers, arrays, structs, unions, enums
//! - Varargs, calling conventions, and pointer aliasing
//!
//! Task 13: C ABI extraction/lowering crate

use chimera_c_schema::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for ABI operations
pub type Result<T> = std::result::Result<T, AbiError>;

/// ABI-related errors
#[derive(Debug, thiserror::Error)]
pub enum AbiError {
    #[error("incompatible calling conventions: {0} vs {1}")]
    IncompatibleCallingConventions(String, String),
    #[error("varargs not supported for direct crossing: {0}")]
    VarargsUnsupported(String),
    #[error("function pointer ABI mismatch: {0}")]
    FunctionPointerMismatch(String),
    #[error("invalid struct layout: {0}")]
    InvalidStructLayout(String),
    #[error("invalid union layout: {0}")]
    InvalidUnionLayout(String),
    #[error("pointer nullability violation: {0}")]
    PointerNullabilityViolation(String),
    #[error("pointer aliasing violation: {0}")]
    PointerAliasingViolation(String),
    #[error("errno convention violation: {0}")]
    ErrnoViolation(String),
    #[error("allocator contract violation: {0}")]
    AllocatorViolation(String),
    #[error("callback ABI mismatch: {0}")]
    CallbackMismatch(String),
    #[error("missing wrapper for varargs: {0}")]
    MissingVarargsWrapper(String),
}

/// C ABI calling conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CCallingConvention {
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall,
    Vectorcall,
    Regparm(u32),
    SysV,
    Win64,
    /// Chimera native calling convention
    ChimeraNative,
}

impl Default for CCallingConvention {
    fn default() -> Self {
        CCallingConvention::Cdecl
    }
}

impl CCallingConvention {
    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "cdecl" | "c" => Some(CCallingConvention::Cdecl),
            "stdcall" => Some(CCallingConvention::Stdcall),
            "fastcall" => Some(CCallingConvention::Fastcall),
            "thiscall" => Some(CCallingConvention::Thiscall),
            "vectorcall" => Some(CCallingConvention::Vectorcall),
            "sysv" | "sysv64" => Some(CCallingConvention::SysV),
            "win64" | "windows" => Some(CCallingConvention::Win64),
            _ => None,
        }
    }

    /// Get the platform default calling convention
    pub fn platform_default(target: &str) -> Self {
        if target.contains("windows") {
            CCallingConvention::Win64
        } else {
            CCallingConvention::SysV
        }
    }
}

/// ABI parameter passing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParam {
    /// Position in parameter list
    pub position: u32,
    /// How this parameter is passed
    pub passing: PassingConvention,
    /// Whether passed by value
    pub by_val: bool,
    /// Alignment requirement
    pub align: u32,
    /// Size in bytes
    pub size: u64,
    /// Register allocated (if applicable)
    pub register: Option<AbiRegister>,
}

/// Register for ABI purposes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbiRegister {
    pub name: String,
    pub index: u32,
}

/// How a parameter is passed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PassingConvention {
    Direct,
    ByReference,
    Split,
    Ignore,
    /// Passed via pointer (caller retains pointer)
    PointerIn,
    /// Passed via pointer (callee owns)
    PointerOut,
}

impl Default for PassingConvention {
    fn default() -> Self {
        PassingConvention::Direct
    }
}

/// Return value handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiReturn {
    /// How the return value is passed
    pub passing: PassingConvention,
    /// Alignment requirement
    pub align: u32,
    /// Size in bytes
    pub size: u64,
    /// Register used (if applicable)
    pub register: Option<AbiRegister>,
}

impl Default for AbiReturn {
    fn default() -> Self {
        Self {
            passing: PassingConvention::Direct,
            align: 8,
            size: 0,
            register: None,
        }
    }
}

/// Function ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAbi {
    /// Function name (mangled if needed)
    pub name: String,
    /// Calling convention
    pub cconv: CCallingConvention,
    /// Parameters with passing info
    pub params: Vec<AbiParam>,
    /// Return value handling
    pub ret: Option<AbiReturn>,
    /// Whether this function uses varargs
    pub is_varargs: bool,
    /// Number of fixed (non-varargs) parameters
    pub fixed_param_count: u32,
    /// ABI fingerprint for caching
    pub fingerprint: String,
}

impl FunctionAbi {
    /// Compute fingerprint for this ABI
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.name.as_bytes());
        hasher.update(format!("{:?}", self.cconv).as_bytes());
        hasher.update(&self.fixed_param_count.to_le_bytes());
        hasher.update(&[if self.is_varargs { 1u8 } else { 0u8 }]);
        for param in &self.params {
            hasher.update(&param.position.to_le_bytes());
            hasher.update(format!("{:?}", param.passing).as_bytes());
            hasher.update(&param.size.to_le_bytes());
            hasher.update(&param.align.to_le_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

/// Pointer ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerAbi {
    /// Pointee type reference
    pub pointee_type: TypeRef,
    /// Nullability
    pub nullability: PointerNullability,
    /// Constness
    pub is_const: bool,
    /// Volatile
    pub is_volatile: bool,
    /// Restrict
    pub is_restrict: bool,
    /// Address space
    pub address_space: u32,
    /// Size of pointer in bytes
    pub pointer_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PointerNullability {
    Nullable,
    NonNull,
    Raw,
    /// Borrowed pointer with lifetime
    Borrowed,
    /// Mutable borrowed pointer
    BorrowedMut,
}

impl Default for PointerNullability {
    fn default() -> Self {
        PointerNullability::Raw
    }
}

/// Function pointer ABI (callbacks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPointerAbi {
    /// Parameter types
    pub param_types: Vec<TypeRef>,
    /// Return type
    pub return_type: Option<TypeRef>,
    /// Calling convention
    pub cconv: CCallingConvention,
    /// Whether this is varargs
    pub is_varargs: bool,
    /// Nullability of the pointer itself
    pub nullability: PointerNullability,
}

/// Array ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayAbi {
    /// Element type
    pub element_type: TypeRef,
    /// Number of elements (None for incomplete)
    pub length: Option<u64>,
    /// Total size in bytes
    pub size: u64,
    /// Alignment
    pub align: u32,
}

impl ArrayAbi {
    /// Check if this is an incomplete array type
    pub fn is_incomplete(&self) -> bool {
        self.length.is_none()
    }
}

/// Struct ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructAbi {
    /// Struct name
    pub name: Option<String>,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub align: u32,
    /// Whether packed
    pub is_packed: bool,
    /// Packing alignment (if specified)
    pub pack_align: Option<u32>,
    /// Field ABIs
    pub fields: Vec<FieldAbi>,
    /// ABI fingerprint
    pub fingerprint: String,
}

impl StructAbi {
    /// Compute fingerprint for this struct ABI
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.size.to_le_bytes());
        hasher.update(&self.align.to_le_bytes());
        hasher.update(&[if self.is_packed { 1u8 } else { 0u8 }]);
        if let Some(p) = self.pack_align {
            hasher.update(&p.to_le_bytes());
        }
        for field in &self.fields {
            hasher.update(field.name.as_bytes());
            hasher.update(&field.offset.to_le_bytes());
            hasher.update(&field.size.to_le_bytes());
            hasher.update(&field.align.to_le_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

/// Field ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAbi {
    /// Field name
    pub name: String,
    /// Offset in bytes
    pub offset: u64,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub align: u32,
    /// Bitfield info (if applicable)
    pub bitfield: Option<BitfieldAbi>,
}

/// Bitfield ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitfieldAbi {
    /// Container offset (byte offset to containing storage unit)
    pub container_offset: u64,
    /// Bit offset within the container
    pub bit_offset: u8,
    /// Bit width
    pub bit_width: u8,
    /// Storage unit type
    pub storage_type: TypeRef,
    /// Whether signed
    pub is_signed: bool,
}

impl Default for FieldAbi {
    fn default() -> Self {
        Self {
            name: String::new(),
            offset: 0,
            size: 0,
            align: 1,
            bitfield: None,
        }
    }
}

/// Union ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionAbi {
    /// Union name
    pub name: Option<String>,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub align: u32,
    /// Active member assumption (for unsafe operations)
    pub active_member: Option<String>,
    /// Fingerprint
    pub fingerprint: String,
}

impl UnionAbi {
    /// Compute fingerprint
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.size.to_le_bytes());
        hasher.update(&self.align.to_le_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

/// Enum ABI information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumAbi {
    /// Enum name
    pub name: Option<String>,
    /// Underlying integer type
    pub underlying_type: TypeRef,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub align: u32,
    /// Signedness
    pub is_signed: bool,
}

/// ABI validation result
#[derive(Debug, Clone)]
pub struct AbiValidation {
    /// Whether the ABI is valid
    pub is_valid: bool,
    /// Violations found
    pub violations: Vec<AbiViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiViolation {
    pub code: CDiagnosticCode,
    pub message: String,
    pub location: Option<SourceSpan>,
}

/// Varargs ABI handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarargsAbi {
    /// Function name using varargs
    pub function: String,
    /// Number of fixed parameters
    pub fixed_param_count: u32,
    /// Whether a wrapper is required
    pub requires_wrapper: bool,
    /// ABI-safe parameter types for varargs portion
    pub varargs_param_types: Vec<TypeRef>,
}

/// Callback ABI contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackContract {
    /// Callback function pointer ABI
    pub func_ptr: FunctionPointerAbi,
    /// User data pointer nullability
    pub user_data_nullability: PointerNullability,
    /// Panic/error policy
    pub panic_policy: CallbackPanicPolicy,
}

/// Callback panic policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallbackPanicPolicy {
    /// Callback cannot panic (abort if it does)
    NoPanic,
    /// Callback may panic, foreign code catches it
    Catch,
    /// Callback may panic, translated to error return
    ErrorReturn,
}

/// Pointer aliasing contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasingContract {
    /// Pointer that is restricted
    pub restrict_pointer: TypeRef,
    /// Other pointers that must not alias
    pub non_aliasing_pointers: Vec<TypeRef>,
    /// Contract description
    pub description: String,
}

/// Errno/status convention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrnoConvention {
    /// Function name
    pub function: String,
    /// How errors are reported
    pub error_reporting: ErrorReporting,
    /// Domain for error codes
    pub domain: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorReporting {
    /// Returns negative on error (like POSIX)
    Negative,
    /// Returns ch_status_t
    ChStatus,
    /// Uses errno parameter
    ErrnoParam,
    /// Returns via out parameter
    OutParam,
}

/// Allocator contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorContract {
    /// Allocator function
    pub allocate: String,
    /// Deallocator function
    pub deallocate: String,
    /// Allocator ID
    pub allocator_id: String,
    /// Whether this is a system allocator
    pub is_system: bool,
}

/// ABI context for a translation unit
#[derive(Debug, Clone, Default)]
pub struct AbiContext {
    /// Function ABIs
    pub functions: HashMap<String, FunctionAbi>,
    /// Pointer ABIs
    pub pointers: HashMap<TypeRef, PointerAbi>,
    /// Array ABIs
    pub arrays: HashMap<TypeRef, ArrayAbi>,
    /// Struct ABIs
    pub structs: HashMap<TypeRef, StructAbi>,
    /// Union ABIs
    pub unions: HashMap<TypeRef, UnionAbi>,
    /// Enum ABIs
    pub enums: HashMap<TypeRef, EnumAbi>,
    /// Function pointer ABIs
    pub function_pointers: HashMap<TypeRef, FunctionPointerAbi>,
    /// Varargs contracts
    pub varargs: HashMap<String, VarargsAbi>,
    /// Callback contracts
    pub callbacks: HashMap<String, CallbackContract>,
    /// Aliasing contracts
    pub aliasing: Vec<AliasingContract>,
    /// Errno conventions
    pub errno_conventions: HashMap<String, ErrnoConvention>,
    /// Allocator contracts
    pub allocators: HashMap<String, AllocatorContract>,
}

impl AbiContext {
    /// Add a function ABI
    pub fn add_function(&mut self, name: String, abi: FunctionAbi) {
        self.functions.insert(name, abi);
    }

    /// Add a struct ABI
    pub fn add_struct(&mut self, type_ref: TypeRef, abi: StructAbi) {
        self.structs.insert(type_ref, abi);
    }

    /// Get function ABI
    pub fn get_function(&self, name: &str) -> Option<&FunctionAbi> {
        self.functions.get(name)
    }

    /// Get struct ABI
    pub fn get_struct(&self, type_ref: TypeRef) -> Option<&StructAbi> {
        self.structs.get(&type_ref)
    }
}

/// ABI extractor trait - implemented by clang-based extraction
pub trait AbiExtractor {
    /// Extract function ABI
    fn extract_function_abi(&self, decl: &FunctionDecl) -> Result<FunctionAbi>;

    /// Extract struct ABI
    fn extract_struct_abi(&self, decl: &StructDecl) -> Result<StructAbi>;

    /// Extract union ABI
    fn extract_union_abi(&self, decl: &UnionDecl) -> Result<UnionAbi>;

    /// Extract enum ABI
    fn extract_enum_abi(&self, decl: &EnumDecl) -> Result<EnumAbi>;

    /// Extract pointer ABI
    fn extract_pointer_abi(&self, type_ref: TypeRef, pointee_type: TypeRef) -> Result<PointerAbi>;

    /// Extract array ABI
    fn extract_array_abi(
        &self,
        type_ref: TypeRef,
        element_type: TypeRef,
        length: Option<u64>,
    ) -> Result<ArrayAbi>;
}

/// ABI validator trait
pub trait AbiValidator {
    /// Validate function ABI crossing boundary
    fn validate_function_abi(&self, abi: &FunctionAbi) -> AbiValidation;

    /// Validate struct ABI crossing boundary
    fn validate_struct_abi(&self, abi: &StructAbi) -> AbiValidation;

    /// Validate pointer contract
    fn validate_pointer_contract(&self, ptr_abi: &PointerAbi, contract: &str) -> AbiValidation;

    /// Validate varargs crossing
    fn validate_varargs(&self, varargs: &VarargsAbi) -> AbiValidation;
}

/// C ABI extractor using Clang facts
pub struct CAbiExtractor {
    target: String,
}

impl CAbiExtractor {
    /// Create new ABI extractor
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
        }
    }

    /// Extract ABI for a function declaration
    pub fn extract_function(&self, decl: &FunctionDecl) -> Result<FunctionAbi> {
        let cconv = CCallingConvention::parse(&decl.calling_convention)
            .unwrap_or_else(|| CCallingConvention::platform_default(&self.target));

        let params: Vec<AbiParam> = decl
            .params
            .iter()
            .enumerate()
            .map(|(i, _p)| AbiParam {
                position: i as u32,
                passing: PassingConvention::Direct,
                by_val: true,
                align: 8, // Would need more sophisticated extraction
                size: 8,  // Would need type-based lookup
                register: None,
            })
            .collect();

        let ret = Some(AbiReturn {
            passing: PassingConvention::Direct,
            align: 8,
            size: 8,
            register: None,
        });

        let fixed_param_count = params.len() as u32;

        let mut abi = FunctionAbi {
            name: decl.name.clone(),
            cconv,
            params,
            ret,
            is_varargs: false,
            fixed_param_count,
            fingerprint: String::new(),
        };

        abi.fingerprint = abi.compute_fingerprint();
        Ok(abi)
    }

    /// Extract ABI for a struct declaration
    pub fn extract_struct(&self, decl: &StructDecl) -> Result<StructAbi> {
        let fields: Vec<FieldAbi> = decl
            .fields
            .iter()
            .map(|f| FieldAbi {
                name: f.name.clone(),
                offset: f.offset,
                size: f.size,
                align: f.align,
                bitfield: None,
            })
            .collect();

        let size = fields.last().map(|f| f.offset + f.size).unwrap_or(0);
        let align = fields.iter().map(|f| f.align).max().unwrap_or(1);

        let mut abi = StructAbi {
            name: decl.name.clone(),
            size,
            align,
            is_packed: decl.is_packed,
            pack_align: decl.pack_align,
            fields,
            fingerprint: String::new(),
        };

        abi.fingerprint = abi.compute_fingerprint();
        Ok(abi)
    }

    /// Extract ABI for a union declaration
    pub fn extract_union(&self, decl: &UnionDecl) -> Result<UnionAbi> {
        let mut abi = UnionAbi {
            name: decl.name.clone(),
            size: decl.size,
            align: decl.align,
            active_member: None,
            fingerprint: String::new(),
        };

        abi.fingerprint = abi.compute_fingerprint();
        Ok(abi)
    }

    /// Extract ABI for an enum declaration
    pub fn extract_enum(&self, decl: &EnumDecl) -> Result<EnumAbi> {
        let underlying_type = decl.underlying_type.unwrap_or(TypeRef(4)); // Default to int

        Ok(EnumAbi {
            name: decl.name.clone(),
            underlying_type,
            size: 4, // Would need extraction from clang
            align: 4,
            is_signed: true, // Would need extraction
        })
    }
}

impl AbiExtractor for CAbiExtractor {
    fn extract_function_abi(&self, decl: &FunctionDecl) -> Result<FunctionAbi> {
        self.extract_function(decl)
    }

    fn extract_struct_abi(&self, decl: &StructDecl) -> Result<StructAbi> {
        self.extract_struct(decl)
    }

    fn extract_union_abi(&self, decl: &UnionDecl) -> Result<UnionAbi> {
        self.extract_union(decl)
    }

    fn extract_enum_abi(&self, decl: &EnumDecl) -> Result<EnumAbi> {
        self.extract_enum(decl)
    }

    fn extract_pointer_abi(&self, _type_ref: TypeRef, pointee_type: TypeRef) -> Result<PointerAbi> {
        Ok(PointerAbi {
            pointee_type,
            nullability: PointerNullability::Raw,
            is_const: false,
            is_volatile: false,
            is_restrict: false,
            address_space: 0,
            pointer_size: 8,
        })
    }

    fn extract_array_abi(
        &self,
        _type_ref: TypeRef,
        element_type: TypeRef,
        length: Option<u64>,
    ) -> Result<ArrayAbi> {
        let size = length.unwrap_or(0) * 8; // Would need element size lookup
        Ok(ArrayAbi {
            element_type,
            length,
            size,
            align: 8,
        })
    }
}

// ============================================================================
// ABI Fingerprints (Task 128)
// ============================================================================

/// ABI fingerprint components for a function symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFingerprintComponents {
    /// Symbol name
    pub symbol: String,
    /// Calling convention
    pub calling_convention: String,
    /// Return type fingerprint
    pub return_type_fp: String,
    /// Parameter type fingerprints
    pub param_type_fps: Vec<String>,
    /// Target triple
    pub target: String,
    /// Compiler config hash
    pub compiler_config_hash: String,
}

impl AbiFingerprintComponents {
    /// Compute final fingerprint hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-abi-fingerprint");
        hasher.update_str(&self.symbol);
        hasher.update_str(&self.calling_convention);
        hasher.update_str(&self.return_type_fp);
        for fp in &self.param_type_fps {
            hasher.update_str(fp);
        }
        hasher.update_str(&self.target);
        hasher.update_str(&self.compiler_config_hash);
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// ABI fingerprint for function symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFingerprint {
    /// Symbol this fingerprint is for
    pub symbol: String,
    /// The computed hash
    pub hash: String,
    /// Components used to compute
    pub components: Vec<String>,
}

impl AbiFingerprint {
    /// Create new fingerprint from components
    pub fn from_components(components: &AbiFingerprintComponents) -> Self {
        let hash = components.compute_fingerprint();
        Self {
            symbol: components.symbol.clone(),
            hash: hash.clone(),
            components: vec![
                components.calling_convention.clone(),
                components.return_type_fp.clone(),
                format!("{} params", components.param_type_fps.len()),
                components.target.clone(),
            ],
        }
    }

    /// Check if this fingerprint matches another
    pub fn matches(&self, other: &AbiFingerprint) -> bool {
        self.hash == other.hash
    }
}

/// ABI fingerprint engine
pub struct AbiFingerprintEngine {
    target: String,
    compiler_config_hash: String,
}

impl AbiFingerprintEngine {
    /// Create new fingerprint engine
    pub fn new(target: String, compiler_config_hash: String) -> Self {
        Self {
            target,
            compiler_config_hash,
        }
    }

    /// Compute fingerprint for a function
    pub fn compute_function_fingerprint(
        &self,
        symbol: &str,
        calling_convention: &CCallingConvention,
        return_type_fp: &str,
        param_type_fps: &[String],
    ) -> AbiFingerprint {
        let components = AbiFingerprintComponents {
            symbol: symbol.to_string(),
            calling_convention: format!("{:?}", calling_convention).to_lowercase(),
            return_type_fp: return_type_fp.to_string(),
            param_type_fps: param_type_fps.to_vec(),
            target: self.target.clone(),
            compiler_config_hash: self.compiler_config_hash.clone(),
        };
        AbiFingerprint::from_components(&components)
    }

    /// Check if two fingerprints are compatible
    pub fn are_compatible(&self, fp1: &AbiFingerprint, fp2: &AbiFingerprint) -> bool {
        fp1.matches(fp2)
    }
}

/// Layout fingerprint components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFingerprintComponents {
    /// Type kind (struct, union, enum)
    pub type_kind: String,
    /// Type name (if any)
    pub type_name: Option<String>,
    /// Field fingerprints (for structs)
    pub field_fps: Vec<FieldFingerprint>,
    /// Size in bytes
    pub size: u64,
    /// Alignment
    pub alignment: u32,
    /// Packed flag
    pub is_packed: bool,
    /// Pack alignment (if set)
    pub pack_align: Option<u32>,
    /// Target ABI
    pub target_abi: String,
}

impl LayoutFingerprintComponents {
    /// Compute layout fingerprint hash using BLAKE3
    pub fn compute_fingerprint(&self) -> String {
        let mut hasher = zigmera_hash::Blake3Hasher::with_schema_tag("c-layout-fingerprint");
        hasher.update_str(&self.type_kind);
        if let Some(ref name) = self.type_name {
            hasher.update_str(name);
        }
        for fp in &self.field_fps {
            // FieldFingerprint: name, offset, size, field_type_fp
            hasher.update_str(&fp.name);
            hasher.update_u64(fp.offset);
            hasher.update_u64(fp.size);
            hasher.update_str(&fp.field_type_fp);
        }
        hasher.update_u64(self.size as u64);
        hasher.update_u64(self.alignment as u64);
        hasher.update_bool(self.is_packed);
        if let Some(pa) = self.pack_align {
            hasher.update_u64(pa as u64);
        }
        hasher.update_str(&self.target_abi);
        hasher.finalize().as_hex()[..16].to_string()
    }
}

/// Field fingerprint
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct FieldFingerprint {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub field_type_fp: String,
}

/// Layout fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFingerprint {
    pub type_name: String,
    pub hash: String,
    pub size: u64,
    pub alignment: u32,
}

impl LayoutFingerprint {
    /// Create new fingerprint from components
    pub fn from_components(components: &LayoutFingerprintComponents) -> Self {
        let hash = components.compute_fingerprint();
        Self {
            type_name: components
                .type_name
                .clone()
                .unwrap_or_else(|| "unnamed".to_string()),
            hash,
            size: components.size,
            alignment: components.alignment,
        }
    }

    /// Check if this fingerprint matches another
    pub fn matches(&self, other: &LayoutFingerprint) -> bool {
        self.hash == other.hash
    }
}

/// Layout fingerprint engine
pub struct LayoutFingerprintEngine {
    target_abi: String,
}

impl LayoutFingerprintEngine {
    /// Create new layout fingerprint engine
    pub fn new(target_abi: String) -> Self {
        Self { target_abi }
    }

    /// Compute layout fingerprint for a struct
    pub fn compute_struct_fingerprint(
        &self,
        name: Option<&str>,
        size: u64,
        alignment: u32,
        is_packed: bool,
        pack_align: Option<u32>,
        fields: &[(String, String, u64, u64)], // (name, type_fp, offset, size)
    ) -> LayoutFingerprint {
        let field_fps: Vec<FieldFingerprint> = fields
            .iter()
            .map(|(name, type_fp, offset, size)| FieldFingerprint {
                name: name.clone(),
                offset: *offset,
                size: *size,
                field_type_fp: type_fp.clone(),
            })
            .collect();

        let components = LayoutFingerprintComponents {
            type_kind: "struct".to_string(),
            type_name: name.map(|s| s.to_string()),
            field_fps,
            size,
            alignment,
            is_packed,
            pack_align,
            target_abi: self.target_abi.clone(),
        };
        LayoutFingerprint::from_components(&components)
    }
}

#[cfg(test)]
mod fingerprint_tests {
    use super::*;

    #[test]
    fn test_abi_fingerprint_components_compute() {
        let components = AbiFingerprintComponents {
            symbol: "my_func".to_string(),
            calling_convention: "sysv".to_string(),
            return_type_fp: "i32".to_string(),
            param_type_fps: vec!["i32".to_string(), "i64".to_string()],
            target: "x86_64-unknown-linux-gnu".to_string(),
            compiler_config_hash: "config123".to_string(),
        };
        let fp = components.compute_fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 16); // 16 hex chars = 64 bits
    }

    #[test]
    fn test_abi_fingerprint_from_components() {
        let components = AbiFingerprintComponents {
            symbol: "test_func".to_string(),
            calling_convention: "cdecl".to_string(),
            return_type_fp: "void".to_string(),
            param_type_fps: vec![],
            target: "x86_64-unknown-linux-gnu".to_string(),
            compiler_config_hash: "abc".to_string(),
        };
        let fp = AbiFingerprint::from_components(&components);
        assert_eq!(fp.symbol, "test_func");
        assert!(!fp.hash.is_empty());
    }

    #[test]
    fn test_abi_fingerprint_matches() {
        let components = AbiFingerprintComponents {
            symbol: "func".to_string(),
            calling_convention: "sysv".to_string(),
            return_type_fp: "i32".to_string(),
            param_type_fps: vec!["i32".to_string()],
            target: "x86_64".to_string(),
            compiler_config_hash: "cfg".to_string(),
        };
        let fp1 = AbiFingerprint::from_components(&components);
        let fp2 = AbiFingerprint::from_components(&components);
        assert!(fp1.matches(&fp2));
    }

    #[test]
    fn test_abi_fingerprint_engine() {
        let engine = AbiFingerprintEngine::new(
            "x86_64-unknown-linux-gnu".to_string(),
            "clang-15".to_string(),
        );
        let fp = engine.compute_function_fingerprint(
            "my_function",
            &CCallingConvention::SysV,
            "i32",
            &["i64".to_string(), "i32".to_string()],
        );
        assert_eq!(fp.symbol, "my_function");
        assert!(!fp.hash.is_empty());
    }

    #[test]
    fn test_layout_fingerprint_components_compute() {
        let components = LayoutFingerprintComponents {
            type_kind: "struct".to_string(),
            type_name: Some("Point".to_string()),
            field_fps: vec![
                FieldFingerprint {
                    name: "x".to_string(),
                    offset: 0,
                    size: 4,
                    field_type_fp: "i32".to_string(),
                },
                FieldFingerprint {
                    name: "y".to_string(),
                    offset: 4,
                    size: 4,
                    field_type_fp: "i32".to_string(),
                },
            ],
            size: 8,
            alignment: 4,
            is_packed: false,
            pack_align: None,
            target_abi: "sysv".to_string(),
        };
        let fp = components.compute_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_layout_fingerprint_engine() {
        let engine = LayoutFingerprintEngine::new("sysv".to_string());
        let fp = engine.compute_struct_fingerprint(
            Some("MyStruct"),
            16,
            8,
            false,
            None,
            &[
                ("a".to_string(), "i32".to_string(), 0, 4),
                ("b".to_string(), "i64".to_string(), 8, 8),
            ],
        );
        assert_eq!(fp.type_name, "MyStruct");
        assert_eq!(fp.size, 16);
        assert_eq!(fp.alignment, 8);
    }

    #[test]
    fn test_layout_fingerprint_matches() {
        let engine = LayoutFingerprintEngine::new("sysv".to_string());
        let fp1 = engine.compute_struct_fingerprint(
            Some("Test"),
            8,
            4,
            false,
            None,
            &[("x".to_string(), "i32".to_string(), 0, 4)],
        );
        let fp2 = engine.compute_struct_fingerprint(
            Some("Test"),
            8,
            4,
            false,
            None,
            &[("x".to_string(), "i32".to_string(), 0, 4)],
        );
        assert!(fp1.matches(&fp2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calling_convention_parse() {
        assert_eq!(
            CCallingConvention::parse("cdecl"),
            Some(CCallingConvention::Cdecl)
        );
        assert_eq!(
            CCallingConvention::parse("sysv"),
            Some(CCallingConvention::SysV)
        );
        assert_eq!(
            CCallingConvention::parse("win64"),
            Some(CCallingConvention::Win64)
        );
        assert_eq!(CCallingConvention::parse("unknown"), None);
    }

    #[test]
    fn test_calling_convention_platform_default() {
        let unix = CCallingConvention::platform_default("x86_64-unknown-linux-gnu");
        assert_eq!(unix, CCallingConvention::SysV);

        let windows = CCallingConvention::platform_default("x86_64-pc-windows-msvc");
        assert_eq!(windows, CCallingConvention::Win64);
    }

    #[test]
    fn test_passing_convention_default() {
        let passing = PassingConvention::default();
        assert_eq!(passing, PassingConvention::Direct);
    }

    #[test]
    fn test_pointer_nullability_default() {
        let nullability = PointerNullability::default();
        assert_eq!(nullability, PointerNullability::Raw);
    }

    #[test]
    fn test_function_abi_fingerprint() {
        let abi = FunctionAbi {
            name: "foo".to_string(),
            cconv: CCallingConvention::Cdecl,
            params: vec![],
            ret: None,
            is_varargs: false,
            fixed_param_count: 0,
            fingerprint: String::new(),
        };

        let fp = abi.compute_fingerprint();
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64); // blake3 hex
    }

    #[test]
    fn test_struct_abi_fingerprint() {
        let abi = StructAbi {
            name: Some("Point".to_string()),
            size: 8,
            align: 8,
            is_packed: false,
            pack_align: None,
            fields: vec![
                FieldAbi {
                    name: "x".to_string(),
                    offset: 0,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
                FieldAbi {
                    name: "y".to_string(),
                    offset: 4,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
            ],
            fingerprint: String::new(),
        };

        let fp = abi.compute_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_union_abi_fingerprint() {
        let abi = UnionAbi {
            name: Some("Data".to_string()),
            size: 8,
            align: 8,
            active_member: None,
            fingerprint: String::new(),
        };

        let fp = abi.compute_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn test_array_abi_is_incomplete() {
        let complete = ArrayAbi {
            element_type: TypeRef(0),
            length: Some(10),
            size: 80,
            align: 8,
        };
        assert!(!complete.is_incomplete());

        let incomplete = ArrayAbi {
            element_type: TypeRef(0),
            length: None,
            size: 0,
            align: 8,
        };
        assert!(incomplete.is_incomplete());
    }

    #[test]
    fn test_abi_context_insert_and_lookup() {
        let mut ctx = AbiContext::default();

        let func_abi = FunctionAbi {
            name: "test_func".to_string(),
            cconv: CCallingConvention::Cdecl,
            params: vec![],
            ret: None,
            is_varargs: false,
            fixed_param_count: 0,
            fingerprint: String::new(),
        };

        ctx.add_function("test_func".to_string(), func_abi.clone());
        assert!(ctx.get_function("test_func").is_some());
        assert!(ctx.get_function("nonexistent").is_none());
    }

    #[test]
    fn test_struct_abi_compute_size() {
        let abi = StructAbi {
            name: Some("TestStruct".to_string()),
            size: 16,
            align: 8,
            is_packed: false,
            pack_align: None,
            fields: vec![
                FieldAbi {
                    name: "a".to_string(),
                    offset: 0,
                    size: 4,
                    align: 4,
                    bitfield: None,
                },
                FieldAbi {
                    name: "b".to_string(),
                    offset: 8,
                    size: 8,
                    align: 8,
                    bitfield: None,
                },
            ],
            fingerprint: String::new(),
        };

        assert_eq!(abi.size, 16);
        assert_eq!(abi.align, 8);
    }

    #[test]
    fn test_bitfield_abi_creation() {
        let bf = BitfieldAbi {
            container_offset: 0,
            bit_offset: 0,
            bit_width: 4,
            storage_type: TypeRef(4),
            is_signed: false,
        };

        assert_eq!(bf.bit_width, 4);
        assert!(!bf.is_signed);
    }

    #[test]
    fn test_callback_panic_policy_variants() {
        assert_eq!(
            std::mem::discriminant(&CallbackPanicPolicy::NoPanic),
            std::mem::discriminant(&CallbackPanicPolicy::NoPanic)
        );
        assert_ne!(
            std::mem::discriminant(&CallbackPanicPolicy::NoPanic),
            std::mem::discriminant(&CallbackPanicPolicy::Catch)
        );
    }

    #[test]
    fn test_error_reporting_variants() {
        assert_eq!(
            std::mem::discriminant(&ErrorReporting::Negative),
            std::mem::discriminant(&ErrorReporting::Negative)
        );
        assert_ne!(
            std::mem::discriminant(&ErrorReporting::Negative),
            std::mem::discriminant(&ErrorReporting::ChStatus)
        );
    }

    #[test]
    fn test_cabi_extractor_function() {
        let extractor = CAbiExtractor::new("x86_64-unknown-linux-gnu");

        let decl = FunctionDecl {
            id: DeclId(0),
            name: "my_func".to_string(),
            linkage: Linkage::External,
            storage_class: StorageClass::None,
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
            is_definition: true,
            is_inline: false,
            has_body: true,
        };

        let abi = extractor.extract_function(&decl).unwrap();
        assert_eq!(abi.name, "my_func");
        assert_eq!(abi.cconv, CCallingConvention::Cdecl);
        assert!(!abi.is_varargs);
    }

    #[test]
    fn test_cabi_extractor_struct() {
        let extractor = CAbiExtractor::new("x86_64-unknown-linux-gnu");

        let decl = StructDecl {
            id: DeclId(0),
            name: Some("Point".to_string()),
            fields: vec![
                FieldDecl {
                    name: "x".to_string(),
                    typ: TypeRef(4),
                    bitfield_width: None,
                    offset: 0,
                    size: 4,
                    align: 4,
                },
                FieldDecl {
                    name: "y".to_string(),
                    typ: TypeRef(4),
                    bitfield_width: None,
                    offset: 4,
                    size: 4,
                    align: 4,
                },
            ],
            is_packed: false,
            pack_align: None,
            is_incomplete: false,
            source_span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
        };

        let abi = extractor.extract_struct(&decl).unwrap();
        assert_eq!(abi.name, Some("Point".to_string()));
        assert_eq!(abi.size, 8);
        assert_eq!(abi.fields.len(), 2);
    }
}
