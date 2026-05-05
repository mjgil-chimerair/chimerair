//! `chimera-component` - Component identity and specification types for ChimeraIR.
//!
//! This crate provides the foundational types for ChimeraIR's component-based model:
//! - `ComponentId` - stable identifiers for build components
//! - `ComponentKind` - kinds of components (cargo-package, zig-exe, c-source, etc.)
//! - `Language` - supported languages (Rust, Zig, C)
//! - `ComponentSpec` - complete component definition
//! - `TargetSpec` - target triple and features
//! - `ProfileSpec` - optimization level, debug info, etc.
//! - `ToolchainSpec` - toolchain overrides
//! - `ModuleMap` - named modules for Zig/C
//! - `ImportMap` - import path mappings
//! - `ComponentGraph` - build graph with nodes and edges

mod graph;

pub use graph::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// A stable, unique identifier for a component in the build graph.
///
/// ComponentIds are used to reference components in ABI edges, build graph nodes,
/// and artifact envelopes. They must be unique within a single manifest.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(String);

impl ComponentId {
    /// Create a new ComponentId from a string.
    ///
    /// # Panics
    ///
    /// Panics if the string contains invalid characters (whitespace, newlines).
    pub fn new(id: impl Into<String>) -> Self {
        let s = id.into();
        if s.contains(' ') || s.contains('\n') || s.contains('\t') {
            panic!("ComponentId cannot contain whitespace: {:?}", s);
        }
        ComponentId(s)
    }

    /// Parse a ComponentId from a string, returning None if invalid.
    pub fn parse(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.contains(' ') || s.contains('\n') || s.contains('\t') {
            None
        } else {
            Some(ComponentId(s))
        }
    }

    /// Get the underlying string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ComponentId {
    fn from(s: String) -> Self {
        ComponentId::new(s)
    }
}

impl From<&str> for ComponentId {
    fn from(s: &str) -> Self {
        ComponentId::new(s)
    }
}

/// The kind of a component, indicating what type of build entity it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentKind {
    /// A Rust Cargo package or workspace member
    CargoPackage,
    /// A Zig executable
    ZigExe,
    /// A Zig library
    ZigLib,
    /// A C translation unit group
    CSource,
    /// A prebuilt native static or shared library
    PrebuiltNative,
    /// An already-produced ChimeraIR module (already lowered, for linking/merging)
    ChimeraModule,
    /// A Rust component that lowers to ChimeraIR as its primary output
    /// (emits .chimera/.chir without going through native archive first)
    RustChimeraComponent,
    /// A Zig component that lowers to ChimeraIR as its primary output
    /// (emits .chimera/.chir without going through native archive first)
    ZigChimeraComponent,
    /// A C component that lowers to ChimeraIR as its primary output
    /// (emits .chimera/.chir without going through native object first)
    CChimeraComponent,
}

impl ComponentKind {
    /// Returns the language associated with this component kind.
    pub fn language(self) -> Language {
        match self {
            ComponentKind::CargoPackage => Language::Rust,
            ComponentKind::ZigExe | ComponentKind::ZigLib => Language::Zig,
            ComponentKind::CSource => Language::C,
            ComponentKind::PrebuiltNative | ComponentKind::ChimeraModule => Language::Unknown,
            ComponentKind::RustChimeraComponent => Language::Rust,
            ComponentKind::ZigChimeraComponent => Language::Zig,
            ComponentKind::CChimeraComponent => Language::C,
        }
    }

    /// Check if this kind is compatible with the given language.
    pub fn is_compatible_with(self, lang: Language) -> bool {
        match self {
            ComponentKind::CargoPackage => lang == Language::Rust,
            ComponentKind::ZigExe | ComponentKind::ZigLib => lang == Language::Zig,
            ComponentKind::CSource => lang == Language::C,
            ComponentKind::PrebuiltNative | ComponentKind::ChimeraModule => true,
            ComponentKind::RustChimeraComponent => lang == Language::Rust,
            ComponentKind::ZigChimeraComponent => lang == Language::Zig,
            ComponentKind::CChimeraComponent => lang == Language::C,
        }
    }

    /// Check if this component kind primarily produces ChimeraIR as output.
    pub fn is_chimera_ir_primary(self) -> bool {
        matches!(
            self,
            ComponentKind::ChimeraModule
                | ComponentKind::RustChimeraComponent
                | ComponentKind::ZigChimeraComponent
                | ComponentKind::CChimeraComponent
        )
    }

    /// Returns the preferred component kind for unified lowering mode.
    ///
    /// This lets the build planner promote native-oriented component kinds
    /// into ChimeraIR-producing kinds without hardcoding the graph to a
    /// fixed set of languages. Languages without a unified-lowering kind yet
    /// return `None` and stay on their native path.
    pub fn unified_lowering_variant(self) -> Option<Self> {
        match self {
            ComponentKind::CargoPackage => Some(ComponentKind::RustChimeraComponent),
            ComponentKind::ZigExe | ComponentKind::ZigLib => {
                Some(ComponentKind::ZigChimeraComponent)
            }
            ComponentKind::CSource => Some(ComponentKind::CChimeraComponent),
            ComponentKind::ChimeraModule
            | ComponentKind::RustChimeraComponent
            | ComponentKind::ZigChimeraComponent
            | ComponentKind::CChimeraComponent => Some(self),
            ComponentKind::PrebuiltNative => None,
        }
    }
}

