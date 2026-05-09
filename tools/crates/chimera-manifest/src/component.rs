//! Component manifest types for ChimeraIR v0.2.
//!
//! This module provides types for the `[[components]]` section of Chimera.toml
//! which replaces `[[sources]]` as the primary build model.

use anyhow::Result;
use chimera_component::{
    AbiEdge, ComponentId, ComponentKind, CrateType, ImportMap, Language, LinkMode, ModuleMap,
    PanicPolicy, ProfileSpec, ProofPolicy, Symbol, TargetSpec, WrapperPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A component entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEntry {
    /// Unique component identifier
    pub id: String,
    /// Component language (rust, zig, c)
    pub language: String,
    /// Component kind (cargo-package, zig-exe, zig-lib, c-source, prebuilt-native, chimera-module)
    #[serde(default)]
    pub kind: Option<String>,
    /// Root sources or manifests
    #[serde(default)]
    pub roots: Vec<String>,
    /// Path to Cargo.toml (for cargo-package)
    #[serde(default)]
    pub manifest: Option<String>,
    /// Package name
    #[serde(default)]
    pub package: Option<String>,
    /// Crate types (for Rust)
    #[serde(default)]
    pub crate_types: Vec<String>,
    /// Enabled features (for Rust)
    #[serde(default)]
    pub features: Vec<String>,
    /// Panic policy (for Rust)
    #[serde(default)]
    pub panic_policy: Option<String>,
    /// Target triple
    #[serde(default)]
    pub target: Option<String>,
    /// Optimization level (0-3)
    #[serde(default)]
    pub opt_level: Option<u8>,
    /// Include debug info
    #[serde(default)]
    pub debug: Option<bool>,
    /// LTO mode
    #[serde(default)]
    pub lto: Option<bool>,
    /// Module map entries (for Zig/C)
    #[serde(default)]
    pub modules: Vec<ModuleEntry>,
    /// Import mappings
    #[serde(default)]
    pub imports: HashMap<String, String>,
    /// Include directories (for C)
    #[serde(default)]
    pub include_dirs: Vec<String>,
    /// Preprocessor defines
    #[serde(default)]
    pub defines: Vec<DefineEntry>,
    /// Exported symbols
    #[serde(default)]
    pub exported_symbols: Vec<String>,
    /// Imported symbols
    #[serde(default)]
    pub imported_symbols: Vec<String>,
    /// Preferred executable entry symbol for unified executable emission.
    #[serde(default)]
    pub entry_symbol: Option<String>,
    /// Optional unified entry builtin contract for entry-wrapper bridging.
    #[serde(default)]
    pub unified_entry_builtin: Option<String>,
}

/// A named module entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry {
    pub name: String,
    pub path: String,
}

/// A preprocessor define entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineEntry {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
}

/// Diagnostic codes for manifest validation errors.
pub mod diag {
    /// Component kind/language compatibility
    pub const KIND_LANG_MISMATCH: &str = "E101";
    /// Target triple inconsistency
    pub const TARGET_INCONSISTENT: &str = "E102";
    /// ABI edge references unknown component
    pub const ABI_COMPONENT_MISSING: &str = "E103";
    /// Runtime delivery rule violation
    pub const RUNTIME_DELIVERY_INVALID: &str = "E104";
    /// Wrapper/proof policy incompatibility
    pub const POLICY_INCOMPATIBLE: &str = "E105";
    /// Output kind incompatibility
    pub const OUTPUT_KIND_INCOMPATIBLE: &str = "E106";
    /// Crate type/target mismatch
    pub const CRATE_TYPE_TARGET_MISMATCH: &str = "E107";
    /// Missing required field
    pub const REQUIRED_FIELD_MISSING: &str = "E108";
    /// Symbol export/import mismatch
    pub const SYMBOL_MISMATCH: &str = "E109";
}

use diag::*;