impl fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentKind::CargoPackage => write!(f, "cargo-package"),
            ComponentKind::ZigExe => write!(f, "zig-exe"),
            ComponentKind::ZigLib => write!(f, "zig-lib"),
            ComponentKind::CSource => write!(f, "c-source"),
            ComponentKind::PrebuiltNative => write!(f, "prebuilt-native"),
            ComponentKind::ChimeraModule => write!(f, "chimera-module"),
            ComponentKind::RustChimeraComponent => write!(f, "rust-chimera-component"),
            ComponentKind::ZigChimeraComponent => write!(f, "zig-chimera-component"),
            ComponentKind::CChimeraComponent => write!(f, "c-chimera-component"),
        }
    }
}

impl std::str::FromStr for ComponentKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cargo-package" => Ok(ComponentKind::CargoPackage),
            "zig-exe" => Ok(ComponentKind::ZigExe),
            "zig-lib" => Ok(ComponentKind::ZigLib),
            "c-source" => Ok(ComponentKind::CSource),
            "prebuilt-native" => Ok(ComponentKind::PrebuiltNative),
            "chimera-module" => Ok(ComponentKind::ChimeraModule),
            "rust-chimera-component" => Ok(ComponentKind::RustChimeraComponent),
            "zig-chimera-component" => Ok(ComponentKind::ZigChimeraComponent),
            "c-chimera-component" => Ok(ComponentKind::CChimeraComponent),
            _ => Err(format!("unknown component kind: {}", s)),
        }
    }
}

/// Supported programming languages in ChimeraIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Zig,
    C,
    /// Unknown language (used for prebuilt-native, chimera-module)
    Unknown,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::Zig => write!(f, "zig"),
            Language::C => write!(f, "c"),
            Language::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "zig" => Ok(Language::Zig),
            "c" => Ok(Language::C),
            _ => Err(format!("unknown language: {}", s)),
        }
    }
}

/// Target specification for a component's build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSpec {
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub triple: String,
    /// Target-specific features
    #[serde(default)]
    pub features: Vec<String>,
}

impl TargetSpec {
    /// Create a new target specification.
    pub fn new(triple: impl Into<String>) -> Self {
        TargetSpec {
            triple: triple.into(),
            features: Vec::new(),
        }
    }

    /// Create a new target specification with features.
    pub fn with_features(triple: impl Into<String>, features: Vec<String>) -> Self {
        TargetSpec {
            triple: triple.into(),
            features,
        }
    }
}

impl Default for TargetSpec {
    fn default() -> Self {
        TargetSpec::new("x86_64-unknown-linux-gnu")
    }
}

impl Hash for TargetSpec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.triple.hash(state);
        self.features.hash(state);
    }
}

impl TargetSpec {
    /// Compute a deterministic identity hash for cache keying.
    pub fn identity_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Build profile specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSpec {
    /// Optimization level (0-3)
    #[serde(default)]
    pub opt_level: u8,
    /// Include debug info
    #[serde(default)]
    pub debug: bool,
    /// LTO mode
    #[serde(default)]
    pub lto: bool,
}

impl Default for ProfileSpec {
    fn default() -> Self {
        ProfileSpec {
            opt_level: 3,
            debug: false,
            lto: false,
        }
    }
}

impl Hash for ProfileSpec {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.opt_level.hash(state);
        self.debug.hash(state);
        self.lto.hash(state);
    }
}

impl ProfileSpec {
    /// Compute a deterministic identity hash for cache keying.
    pub fn identity_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Toolchain specification for overriding default tools.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolchainSpec {
    /// Rust toolchain identifier (e.g., "nightly-2024-01-01")
    #[serde(default)]
    pub rust: Option<String>,
    /// C compiler identifier (e.g., "gcc-13", "clang-16")
    #[serde(default)]
    pub c: Option<String>,
    /// Zig compiler version (e.g., "0.14.0")
    #[serde(default)]
    pub zig: Option<String>,
}

impl Default for ToolchainSpec {
    fn default() -> Self {
        ToolchainSpec {
            rust: None,
            c: None,
            zig: None,
        }
    }
}

/// A named module in a component (for Zig and C).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedModule {
    /// Module name
    pub name: String,
    /// Path to the module source
    pub path: PathBuf,
}

impl NamedModule {
    /// Create a new named module.
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        NamedModule {
            name: name.into(),
            path,
        }
    }
}

/// Module map: named modules for a component.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleMap {
    /// Named modules
    #[serde(default)]
    pub modules: Vec<NamedModule>,
}

impl ModuleMap {
    /// Create a new empty module map.
    pub fn new() -> Self {
        ModuleMap {
            modules: Vec::new(),
        }
    }

    /// Add a module to the map.
    pub fn add_module(&mut self, name: impl Into<String>, path: PathBuf) {
        self.modules.push(NamedModule::new(name, path));
    }

    /// Get a module by name.
    pub fn get(&self, name: &str) -> Option<&NamedModule> {
        self.modules.iter().find(|m| m.name == name)
    }
}

/// Import map: mappings from import paths to actual paths.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMap {
    /// Import mappings
    #[serde(default)]
    pub mappings: Vec<(String, PathBuf)>,
}

impl ImportMap {
    /// Create a new empty import map.
    pub fn new() -> Self {
        ImportMap {
            mappings: Vec::new(),
        }
    }

    /// Add a mapping.
    pub fn add_mapping(&mut self, from: impl Into<String>, to: PathBuf) {
        self.mappings.push((from.into(), to));
    }

    /// Get the resolved path for an import.
    pub fn resolve(&self, import: &str) -> Option<&PathBuf> {
        self.mappings
            .iter()
            .find(|(k, _)| k == import)
            .map(|(_, v)| v)
    }
}

/// Crate type for Rust components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrateType {
    Lib,
    Bin,
    Staticlib,
    Cdylib,
    Rlib,
    ProcMacro,
}

impl fmt::Display for CrateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrateType::Lib => write!(f, "lib"),
            CrateType::Bin => write!(f, "bin"),
            CrateType::Staticlib => write!(f, "staticlib"),
            CrateType::Cdylib => write!(f, "cdylib"),
            CrateType::Rlib => write!(f, "rlib"),
            CrateType::ProcMacro => write!(f, "proc-macro"),
        }
    }
}

impl std::str::FromStr for CrateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lib" => Ok(CrateType::Lib),
            "bin" => Ok(CrateType::Bin),
            "staticlib" => Ok(CrateType::Staticlib),
            "cdylib" => Ok(CrateType::Cdylib),
            "rlib" => Ok(CrateType::Rlib),
            "proc-macro" => Ok(CrateType::ProcMacro),
            _ => Err(format!("unknown crate type: {}", s)),
        }
    }
}

/// Panic policy for Rust components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PanicPolicy {
    Unwind,
    Abort,
}

impl Default for PanicPolicy {
    fn default() -> Self {
        PanicPolicy::Abort
    }
}

/// An exported or imported symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Optional type signature (mangled or unmangled)
    #[serde(default)]
    pub signature: Option<String>,
    /// Calling convention for this symbol (Task 29)
    #[serde(default)]
    pub calling_convention: Option<CallingConvention>,
}

impl Symbol {
    /// Create a new symbol.
    pub fn new(name: impl Into<String>) -> Self {
        Symbol {
            name: name.into(),
            signature: None,
            calling_convention: None,
        }
    }

    /// Create a symbol with a signature.
    pub fn with_signature(name: impl Into<String>, sig: impl Into<String>) -> Self {
        Symbol {
            name: name.into(),
            signature: Some(sig.into()),
            calling_convention: None,
        }
    }

    /// Create a symbol with a calling convention.
    pub fn with_calling_convention(name: impl Into<String>, cc: CallingConvention) -> Self {
        Symbol {
            name: name.into(),
            signature: None,
            calling_convention: Some(cc),
        }
    }

    /// Set the calling convention from a source ABI string.
    pub fn set_calling_convention_from_abi(&mut self, abi: &str) {
        self.calling_convention = Some(CallingConvention::normalize(abi));
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref sig) = self.signature {
            write!(f, ": {}", sig)?;
        }
        Ok(())
    }
}

/// A complete component specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Unique component identifier
    pub id: ComponentId,
    /// Component language
    pub language: Language,
    /// Component kind
    pub kind: ComponentKind,
    /// Root sources or manifests
    #[serde(default)]
    pub roots: Vec<PathBuf>,
    /// Path to Cargo.toml (for cargo-package kind)
    #[serde(default)]
    pub manifest: Option<PathBuf>,
    /// Package name
    #[serde(default)]
    pub package: Option<String>,
    /// Crate types (for Rust components)
    #[serde(default)]
    pub crate_types: Vec<CrateType>,
    /// Enabled features (for Rust components)
    #[serde(default)]
    pub features: Vec<String>,
    /// Panic policy (for Rust components)
    #[serde(default)]
    pub panic_policy: Option<PanicPolicy>,
    /// Target specification
    #[serde(default)]
    pub target: Option<TargetSpec>,
    /// Profile specification
    #[serde(default)]
    pub profile: Option<ProfileSpec>,
    /// Exported symbols
    #[serde(default)]
    pub exported_symbols: Vec<Symbol>,
    /// Imported symbols
    #[serde(default)]
    pub imported_symbols: Vec<Symbol>,
    /// Preferred executable entry symbol for unified executable emission.
    #[serde(default)]
    pub entry_symbol: Option<String>,
    /// Optional unified entry builtin contract for entry-wrapper bridging.
    #[serde(default)]
    pub unified_entry_builtin: Option<String>,
    /// Module map (for Zig/C components)
    #[serde(default)]
    pub module_map: ModuleMap,
    /// Import map
    #[serde(default)]
    pub import_map: ImportMap,
    /// Include directories (for C components)
    #[serde(default)]
    pub include_dirs: Vec<PathBuf>,
    /// Preprocessor defines
    #[serde(default)]
    pub defines: Vec<(String, Option<String>)>,
}

impl ComponentSpec {
    /// Create a new component specification.
    pub fn new(id: ComponentId, language: Language, kind: ComponentKind) -> Self {
        ComponentSpec {
            id,
            language,
            kind,
            roots: Vec::new(),
            manifest: None,
            package: None,
            crate_types: Vec::new(),
            features: Vec::new(),
            panic_policy: None,
            target: None,
            profile: None,
            exported_symbols: Vec::new(),
            imported_symbols: Vec::new(),
            entry_symbol: None,
            unified_entry_builtin: None,
            module_map: ModuleMap::new(),
            import_map: ImportMap::new(),
            include_dirs: Vec::new(),
            defines: Vec::new(),
        }
    }