/// An ABI edge entry in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEdgeEntry {
    /// Consumer component ID
    pub consumer: String,
    /// Provider component ID
    pub provider: String,
    /// Symbols provided by the edge
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Link mode (direct-link, static-link, dynamic-link, runtime-dlopen, generated-wrapper)
    #[serde(default)]
    pub mode: Option<String>,
    /// Wrapper policy (auto, c, rust, zig, none)
    #[serde(default)]
    pub wrapper: Option<String>,
    /// Proof policy (required, optional, disabled)
    #[serde(default)]
    pub proof: Option<String>,
    /// Runtime argument (for runtime-dlopen)
    #[serde(default)]
    pub runtime_arg: Option<String>,
    /// Visibility
    #[serde(default)]
    pub visibility: Option<String>,
    /// Failure policy
    #[serde(default)]
    pub failure_policy: Option<String>,
}

impl ComponentEntry {
    /// Validate the component entry.
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            anyhow::bail!("Component ID cannot be empty");
        }

        if self.id.contains(' ') || self.id.contains('\n') || self.id.contains('\t') {
            anyhow::bail!("Component ID cannot contain whitespace");
        }

        let lang = self.language.to_lowercase();
        if !["rust", "zig", "c", "unknown"].contains(&lang.as_str()) {
            anyhow::bail!(
                "Unknown component language: {}. Expected: rust, zig, c",
                self.language
            );
        }

        // Validate kind if provided
        if let Some(ref kind) = self.kind {
            let k = kind.to_lowercase();
            if ![
                "cargo-package",
                "zig-exe",
                "zig-lib",
                "c-source",
                "prebuilt-native",
                "chimera-module",
                "rust-chimera-component",
                "zig-chimera-component",
                "c-chimera-component",
            ]
            .contains(&k.as_str())
            {
                anyhow::bail!(
                    "Unknown component kind: {}. Expected: cargo-package, zig-exe, zig-lib, c-source, prebuilt-native, chimera-module, rust-chimera-component, zig-chimera-component, c-chimera-component",
                    kind
                );
            }
        }

        // Validate roots
        if self.roots.is_empty() && self.manifest.is_none() {
            anyhow::bail!(
                "Component '{}' must have at least one root source or manifest",
                self.id
            );
        }

        if let Some(ref entry_symbol) = self.entry_symbol {
            if entry_symbol.trim().is_empty() {
                anyhow::bail!("Component '{}' entry_symbol cannot be empty", self.id);
            }
        }

        if let Some(ref builtin) = self.unified_entry_builtin {
            let builtin = builtin.trim();
            if builtin.is_empty() {
                anyhow::bail!(
                    "Component '{}' unified_entry_builtin cannot be empty",
                    self.id
                );
            }
            if !["argv-entry-bridge"].contains(&builtin) {
                anyhow::bail!(
                    "Component '{}' unified_entry_builtin '{}' is not supported; expected: argv-entry-bridge",
                    self.id,
                    builtin
                );
            }
        }

        for root in &self.roots {
            if root.is_empty() {
                anyhow::bail!("Root path cannot be empty");
            }
        }

        // Validate crate_types if provided
        for ct in &self.crate_types {
            let ct_lower = ct.to_lowercase();
            if !["lib", "bin", "staticlib", "cdylib", "rlib", "proc-macro"]
                .contains(&ct_lower.as_str())
            {
                anyhow::bail!(
                    "Unknown crate type: {}. Expected: lib, bin, staticlib, cdylib, rlib, proc-macro",
                    ct
                );
            }
        }

        // Validate panic_policy if provided
        if let Some(ref pp) = self.panic_policy {
            let pp_lower = pp.to_lowercase();
            if !["unwind", "abort"].contains(&pp_lower.as_str()) {
                anyhow::bail!("Unknown panic policy: {}. Expected: unwind, abort", pp);
            }
        }

        Ok(())
    }

    /// Convert to a `ComponentSpec`.
    pub fn to_component_spec(&self) -> Result<chimera_component::ComponentSpec> {
        let id = ComponentId::new(&self.id);

        let lang: Language = self
            .language
            .to_lowercase()
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid language {}: {}", self.language, e))?;

        let kind = if let Some(ref k) = self.kind {
            k.parse()
                .map_err(|e| anyhow::anyhow!("invalid kind {}: {}", k, e))?
        } else {
            // Infer kind from language
            match lang {
                Language::Rust => ComponentKind::CargoPackage,
                Language::Zig => ComponentKind::ZigExe,
                Language::C => ComponentKind::CSource,
                Language::Unknown => ComponentKind::ChimeraModule,
            }
        };

        let mut spec = chimera_component::ComponentSpec::new(id.clone(), lang, kind);

        // Set roots
        for root in &self.roots {
            spec.add_root(PathBuf::from(root));
        }

        // Set manifest
        if let Some(ref manifest) = self.manifest {
            spec.set_manifest(PathBuf::from(manifest));
        }

        // Set package name
        if let Some(ref package) = self.package {
            spec.set_package(package);
        }

        // Set crate types
        for ct_str in &self.crate_types {
            if let Ok(ct) = ct_str.parse::<CrateType>() {
                spec.crate_types.push(ct);
            }
        }

        // Set features
        spec.features.clone_from(&self.features);

        // Set panic policy
        if let Some(ref pp) = self.panic_policy {
            let pp_lower = pp.to_lowercase();
            spec.panic_policy = Some(if pp_lower == "abort" {
                PanicPolicy::Abort
            } else {
                PanicPolicy::Unwind
            });
        }

        // Set target
        if let Some(ref triple) = self.target {
            spec.target = Some(TargetSpec::new(triple));
        }

        // Set profile
        if self.opt_level.is_some() || self.debug.is_some() || self.lto.is_some() {
            let mut profile = ProfileSpec::default();
            if let Some(level) = self.opt_level {
                profile.opt_level = level;
            }
            if let Some(debug) = self.debug {
                profile.debug = debug;
            }
            if let Some(lto) = self.lto {
                profile.lto = lto;
            }
            spec.profile = Some(profile);
        }

        // Set module map
        let mut module_map = ModuleMap::new();
        for module in &self.modules {
            module_map.add_module(&module.name, PathBuf::from(&module.path));
        }
        spec.module_map = module_map;

        // Set import map
        let mut import_map = ImportMap::new();
        for (from, to) in &self.imports {
            import_map.add_mapping(from, PathBuf::from(to));
        }
        spec.import_map = import_map;

        // Set include dirs
        for dir in &self.include_dirs {
            spec.include_dirs.push(PathBuf::from(dir));
        }

        // Set defines
        for define in &self.defines {
            spec.defines
                .push((define.name.clone(), define.value.clone()));
        }

        // Set exported symbols
        for sym_str in &self.exported_symbols {
            spec.add_exported_symbol(Symbol::new(sym_str));
        }

        // Set imported symbols
        for sym_str in &self.imported_symbols {
            spec.add_imported_symbol(Symbol::new(sym_str));
        }

        if let Some(ref entry_symbol) = self.entry_symbol {
            spec.set_entry_symbol(entry_symbol);
        }
        if let Some(ref builtin) = self.unified_entry_builtin {
            spec.set_unified_entry_builtin(builtin);
        }

        Ok(spec)
    }
}

impl AbiEdgeEntry {
    /// Validate the ABI edge entry.
    pub fn validate(&self) -> Result<()> {
        if self.consumer.is_empty() {
            anyhow::bail!("Consumer component ID cannot be empty");
        }

        if self.provider.is_empty() {
            anyhow::bail!("Provider component ID cannot be empty");
        }

        if self.consumer.contains(' ') || self.consumer.contains('\n') {
            anyhow::bail!("Consumer ID cannot contain whitespace");
        }

        if self.provider.contains(' ') || self.provider.contains('\n') {
            anyhow::bail!("Provider ID cannot contain whitespace");
        }

        // Validate mode if provided
        if let Some(ref mode) = self.mode {
            let mode_lower = mode.to_lowercase();
            if ![
                "direct-link",
                "static-link",
                "dynamic-link",
                "runtime-dlopen",
                "generated-wrapper",
            ]
            .contains(&mode_lower.as_str())
            {
                anyhow::bail!(
                    "Unknown link mode: {}. Expected: direct-link, static-link, dynamic-link, runtime-dlopen, generated-wrapper",
                    mode
                );
            }
        }

        // Validate wrapper if provided
        if let Some(ref wrapper) = self.wrapper {
            let wrapper_lower = wrapper.to_lowercase();
            if !["auto", "c", "rust", "zig", "none"].contains(&wrapper_lower.as_str()) {
                anyhow::bail!(
                    "Unknown wrapper policy: {}. Expected: auto, c, rust, zig, none",
                    wrapper
                );
            }
        }

        // Validate proof if provided
        if let Some(ref proof) = self.proof {
            let proof_lower = proof.to_lowercase();
            if !["required", "optional", "disabled"].contains(&proof_lower.as_str()) {
                anyhow::bail!(
                    "Unknown proof policy: {}. Expected: required, optional, disabled",
                    proof
                );
            }
        }

        Ok(())
    }