    /// Add a root source file or manifest.
    pub fn add_root(&mut self, root: PathBuf) {
        self.roots.push(root);
    }

    /// Set the manifest path.
    pub fn set_manifest(&mut self, manifest: PathBuf) {
        self.manifest = Some(manifest);
    }

    /// Set the package name.
    pub fn set_package(&mut self, name: impl Into<String>) {
        self.package = Some(name.into());
    }

    /// Add an exported symbol.
    pub fn add_exported_symbol(&mut self, symbol: Symbol) {
        self.exported_symbols.push(symbol);
    }

    /// Add an imported symbol.
    pub fn add_imported_symbol(&mut self, symbol: Symbol) {
        self.imported_symbols.push(symbol);
    }

    /// Set the preferred executable entry symbol for this component.
    pub fn set_entry_symbol(&mut self, symbol: impl Into<String>) {
        self.entry_symbol = Some(symbol.into());
    }

    /// Set the unified entry builtin contract for this component.
    pub fn set_unified_entry_builtin(&mut self, builtin: impl Into<String>) {
        self.unified_entry_builtin = Some(builtin.into());
    }
}

/// Link mode for ABI edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkMode {
    /// Provider participates in native link
    DirectLink,
    /// Provider static archive participates in native link
    StaticLink,
    /// Provider shared library linked at compile time
    DynamicLink,
    /// Provider cdylib packaged, loaded at runtime via dlopen
    RuntimeDlopen,
    /// Chimera generates a wrapper from provider's .chmeta contract
    GeneratedWrapper,
}

impl Default for LinkMode {
    fn default() -> Self {
        LinkMode::DirectLink
    }
}

impl fmt::Display for LinkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkMode::DirectLink => write!(f, "direct-link"),
            LinkMode::StaticLink => write!(f, "static-link"),
            LinkMode::DynamicLink => write!(f, "dynamic-link"),
            LinkMode::RuntimeDlopen => write!(f, "runtime-dlopen"),
            LinkMode::GeneratedWrapper => write!(f, "generated-wrapper"),
        }
    }
}

impl std::str::FromStr for LinkMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "direct-link" => Ok(LinkMode::DirectLink),
            "static-link" => Ok(LinkMode::StaticLink),
            "dynamic-link" => Ok(LinkMode::DynamicLink),
            "runtime-dlopen" => Ok(LinkMode::RuntimeDlopen),
            "generated-wrapper" => Ok(LinkMode::GeneratedWrapper),
            _ => Err(format!("unknown link mode: {}", s)),
        }
    }
}

/// Wrapper policy for ABI edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WrapperPolicy {
    Auto,
    C,
    Rust,
    Zig,
    None,
}

impl Default for WrapperPolicy {
    fn default() -> Self {
        WrapperPolicy::Auto
    }
}

/// Canonical calling conventions supported by ChimeraIR.
///
/// This enum normalizes calling conventions from different source languages
/// (Rust, Zig, C) into a unified model for cross-language ABI compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CallingConvention {
    /// C calling convention (default for FFI)
    C,
    /// Rust unsafe extern ABI (same as C but with Rust name mangling rules)
    Rust,
    /// Zig's C ABI (same representation as C)
    Zig,
    /// Cold calling convention (unlikely to be called)
    Cold,
    /// Fast calling convention (leaf functions, no stack args)
    Fast,
    /// Standard calling convention (callee-saved regs)
    Std,
    /// Vector calling convention (SIMD arguments)
    Vector,
}

impl CallingConvention {
    /// Normalize a source-language ABI string to a canonical calling convention.
    ///
    /// Rust and Zig both support multiple ABIs, but they map to a common
    /// set of representations at the ChimeraIR layer.
    pub fn normalize(abi: &str) -> Self {
        let lower = abi.to_lowercase();
        match lower.as_str() {
            // Zig ABIs (prefixed with dot)
            ".c" => CallingConvention::C,
            ".opaque" => CallingConvention::Zig,
            ".threadlocal" => CallingConvention::Zig,

            // C ABIs (exact forms)
            "cdecl" | "__cdecl" => CallingConvention::C,
            "stdcall" | "__stdcall" => CallingConvention::Std,
            "fastcall" | "__fastcall" => CallingConvention::Fast,
            "vectorcall" | "__vectorcall" => CallingConvention::Vector,

            // Rust ABIs (explicit forms)
            "rust" | "rust-unwind" => CallingConvention::Rust,
            "expose" => CallingConvention::Zig,

            // Generic "c" (lowercase without prefix)
            "c" => CallingConvention::C,

            // Default unknown to C
            _ => CallingConvention::C,
        }
    }

    /// Check if this calling convention is ABI-compatible with another.
    pub fn is_compatible_with(self, other: CallingConvention) -> bool {
        if self == other {
            return true;
        }
        match (self, other) {
            (CallingConvention::C, CallingConvention::Rust) => true,
            (CallingConvention::Rust, CallingConvention::C) => true,
            (CallingConvention::C, CallingConvention::Zig) => true,
            (CallingConvention::Zig, CallingConvention::C) => true,
            (CallingConvention::Rust, CallingConvention::Zig) => true,
            (CallingConvention::Zig, CallingConvention::Rust) => true,
            _ => false,
        }
    }