    /// Convert to an `AbiEdge`.
    pub fn to_abi_edge(&self) -> Result<AbiEdge> {
        let consumer = ComponentId::new(&self.consumer);
        let provider = ComponentId::new(&self.provider);

        let mut edge = AbiEdge::new(consumer, provider);

        // Set symbols
        for sym_str in &self.symbols {
            edge.add_symbols(vec![Symbol::new(sym_str)]);
        }

        // Set mode
        if let Some(ref mode) = self.mode {
            let mode_lower = mode.to_lowercase();
            let link_mode = match mode_lower.as_str() {
                "direct-link" => LinkMode::DirectLink,
                "static-link" => LinkMode::StaticLink,
                "dynamic-link" => LinkMode::DynamicLink,
                "runtime-dlopen" => LinkMode::RuntimeDlopen,
                "generated-wrapper" => LinkMode::GeneratedWrapper,
                _ => anyhow::bail!("unknown link mode: {}", mode),
            };
            edge.set_mode(link_mode);
        }

        // Set wrapper policy
        if let Some(ref wrapper) = self.wrapper {
            let wrapper_lower = wrapper.to_lowercase();
            edge.wrapper = match wrapper_lower.as_str() {
                "auto" => WrapperPolicy::Auto,
                "c" => WrapperPolicy::C,
                "rust" => WrapperPolicy::Rust,
                "zig" => WrapperPolicy::Zig,
                "none" => WrapperPolicy::None,
                _ => anyhow::bail!("unknown wrapper policy: {}", wrapper),
            };
        }

        // Set proof policy
        if let Some(ref proof) = self.proof {
            let proof_lower = proof.to_lowercase();
            edge.proof = match proof_lower.as_str() {
                "required" => ProofPolicy::Required,
                "optional" => ProofPolicy::Optional,
                "disabled" => ProofPolicy::Disabled,
                _ => anyhow::bail!("unknown proof policy: {}", proof),
            };
        }

        // Set runtime arg
        if let Some(ref runtime_arg) = self.runtime_arg {
            edge.runtime_arg = Some(runtime_arg.clone());
        }

        // Set visibility
        if let Some(ref visibility) = self.visibility {
            edge.visibility = visibility.clone();
        }

        // Set failure policy
        if let Some(ref failure_policy) = self.failure_policy {
            edge.failure_policy = failure_policy.clone();
        }

        Ok(edge)
    }
}

/// Validation errors with diagnostic codes.
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for SemanticError {}

impl ComponentEntry {
    /// Validate kind/language compatibility.
    /// Certain kinds only make sense for certain languages.
    pub fn validate_kind_language_compatibility(&self) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        let lang = self.language.to_lowercase();
        let kind = self.kind.as_ref().map(|k| k.to_lowercase());