    /// Get the canonical string representation for LLVM.
    pub fn as_llvm_str(self) -> &'static str {
        match self {
            CallingConvention::C => "ccc",
            CallingConvention::Rust => "rust",
            CallingConvention::Zig => "zig",
            CallingConvention::Cold => "cold",
            CallingConvention::Fast => "fastcc",
            CallingConvention::Std => "stdcc",
            CallingConvention::Vector => "vectorcall",
        }
    }
}

impl Default for CallingConvention {
    fn default() -> Self {
        CallingConvention::C
    }
}

impl fmt::Display for CallingConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallingConvention::C => write!(f, "C"),
            CallingConvention::Rust => write!(f, "Rust"),
            CallingConvention::Zig => write!(f, "Zig"),
            CallingConvention::Cold => write!(f, "Cold"),
            CallingConvention::Fast => write!(f, "Fast"),
            CallingConvention::Std => write!(f, "Std"),
            CallingConvention::Vector => write!(f, "Vector"),
        }
    }
}

impl std::str::FromStr for CallingConvention {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "c" => Ok(CallingConvention::C),
            "rust" => Ok(CallingConvention::Rust),
            "zig" => Ok(CallingConvention::Zig),
            "cold" => Ok(CallingConvention::Cold),
            "fast" => Ok(CallingConvention::Fast),
            "std" => Ok(CallingConvention::Std),
            "vector" => Ok(CallingConvention::Vector),
            _ => Err(format!("unknown calling convention: {}", s)),
        }
    }
}

/// Proof policy for ABI edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProofPolicy {
    Required,
    Optional,
    Disabled,
}

impl Default for ProofPolicy {
    fn default() -> Self {
        ProofPolicy::Required
    }
}

/// An ABI edge between two components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEdge {
    /// Consumer component ID
    pub consumer: ComponentId,
    /// Provider component ID
    pub provider: ComponentId,
    /// Symbols provided by the edge
    #[serde(default)]
    pub symbols: Vec<Symbol>,
    /// Link mode
    #[serde(default)]
    pub mode: LinkMode,
    /// Wrapper policy
    #[serde(default)]
    pub wrapper: WrapperPolicy,
    /// Proof policy
    #[serde(default)]
    pub proof: ProofPolicy,
    /// Runtime argument (for runtime-dlopen mode)
    #[serde(default)]
    pub runtime_arg: Option<String>,
    /// Visibility (pub or pub(crate))
    #[serde(default)]
    pub visibility: String,
    /// Failure policy
    #[serde(default)]
    pub failure_policy: String,
}

impl AbiEdge {
    /// Create a new ABI edge.
    pub fn new(consumer: ComponentId, provider: ComponentId) -> Self {
        AbiEdge {
            consumer,
            provider,
            symbols: Vec::new(),
            mode: LinkMode::DirectLink,
            wrapper: WrapperPolicy::Auto,
            proof: ProofPolicy::Required,
            runtime_arg: None,
            visibility: "pub".to_string(),
            failure_policy: "error".to_string(),
        }
    }

    /// Add symbols to the edge.
    pub fn add_symbols(&mut self, symbols: Vec<Symbol>) {
        self.symbols.extend(symbols);
    }

    /// Set the link mode.
    pub fn set_mode(&mut self, mode: LinkMode) {
        self.mode = mode;
    }
}