        match (lang.as_str(), kind.as_deref()) {
            ("rust", Some("zig-exe"))
            | ("rust", Some("zig-lib"))
            | ("rust", Some("c-chimera-component")) => {
                errors.push(SemanticError {
                    code: KIND_LANG_MISMATCH,
                    message: format!(
                        "Component '{}': kind '{}' requires language 'zig', not 'rust'",
                        self.id,
                        kind.as_ref().unwrap()
                    ),
                });
            }
            ("zig", Some("cargo-package"))
            | ("zig", Some("c-source"))
            | ("zig", Some("rust-chimera-component"))
            | ("zig", Some("c-chimera-component")) => {
                errors.push(SemanticError {
                    code: KIND_LANG_MISMATCH,
                    message: format!(
                        "Component '{}': kind '{}' requires language 'c' or 'rust', not 'zig'",
                        self.id,
                        kind.as_ref().unwrap()
                    ),
                });
            }
            ("c", Some("cargo-package"))
            | ("c", Some("zig-exe"))
            | ("c", Some("zig-lib"))
            | ("c", Some("rust-chimera-component"))
            | ("c", Some("zig-chimera-component")) => {
                errors.push(SemanticError {
                    code: KIND_LANG_MISMATCH,
                    message: format!(
                        "Component '{}': kind '{}' requires language 'rust' or 'zig', not 'c'",
                        self.id,
                        kind.as_ref().unwrap()
                    ),
                });
            }
            _ => {}
        }
        errors
    }

    /// Validate target consistency for Rust cdylib/staticlib.
    pub fn validate_target_consistency(
        &self,
        all_components: &[ComponentEntry],
    ) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        let lang = self.language.to_lowercase();

        if lang != "rust" {
            return errors;
        }

        let has_cdylib = self
            .crate_types
            .iter()
            .any(|ct| ct.to_lowercase() == "cdylib");
        let has_staticlib = self
            .crate_types
            .iter()
            .any(|ct| ct.to_lowercase() == "staticlib");

        if !has_cdylib && !has_staticlib {
            return errors;
        }

        let self_target = self.target.as_ref();

        for other in all_components {
            if other.id == self.id {
                continue;
            }
            let other_has_native = other.crate_types.iter().any(|ct| {
                let ct_lower = ct.to_lowercase();
                ct_lower == "cdylib" || ct_lower == "staticlib"
            });

            if other_has_native {
                if let (Some(self_t), Some(other_t)) = (self_target, other.target.as_ref()) {
                    if self_t != other_t {
                        errors.push(SemanticError {
                            code: TARGET_INCONSISTENT,
                            message: format!(
                                "Component '{}' has target '{}' but component '{}' has target '{}'. Native libraries with different targets cannot be linked together.",
                                self.id, self_t, other.id, other_t
                            ),
                        });
                    }
                }
            }
        }
        errors
    }

    /// Validate output kind compatibility with component type.
    pub fn validate_output_kind_compatibility(&self, output_kind: &str) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        let _lang = self.language.to_lowercase();
        let kind = self.kind.as_ref().map(|k| k.to_lowercase());

        // Libraries should not produce executable output unless explicitly designed as one-binary
        if output_kind == "executable" {
            match kind.as_deref() {
                Some("cargo-package")
                | Some("zig-lib")
                | Some("c-source")
                | Some("rust-chimera-component")
                | Some("zig-chimera-component")
                | Some("c-chimera-component") => {
                    // These are library kinds - they can be linked into executables
                    // Only error if no exported symbols and no roots indicating an executable
                    if self.exported_symbols.is_empty()
                        && !self.roots.iter().any(|r| {
                            let r_lower = r.to_lowercase();
                            r_lower.contains("main.c")
                                || r_lower.contains("main.rs")
                                || r_lower.contains("main.zig")
                        })
                    {
                        errors.push(SemanticError {
                            code: OUTPUT_KIND_INCOMPATIBLE,
                            message: format!(
                                "Component '{}' is a library (kind={:?}) but output is 'executable'. Consider linking it into a binary component instead.",
                                self.id, kind
                            ),
                        });
                    }
                }
                _ => {}
            }
        }
        errors
    }
}

impl AbiEdgeEntry {
    /// Validate wrapper/proof policy compatibility.
    pub fn validate_policy_compatibility(&self) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        let mode = self.mode.as_ref().map(|m| m.to_lowercase());
        let wrapper = self.wrapper.as_ref().map(|w| w.to_lowercase());
        let proof = self.proof.as_ref().map(|p| p.to_lowercase());

        // generated-wrapper mode requires proof=required
        if mode.as_deref() == Some("generated-wrapper") {
            if proof.as_deref() == Some("disabled") {
                errors.push(SemanticError {
                    code: POLICY_INCOMPATIBLE,
                    message: format!(
                        "ABI edge {{consumer='{}', provider='{}'}}: generated-wrapper mode requires proof policy 'required' or 'optional', not 'disabled'",
                        self.consumer, self.provider
                    ),
                });
            }
        }

        // runtime-dlopen mode requires wrapper=none
        if mode.as_deref() == Some("runtime-dlopen") {
            if wrapper.as_deref() == Some("none") {
                // none is OK for dlopen
            } else if wrapper.is_some() {
                errors.push(SemanticError {
                    code: POLICY_INCOMPATIBLE,
                    message: format!(
                        "ABI edge {{consumer='{}', provider='{}'}}: runtime-dlopen mode should have wrapper='none', got '{}'",
                        self.consumer, self.provider, wrapper.as_ref().unwrap()
                    ),
                });
            }
        }

        // proof=required with proof policy and no symbols is suspicious
        if proof.as_deref() == Some("required") && self.symbols.is_empty() {
            errors.push(SemanticError {
                code: POLICY_INCOMPATIBLE,
                message: format!(
                    "ABI edge {{consumer='{}', provider='{}'}}: proof policy 'required' with no symbols specified",
                    self.consumer, self.provider
                ),
            });
        }

        errors
    }

    /// Validate runtime delivery rules.
    pub fn validate_runtime_delivery(&self) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        let mode = self.mode.as_ref().map(|m| m.to_lowercase());
        let runtime_arg = self.runtime_arg.as_ref();

        // runtime-dlopen mode should have runtime_arg
        if mode.as_deref() == Some("runtime-dlopen") {
            if runtime_arg.is_none() {
                errors.push(SemanticError {
                    code: RUNTIME_DELIVERY_INVALID,
                    message: format!(
                        "ABI edge {{consumer='{}', provider='{}'}}: runtime-dlopen mode requires runtime_arg to specify the runtime library path",
                        self.consumer, self.provider
                    ),
                });
            }
        }

        // Non-runtime modes should NOT have runtime_arg
        if mode.as_deref() != Some("runtime-dlopen") && mode.as_deref() != Some("generated-wrapper")
        {
            if runtime_arg.is_some() {
                errors.push(SemanticError {
                    code: RUNTIME_DELIVERY_INVALID,
                    message: format!(
                        "ABI edge {{consumer='{}', provider='{}'}}: runtime_arg is only valid for runtime-dlopen or generated-wrapper modes",
                        self.consumer, self.provider
                    ),
                });
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_entry_validation() {
        let comp = ComponentEntry {
            id: "my_lib".to_string(),
            language: "rust".to_string(),
            kind: Some("cargo-package".to_string()),
            roots: vec!["src/lib.rs".to_string()],
            manifest: Some("Cargo.toml".to_string()),
            package: Some("my_lib".to_string()),
            crate_types: vec!["staticlib".to_string()],
            features: vec![],
            panic_policy: Some("abort".to_string()),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            opt_level: Some(3),
            debug: Some(false),
            lto: Some(false),
            modules: vec![],
            imports: HashMap::new(),
            include_dirs: vec![],
            defines: vec![],
            exported_symbols: vec![],
            imported_symbols: vec![],
            entry_symbol: None,
            unified_entry_builtin: None,
        };

        assert!(comp.validate().is_ok());
        let spec = comp.to_component_spec();
        assert!(spec.is_ok());
    }

    #[test]
    fn test_component_entry_empty_id_fails() {
        let comp = ComponentEntry {
            id: "".to_string(),
            language: "rust".to_string(),
            ..Default::default()
        };

        assert!(comp.validate().is_err());
    }

    #[test]
    fn test_component_entry_unknown_language_fails() {
        let comp = ComponentEntry {
            id: "test".to_string(),
            language: "unknown_lang".to_string(),
            roots: vec!["src/lib.rs".to_string()],
            ..Default::default()
        };

        assert!(comp.validate().is_err());
    }

    #[test]
    fn test_component_entry_unified_entry_builtin_validation() {
        let comp = ComponentEntry {
            id: "launcher".to_string(),
            language: "c".to_string(),
            kind: Some("c-source".to_string()),
            roots: vec!["main.c".to_string()],
            unified_entry_builtin: Some("argv-entry-bridge".to_string()),
            ..Default::default()
        };

        assert!(comp.validate().is_ok());
        let spec = comp.to_component_spec().unwrap();
        assert_eq!(
            spec.unified_entry_builtin.as_deref(),
            Some("argv-entry-bridge")
        );
    }

    #[test]
    fn test_abi_edge_validation() {
        let edge = AbiEdgeEntry {
            consumer: "cli".to_string(),
            provider: "lib".to_string(),
            symbols: vec!["fn1".to_string(), "fn2".to_string()],
            mode: Some("runtime-dlopen".to_string()),
            wrapper: Some("auto".to_string()),
            proof: Some("required".to_string()),
            runtime_arg: Some("--rust-lib".to_string()),
            visibility: Some("pub".to_string()),
            failure_policy: Some("error".to_string()),
        };

        assert!(edge.validate().is_ok());
        let abi_edge = edge.to_abi_edge();
        assert!(abi_edge.is_ok());
    }

    #[test]
    fn test_abi_edge_empty_consumer_fails() {
        let edge = AbiEdgeEntry {
            consumer: "".to_string(),
            provider: "lib".to_string(),
            symbols: vec![],
            mode: None,
            wrapper: None,
            proof: None,
            runtime_arg: None,
            visibility: None,
            failure_policy: None,
        };

        assert!(edge.validate().is_err());
    }

    impl Default for ComponentEntry {
        fn default() -> Self {
            ComponentEntry {
                id: String::new(),
                language: "rust".to_string(),
                kind: None,
                roots: vec![],
                manifest: None,
                package: None,
                crate_types: vec![],
                features: vec![],
                panic_policy: None,
                target: None,
                opt_level: None,
                debug: None,
                lto: None,
                modules: vec![],
                imports: HashMap::new(),
                include_dirs: vec![],
                defines: vec![],
                exported_symbols: vec![],
                imported_symbols: vec![],
                entry_symbol: None,
                unified_entry_builtin: None,
            }
        }
    }

    // Semantic validation tests

    #[test]
    fn test_kind_language_mismatch_zig_exe_with_rust() {
        let comp = ComponentEntry {
            id: "test".to_string(),
            language: "rust".to_string(),
            kind: Some("zig-exe".to_string()),
            roots: vec!["src/main.zig".to_string()],
            ..Default::default()
        };
        let errors = comp.validate_kind_language_compatibility();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, KIND_LANG_MISMATCH);
    }

    #[test]
    fn test_kind_language_mismatch_cargo_package_with_zig() {
        let comp = ComponentEntry {
            id: "test".to_string(),
            language: "zig".to_string(),
            kind: Some("cargo-package".to_string()),
            roots: vec!["Cargo.toml".to_string()],
            ..Default::default()
        };
        let errors = comp.validate_kind_language_compatibility();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, KIND_LANG_MISMATCH);
    }

    #[test]
    fn test_kind_language_ok_rust_with_cargo_package() {
        let comp = ComponentEntry {
            id: "test".to_string(),
            language: "rust".to_string(),
            kind: Some("cargo-package".to_string()),
            roots: vec!["src/lib.rs".to_string()],
            ..Default::default()
        };
        let errors = comp.validate_kind_language_compatibility();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_target_consistency_mismatch() {
        let all_comps = vec![
            ComponentEntry {
                id: "lib1".to_string(),
                language: "rust".to_string(),
                kind: Some("cargo-package".to_string()),
                crate_types: vec!["cdylib".to_string()],
                target: Some("x86_64-unknown-linux-gnu".to_string()),
                roots: vec!["src/lib.rs".to_string()],
                ..Default::default()
            },
            ComponentEntry {
                id: "lib2".to_string(),
                language: "rust".to_string(),
                kind: Some("cargo-package".to_string()),
                crate_types: vec!["cdylib".to_string()],
                target: Some("aarch64-unknown-linux-gnu".to_string()),
                roots: vec!["src/lib.rs".to_string()],
                ..Default::default()
            },
        ];
        let errors = all_comps[0].validate_target_consistency(&all_comps);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, TARGET_INCONSISTENT);
    }

    #[test]
    fn test_target_consistency_ok_same_target() {
        let all_comps = vec![
            ComponentEntry {
                id: "lib1".to_string(),
                language: "rust".to_string(),
                kind: Some("cargo-package".to_string()),
                crate_types: vec!["cdylib".to_string()],
                target: Some("x86_64-unknown-linux-gnu".to_string()),
                roots: vec!["src/lib.rs".to_string()],
                ..Default::default()
            },
            ComponentEntry {
                id: "lib2".to_string(),
                language: "rust".to_string(),
                kind: Some("cargo-package".to_string()),
                crate_types: vec!["cdylib".to_string()],
                target: Some("x86_64-unknown-linux-gnu".to_string()),
                roots: vec!["src/lib.rs".to_string()],
                ..Default::default()
            },
        ];
        let errors = all_comps[0].validate_target_consistency(&all_comps);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_policy_compatibility_generated_wrapper_proof_disabled() {
        let edge = AbiEdgeEntry {
            consumer: "app".to_string(),
            provider: "lib".to_string(),
            symbols: vec!["my_func".to_string()],
            mode: Some("generated-wrapper".to_string()),
            wrapper: Some("zig".to_string()),
            proof: Some("disabled".to_string()),
            runtime_arg: None,
            visibility: None,
            failure_policy: None,
        };
        let errors = edge.validate_policy_compatibility();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, POLICY_INCOMPATIBLE);
    }

    #[test]
    fn test_policy_compatibility_runtime_dlopen_wrapper_none() {
        let edge = AbiEdgeEntry {
            consumer: "app".to_string(),
            provider: "lib".to_string(),
            symbols: vec!["my_func".to_string()],
            mode: Some("runtime-dlopen".to_string()),
            wrapper: Some("none".to_string()),
            proof: None,
            runtime_arg: Some("--rust-lib".to_string()),
            visibility: None,
            failure_policy: None,
        };
        let errors = edge.validate_policy_compatibility();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_runtime_delivery_missing_runtime_arg() {
        let edge = AbiEdgeEntry {
            consumer: "app".to_string(),
            provider: "lib".to_string(),
            symbols: vec!["my_func".to_string()],
            mode: Some("runtime-dlopen".to_string()),
            wrapper: None,
            proof: None,
            runtime_arg: None,
            visibility: None,
            failure_policy: None,
        };
        let errors = edge.validate_runtime_delivery();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, RUNTIME_DELIVERY_INVALID);
    }

    #[test]
    fn test_runtime_delivery_invalid_runtime_arg_for_direct_link() {
        let edge = AbiEdgeEntry {
            consumer: "app".to_string(),
            provider: "lib".to_string(),
            symbols: vec!["my_func".to_string()],
            mode: Some("direct-link".to_string()),
            wrapper: None,
            proof: None,
            runtime_arg: Some("--rust-lib".to_string()),
            visibility: None,
            failure_policy: None,
        };
        let errors = edge.validate_runtime_delivery();
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, RUNTIME_DELIVERY_INVALID);
    }

    #[test]
    fn test_output_kind_compatibility_library_executable() {
        let comp = ComponentEntry {
            id: "mylib".to_string(),
            language: "rust".to_string(),
            kind: Some("cargo-package".to_string()),
            roots: vec!["src/lib.rs".to_string()],
            manifest: Some("Cargo.toml".to_string()),
            ..Default::default()
        };
        let errors = comp.validate_output_kind_compatibility("executable");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, OUTPUT_KIND_INCOMPATIBLE);
    }

    #[test]
    fn test_output_kind_compatibility_with_main_symbol() {
        let comp = ComponentEntry {
            id: "myapp".to_string(),
            language: "rust".to_string(),
            kind: Some("cargo-package".to_string()),
            roots: vec!["src/main.rs".to_string()],
            manifest: Some("Cargo.toml".to_string()),
            exported_symbols: vec!["main".to_string()],
            ..Default::default()
        };
        let errors = comp.validate_output_kind_compatibility("executable");
        assert!(errors.is_empty());
    }
}