/// Errors that can occur in component operations.
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("invalid component ID: {0}")]
    InvalidId(String),
    #[error("duplicate component ID: {0}")]
    DuplicateId(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("incompatible language and kind: {0}")]
    IncompatibleKind(String),
    #[error("cycle detected in component graph")]
    CycleDetected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_creation() {
        let id = ComponentId::new("my_component");
        assert_eq!(id.as_str(), "my_component");
    }

    #[test]
    fn test_component_id_display() {
        let id = ComponentId::new("test");
        assert_eq!(format!("{}", id), "test");
    }

    #[test]
    fn test_component_id_parse_valid() {
        let id = ComponentId::parse("valid_id");
        assert!(id.is_some());
        assert_eq!(id.unwrap().as_str(), "valid_id");
    }

    #[test]
    fn test_component_id_parse_invalid() {
        let id = ComponentId::parse("invalid id");
        assert!(id.is_none());
    }

    #[test]
    fn test_component_id_from_string() {
        let id: ComponentId = "from_string".to_string().into();
        assert_eq!(id.as_str(), "from_string");
    }

    #[test]
    fn test_component_kind_language() {
        assert_eq!(ComponentKind::CargoPackage.language(), Language::Rust);
        assert_eq!(ComponentKind::ZigExe.language(), Language::Zig);
        assert_eq!(ComponentKind::CSource.language(), Language::C);
    }

    #[test]
    fn test_component_kind_parse() {
        let kind: ComponentKind = "zig-exe".parse().unwrap();
        assert_eq!(kind, ComponentKind::ZigExe);
    }

    #[test]
    fn test_component_kind_is_compatible_with() {
        assert!(ComponentKind::CargoPackage.is_compatible_with(Language::Rust));
        assert!(!ComponentKind::CargoPackage.is_compatible_with(Language::Zig));
    }

    #[test]
    fn test_language_parse() {
        let lang: Language = "rust".parse().unwrap();
        assert_eq!(lang, Language::Rust);
    }

    #[test]
    fn test_target_spec_default() {
        let target = TargetSpec::default();
        assert_eq!(target.triple, "x86_64-unknown-linux-gnu");
        assert!(target.features.is_empty());
    }

    #[test]
    fn test_target_spec_with_features() {
        let target = TargetSpec::with_features("aarch64-apple-darwin", vec!["neon".to_string()]);
        assert_eq!(target.triple, "aarch64-apple-darwin");
        assert_eq!(target.features.len(), 1);
    }

    #[test]
    fn test_profile_spec_default() {
        let profile = ProfileSpec::default();
        assert_eq!(profile.opt_level, 3);
        assert!(!profile.debug);
        assert!(!profile.lto);
    }

    #[test]
    fn test_module_map() {
        let mut map = ModuleMap::new();
        map.add_module("ffi", PathBuf::from("src/ffi.zig"));
        map.add_module("source", PathBuf::from("src/source.zig"));

        assert!(map.get("ffi").is_some());
        assert!(map.get("missing").is_none());
    }

    #[test]
    fn test_import_map() {
        let mut map = ImportMap::new();
        map.add_mapping("std", PathBuf::from("vendor/std"));

        assert_eq!(map.resolve("std").unwrap(), &PathBuf::from("vendor/std"));
        assert!(map.resolve("missing").is_none());
    }

    #[test]
    fn test_crate_type_parse() {
        let ct: CrateType = "staticlib".parse().unwrap();
        assert_eq!(ct, CrateType::Staticlib);
    }

    #[test]
    fn test_symbol_creation() {
        let sym = Symbol::new("my_function");
        assert_eq!(sym.name, "my_function");
        assert!(sym.signature.is_none());
    }

    #[test]
    fn test_symbol_with_signature() {
        let sym = Symbol::with_signature("add", "fn(i32, i32) -> i32");
        assert_eq!(sym.name, "add");
        assert!(sym.signature.is_some());
    }

    #[test]
    fn test_symbol_with_calling_convention() {
        let sym = Symbol::with_calling_convention("foo", CallingConvention::C);
        assert_eq!(sym.name, "foo");
        assert!(sym.calling_convention.is_some());
        assert_eq!(sym.calling_convention.unwrap(), CallingConvention::C);
    }

    #[test]
    fn test_symbol_set_calling_convention_from_abi() {
        let mut sym = Symbol::new("bar");
        sym.set_calling_convention_from_abi("C");
        assert!(sym.calling_convention.is_some());
        assert_eq!(sym.calling_convention.unwrap(), CallingConvention::C);
    }

    #[test]
    fn test_calling_convention_normalize_rust() {
        assert_eq!(CallingConvention::normalize("c"), CallingConvention::C);
        assert_eq!(
            CallingConvention::normalize("rust"),
            CallingConvention::Rust
        );
        assert_eq!(
            CallingConvention::normalize("rust-unwind"),
            CallingConvention::Rust
        );
    }

    #[test]
    fn test_calling_convention_normalize_zig() {
        assert_eq!(CallingConvention::normalize(".c"), CallingConvention::C);
        assert_eq!(CallingConvention::normalize("c"), CallingConvention::C);
        assert_eq!(
            CallingConvention::normalize(".opaque"),
            CallingConvention::Zig
        );
        assert_eq!(
            CallingConvention::normalize(".threadlocal"),
            CallingConvention::Zig
        );
    }

    #[test]
    fn test_calling_convention_normalize_c() {
        assert_eq!(CallingConvention::normalize("cdecl"), CallingConvention::C);
        assert_eq!(
            CallingConvention::normalize("__cdecl"),
            CallingConvention::C
        );
        assert_eq!(
            CallingConvention::normalize("stdcall"),
            CallingConvention::Std
        );
        assert_eq!(
            CallingConvention::normalize("fastcall"),
            CallingConvention::Fast
        );
        assert_eq!(
            CallingConvention::normalize("vectorcall"),
            CallingConvention::Vector
        );
    }

    #[test]
    fn test_calling_convention_is_compatible_with() {
        // Same is compatible
        assert!(CallingConvention::C.is_compatible_with(CallingConvention::C));
        assert!(CallingConvention::Rust.is_compatible_with(CallingConvention::Rust));
        assert!(CallingConvention::Zig.is_compatible_with(CallingConvention::Zig));

        // C, Rust, Zig are mutually compatible (same representation)
        assert!(CallingConvention::C.is_compatible_with(CallingConvention::Rust));
        assert!(CallingConvention::Rust.is_compatible_with(CallingConvention::C));
        assert!(CallingConvention::C.is_compatible_with(CallingConvention::Zig));
        assert!(CallingConvention::Zig.is_compatible_with(CallingConvention::C));
        assert!(CallingConvention::Rust.is_compatible_with(CallingConvention::Zig));
        assert!(CallingConvention::Zig.is_compatible_with(CallingConvention::Rust));

        // Cold, Fast, Std, Vector are not compatible with C
        assert!(!CallingConvention::Cold.is_compatible_with(CallingConvention::C));
        assert!(!CallingConvention::Fast.is_compatible_with(CallingConvention::C));
        assert!(!CallingConvention::Std.is_compatible_with(CallingConvention::C));
        assert!(!CallingConvention::Vector.is_compatible_with(CallingConvention::C));
    }

    #[test]
    fn test_calling_convention_as_llvm_str() {
        assert_eq!(CallingConvention::C.as_llvm_str(), "ccc");
        assert_eq!(CallingConvention::Rust.as_llvm_str(), "rust");
        assert_eq!(CallingConvention::Zig.as_llvm_str(), "zig");
        assert_eq!(CallingConvention::Cold.as_llvm_str(), "cold");
        assert_eq!(CallingConvention::Fast.as_llvm_str(), "fastcc");
        assert_eq!(CallingConvention::Std.as_llvm_str(), "stdcc");
        assert_eq!(CallingConvention::Vector.as_llvm_str(), "vectorcall");
    }

    #[test]
    fn test_calling_convention_display() {
        assert_eq!(format!("{}", CallingConvention::C), "C");
        assert_eq!(format!("{}", CallingConvention::Rust), "Rust");
        assert_eq!(format!("{}", CallingConvention::Zig), "Zig");
    }

    #[test]
    fn test_calling_convention_parse() {
        let cc: CallingConvention = "C".parse().unwrap();
        assert_eq!(cc, CallingConvention::C);
        let cc: CallingConvention = "rust".parse().unwrap();
        assert_eq!(cc, CallingConvention::Rust);
        let cc: CallingConvention = "zig".parse().unwrap();
        assert_eq!(cc, CallingConvention::Zig);
        let cc: CallingConvention = "fast".parse().unwrap();
        assert_eq!(cc, CallingConvention::Fast);
    }

    #[test]
    fn test_link_mode_parse() {
        let mode: LinkMode = "runtime-dlopen".parse().unwrap();
        assert_eq!(mode, LinkMode::RuntimeDlopen);
    }

    #[test]
    fn test_target_spec_identity_hash_deterministic() {
        let t1 = TargetSpec::new("x86_64-unknown-linux-gnu");
        let t2 = TargetSpec::new("x86_64-unknown-linux-gnu");
        assert_eq!(t1.identity_hash(), t2.identity_hash());
    }

    #[test]
    fn test_target_spec_identity_hash_differs() {
        let t1 = TargetSpec::new("x86_64-unknown-linux-gnu");
        let t2 = TargetSpec::new("aarch64-apple-darwin");
        assert_ne!(t1.identity_hash(), t2.identity_hash());
    }

    #[test]
    fn test_target_spec_identity_hash_with_features() {
        let t1 = TargetSpec::with_features("x86_64-unknown-linux-gnu", vec!["sse2".to_string()]);
        let t2 = TargetSpec::with_features("x86_64-unknown-linux-gnu", vec!["sse2".to_string()]);
        assert_eq!(t1.identity_hash(), t2.identity_hash());

        let t3 = TargetSpec::with_features("x86_64-unknown-linux-gnu", vec!["avx2".to_string()]);
        assert_ne!(t1.identity_hash(), t3.identity_hash());
    }

    #[test]
    fn test_profile_spec_identity_hash_deterministic() {
        let p1 = ProfileSpec::default();
        let p2 = ProfileSpec::default();
        assert_eq!(p1.identity_hash(), p2.identity_hash());
    }

    #[test]
    fn test_profile_spec_identity_hash_differs() {
        let p1 = ProfileSpec::default();
        let mut p2 = ProfileSpec::default();
        p2.opt_level = 0;
        assert_ne!(p1.identity_hash(), p2.identity_hash());
    }

    #[test]
    fn test_profile_spec_identity_hash_debug_flag() {
        let mut p1 = ProfileSpec::default();
        p1.debug = true;
        let mut p2 = ProfileSpec::default();
        p2.debug = false;
        assert_ne!(p1.identity_hash(), p2.identity_hash());
    }

    #[test]
    fn test_target_spec_identity_hash_features_order_matters() {
        let t1 = TargetSpec::with_features(
            "x86_64-unknown-linux-gnu",
            vec!["sse2".to_string(), "avx2".to_string()],
        );
        let t2 = TargetSpec::with_features(
            "x86_64-unknown-linux-gnu",
            vec!["avx2".to_string(), "sse2".to_string()],
        );
        assert_ne!(t1.identity_hash(), t2.identity_hash());
    }

    #[test]
    fn test_component_spec_builder() {
        let mut comp = ComponentSpec::new(
            ComponentId::new("my_rust_lib"),
            Language::Rust,
            ComponentKind::CargoPackage,
        );
        comp.set_manifest(PathBuf::from("Cargo.toml"));
        comp.set_package("my_lib");
        comp.add_exported_symbol(Symbol::new("public_fn"));
        comp.set_unified_entry_builtin("argv-entry-bridge");

        assert_eq!(comp.id.as_str(), "my_rust_lib");
        assert_eq!(comp.kind, ComponentKind::CargoPackage);
        assert!(comp.manifest.is_some());
        assert_eq!(comp.exported_symbols.len(), 1);
        assert_eq!(
            comp.unified_entry_builtin.as_deref(),
            Some("argv-entry-bridge")
        );
    }

    #[test]
    fn test_abi_edge_builder() {
        let mut edge = AbiEdge::new(ComponentId::new("consumer"), ComponentId::new("provider"));
        edge.add_symbols(vec![Symbol::new("fn1"), Symbol::new("fn2")]);
        edge.set_mode(LinkMode::RuntimeDlopen);

        assert_eq!(edge.consumer.as_str(), "consumer");
        assert_eq!(edge.provider.as_str(), "provider");
        assert_eq!(edge.symbols.len(), 2);
        assert_eq!(edge.mode, LinkMode::RuntimeDlopen);
    }

    #[test]
    fn test_toml_serde_roundtrip() {
        let id = ComponentId::new("test_id");
        let json = serde_json::to_string(&id).unwrap();
        let parsed: ComponentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_component_spec_serde() {
        let mut comp = ComponentSpec::new(
            ComponentId::new("test"),
            Language::Zig,
            ComponentKind::ZigExe,
        );
        comp.add_root(PathBuf::from("src/main.zig"));

        let json = serde_json::to_string(&comp).unwrap();
        let parsed: ComponentSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(comp.id, parsed.id);
        assert_eq!(comp.language, parsed.language);
        assert_eq!(comp.kind, parsed.kind);
    }

    #[test]
    fn test_rust_chimera_component_kind() {
        let kind = ComponentKind::RustChimeraComponent;
        assert_eq!(kind.language(), Language::Rust);
        assert!(kind.is_compatible_with(Language::Rust));
        assert!(!kind.is_compatible_with(Language::Zig));
        assert!(kind.is_chimera_ir_primary());
    }

    #[test]
    fn test_zig_chimera_component_kind() {
        let kind = ComponentKind::ZigChimeraComponent;
        assert_eq!(kind.language(), Language::Zig);
        assert!(kind.is_compatible_with(Language::Zig));
        assert!(!kind.is_compatible_with(Language::Rust));
        assert!(kind.is_chimera_ir_primary());
    }

    #[test]
    fn test_c_chimera_component_kind() {
        let kind = ComponentKind::CChimeraComponent;
        assert_eq!(kind.language(), Language::C);
        assert!(kind.is_compatible_with(Language::C));
        assert!(!kind.is_compatible_with(Language::Rust));
        assert!(kind.is_chimera_ir_primary());
    }

    #[test]
    fn test_chimera_module_is_chimera_ir_primary() {
        let kind = ComponentKind::ChimeraModule;
        assert!(kind.is_chimera_ir_primary());
        // ChimeraModule should still be compatible with any language
        assert!(kind.is_compatible_with(Language::Rust));
        assert!(kind.is_compatible_with(Language::Zig));
        assert!(kind.is_compatible_with(Language::C));
    }

    #[test]
    fn test_component_kind_parse_rust_chimera() {
        let kind: ComponentKind = "rust-chimera-component".parse().unwrap();
        assert_eq!(kind, ComponentKind::RustChimeraComponent);
    }

    #[test]
    fn test_component_kind_parse_zig_chimera() {
        let kind: ComponentKind = "zig-chimera-component".parse().unwrap();
        assert_eq!(kind, ComponentKind::ZigChimeraComponent);
    }

    #[test]
    fn test_component_kind_parse_c_chimera() {
        let kind: ComponentKind = "c-chimera-component".parse().unwrap();
        assert_eq!(kind, ComponentKind::CChimeraComponent);
    }

    #[test]
    fn test_component_kind_display_rust_chimera() {
        let kind = ComponentKind::RustChimeraComponent;
        assert_eq!(format!("{}", kind), "rust-chimera-component");
    }

    #[test]
    fn test_component_kind_display_zig_chimera() {
        let kind = ComponentKind::ZigChimeraComponent;
        assert_eq!(format!("{}", kind), "zig-chimera-component");
    }

    #[test]
    fn test_component_kind_display_c_chimera() {
        let kind = ComponentKind::CChimeraComponent;
        assert_eq!(format!("{}", kind), "c-chimera-component");
    }

    #[test]
    fn test_traditional_kinds_not_chimera_ir_primary() {
        assert!(!ComponentKind::CargoPackage.is_chimera_ir_primary());
        assert!(!ComponentKind::ZigExe.is_chimera_ir_primary());
        assert!(!ComponentKind::ZigLib.is_chimera_ir_primary());
        assert!(!ComponentKind::CSource.is_chimera_ir_primary());
        assert!(!ComponentKind::PrebuiltNative.is_chimera_ir_primary());
    }

    #[test]
    fn test_unified_lowering_variant_mapping() {
        assert_eq!(
            ComponentKind::CargoPackage.unified_lowering_variant(),
            Some(ComponentKind::RustChimeraComponent)
        );
        assert_eq!(
            ComponentKind::ZigLib.unified_lowering_variant(),
            Some(ComponentKind::ZigChimeraComponent)
        );
        assert_eq!(
            ComponentKind::CSource.unified_lowering_variant(),
            Some(ComponentKind::CChimeraComponent)
        );
        assert_eq!(
            ComponentKind::ChimeraModule.unified_lowering_variant(),
            Some(ComponentKind::ChimeraModule)
        );
        assert_eq!(
            ComponentKind::PrebuiltNative.unified_lowering_variant(),
            None
        );
    }
}
