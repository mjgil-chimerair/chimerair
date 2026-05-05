//! Chimera project manifest
//!
//! Defines and validates `Chimera.toml` project manifests describing sources,
//! imports, targets, toolchains, runtime mode, and output kind.
//!
//! The v0.2 manifest introduces `[[components]]` and `[[abi_edges]]` as the
//! primary schema, with `[[sources]]` maintained as a compatibility alias.

mod component;

use anyhow::{Context, Result};
use component::diag;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Chimera project manifest version
const MANIFEST_VERSION: &str = "0.1.0";

/// Valid target triple pattern (e.g., x86_64-unknown-linux-gnu)
const TARGET_TRIPLE_PATTERN: &str = r"^[a-z0-9_]+(-[a-z0-9_]+)+$";

/// Valid module name pattern
const MODULE_NAME_PATTERN: &str = r"^[a-zA-Z][a-zA-Z0-9_-]*$";

/// Runtime mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeMode {
    /// Core runtime - minimal types, no std
    Core,
    /// Standard runtime - full standard library
    Std,
    /// No standard library - embedded/no_std targets
    #[default]
    NoStd,
}

/// Output kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputKind {
    /// Static library
    #[default]
    StaticLib,
    /// Shared library / dynamic linked
    SharedLib,
    /// Executable binary
    Executable,
}

/// Source file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub path: String,
    pub language: String,
    /// Rust-specific: package path for the crate (e.g., " crates/mycrate")
    #[serde(default)]
    pub package_path: Option<String>,
    /// Rust-specific: crate type (library, binary, proc-macro)
    #[serde(default)]
    pub crate_type: Option<String>,
    /// Rust-specific: edition (2015, 2018, 2021)
    #[serde(default)]
    pub edition: Option<String>,
    /// Rust-specific: features enabled for this source
    #[serde(default)]
    pub features: Vec<String>,
    /// Rust-specific: panic policy (abort, unwind)
    #[serde(default)]
    pub panic_policy: Option<String>,
}

/// Import entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportEntry {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub cconv: Option<String>,
}

/// Target entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetEntry {
    pub triple: String,
    #[serde(default)]
    pub features: Vec<String>,
}

impl TargetEntry {
    /// Validate target triple format
    pub fn validate_triple(&self) -> Result<()> {
        let re = Regex::new(TARGET_TRIPLE_PATTERN).unwrap();
        if !re.is_match(&self.triple) {
            anyhow::bail!(
                "Invalid target triple '{}'. Expected format: arch-os-environment[-ext]",
                self.triple
            );
        }
        Ok(())
    }
}

/// Project manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    /// Manifest version (must be "0.1.0" or "0.2.0")
    pub version: String,
    /// Project name
    pub name: String,
    /// Project description
    #[serde(default)]
    pub description: Option<String>,
    /// Chimera ABI version required
    #[serde(default)]
    pub chimera_version: Option<String>,
    /// Components (v0.2 primary schema)
    #[serde(default)]
    pub components: Vec<component::ComponentEntry>,
    /// ABI edges (v0.2)
    #[serde(default)]
    pub abi_edges: Vec<component::AbiEdgeEntry>,
    /// Source files (v0.1 compatibility)
    #[serde(default)]
    pub sources: Vec<SourceEntry>,
    /// Import declarations
    #[serde(default)]
    pub imports: Vec<ImportEntry>,
    /// Build targets
    #[serde(default)]
    pub targets: Vec<TargetEntry>,
    /// Runtime mode
    #[serde(default)]
    pub runtime_mode: RuntimeMode,
    /// Output kind
    #[serde(default)]
    pub output: OutputKind,
    /// Toolchain overrides
    #[serde(default)]
    pub toolchains: ToolchainConfig,
}

/// Toolchain configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolchainConfig {
    /// C toolchain
    #[serde(default)]
    pub c: Option<String>,
    /// Rust toolchain
    #[serde(default)]
    pub rust: Option<String>,
    /// Zig toolchain
    #[serde(default)]
    pub zig: Option<String>,
}

impl ProjectManifest {
    /// Parse a manifest from a TOML string
    pub fn parse(toml_str: &str) -> Result<Self> {
        let manifest: ProjectManifest = toml::from_str(toml_str).context("Failed to parse TOML")?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parse a manifest from a file
    pub fn parse_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
        Self::parse(&content)
    }

    /// Serialize to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        // Accept both 0.1.0 (legacy) and 0.2.0 (component-based)
        if self.version != "0.1.0" && self.version != "0.2.0" {
            anyhow::bail!(
                "Unsupported manifest version: {}. Expected: 0.1.0 or 0.2.0",
                self.version
            );
        }

        if self.name.is_empty() {
            anyhow::bail!("Project name cannot be empty");
        }

        if self.name.len() > 256 {
            anyhow::bail!("Project name too long (max 256 characters)");
        }

        // Validate project name format
        let name_re = Regex::new(MODULE_NAME_PATTERN).unwrap();
        if !name_re.is_match(&self.name) {
            anyhow::bail!(
                "Invalid project name '{}'. Must start with letter and contain only letters, digits, underscore, or hyphen.",
                self.name
            );
        }

        // Validate components (v0.2)
        let mut component_ids = std::collections::HashSet::new();
        for comp in &self.components {
            comp.validate()?;

            // Semantic validation: kind/language compatibility
            for err in comp.validate_kind_language_compatibility() {
                anyhow::bail!("{}", err);
            }

            // Semantic validation: target consistency for native libs
            for err in comp.validate_target_consistency(&self.components) {
                anyhow::bail!("{}", err);
            }

            // Semantic validation: output kind compatibility
            let output_str = match self.output {
                OutputKind::StaticLib => "staticlib",
                OutputKind::SharedLib => "sharedlib",
                OutputKind::Executable => "executable",
            };
            for err in comp.validate_output_kind_compatibility(output_str) {
                anyhow::bail!("{}", err);
            }

            if !component_ids.insert(comp.id.clone()) {
                anyhow::bail!("Duplicate component ID: {}", comp.id);
            }
        }

        // Validate ABI edges (v0.2)
        for edge in &self.abi_edges {
            edge.validate()?;

            // Semantic validation: policy compatibility
            for err in edge.validate_policy_compatibility() {
                anyhow::bail!("{}", err);
            }

            // Semantic validation: runtime delivery rules
            for err in edge.validate_runtime_delivery() {
                anyhow::bail!("{}", err);
            }

            // Verify consumer and provider exist
            if !component_ids.contains(&edge.consumer) {
                anyhow::bail!(
                    "[{}] ABI edge references unknown consumer component: {}",
                    diag::ABI_COMPONENT_MISSING,
                    edge.consumer
                );
            }
            if !component_ids.contains(&edge.provider) {
                anyhow::bail!(
                    "[{}] ABI edge references unknown provider component: {}",
                    diag::ABI_COMPONENT_MISSING,
                    edge.provider
                );
            }
        }

        // Validate language names in sources
        for source in &self.sources {
            if !["c", "rust", "zig"].contains(&source.language.to_lowercase().as_str()) {
                anyhow::bail!(
                    "Unknown source language: {}. Expected: c, rust, zig",
                    source.language
                );
            }

            // Validate source path is not empty and doesn't contain null bytes
            if source.path.is_empty() {
                anyhow::bail!("Source path cannot be empty");
            }

            if source.path.contains('\0') {
                anyhow::bail!("Source path contains invalid character");
            }

            // Validate Rust-specific fields
            if source.language.to_lowercase() == "rust" {
                // Validate crate_type if provided
                if let Some(ref crate_type) = source.crate_type {
                    if !["library", "binary", "proc-macro"]
                        .contains(&crate_type.to_lowercase().as_str())
                    {
                        anyhow::bail!(
                            "Invalid crate_type '{}'. Expected: library, binary, proc-macro",
                            crate_type
                        );
                    }
                }

                // Validate edition if provided
                if let Some(ref edition) = source.edition {
                    if !["2015", "2018", "2021"].contains(&edition.as_str()) {
                        anyhow::bail!("Invalid edition '{}'. Expected: 2015, 2018, 2021", edition);
                    }
                }

                // Validate panic_policy if provided
                if let Some(ref panic_policy) = source.panic_policy {
                    if !["abort", "unwind"].contains(&panic_policy.to_lowercase().as_str()) {
                        anyhow::bail!(
                            "Invalid panic_policy '{}'. Expected: abort, unwind",
                            panic_policy
                        );
                    }
                }
            }
        }

        // Validate import calling conventions
        for import in &self.imports {
            if import.name.is_empty() {
                anyhow::bail!("Import name cannot be empty");
            }

            if import.path.is_empty() {
                anyhow::bail!("Import path cannot be empty");
            }

            if let Some(ref cconv) = import.cconv {
                if !["c", "sysv", "fastcall", "thiscall"].contains(&cconv.to_lowercase().as_str()) {
                    anyhow::bail!(
                        "Unknown calling convention: {}. Expected: c, sysv, fastcall, thiscall",
                        cconv
                    );
                }
            }
        }

        // Validate target triples
        for target in &self.targets {
            target.validate_triple()?;

            // Validate target features don't contain invalid characters
            for feature in &target.features {
                if feature.is_empty() || feature.contains(' ') || feature.contains('\0') {
                    anyhow::bail!(
                        "Invalid target feature '{}'. Features must be non-empty and contain no spaces or null characters.",
                        feature
                    );
                }
            }
        }

        // Validate chimera_version if provided
        if let Some(ref version) = self.chimera_version {
            if version.is_empty() {
                anyhow::bail!("Chimera version cannot be empty if specified");
            }
            // Basic semver-like check
            let semver_re = Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
            if !semver_re.is_match(version) {
                anyhow::bail!(
                    "Invalid chimera_version '{}'. Expected format: major.minor or major.minor.patch",
                    version
                );
            }
        }

        // Validate LTO setting if specified in description or elsewhere
        // This is a placeholder for potential LTO validation
        // Actual LTO validation would be in chimera-build

        Ok(())
    }

    /// Get the default target triple
    pub fn default_target(&self) -> Option<&str> {
        self.targets.first().map(|t| t.triple.as_str())
    }

    /// Check if a source file exists for the given language
    pub fn has_language(&self, lang: &str) -> bool {
        self.sources
            .iter()
            .any(|s| s.language.to_lowercase() == lang.to_lowercase())
    }

    /// Get the manifest version as a tuple (major, minor, patch).
    pub fn version_tuple(&self) -> (u32, u32, u32) {
        let parts: Vec<&str> = self.version.split('.').collect();
        let major: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch: u32 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        (major, minor, patch)
    }

    /// Check if this is a v0.2 component-based manifest.
    pub fn is_v2(&self) -> bool {
        self.version_tuple() >= (0, 2, 0)
    }

    /// Get components as ComponentSpec list.
    ///
    /// For v0.2 manifests, returns the parsed components.
    /// For v0.1 manifests with only sources, returns sources lowered to components.
    pub fn get_components(&self) -> Vec<Result<chimera_component::ComponentSpec>> {
        if self.is_v2() {
            self.components
                .iter()
                .map(|c| c.to_component_spec())
                .collect()
        } else {
            // Lower sources to components for v0.1 compatibility
            self.lower_sources_to_components()
        }
    }

    /// Get ABI edges as AbiEdge list.
    ///
    /// For v0.2 manifests, returns the parsed ABI edges.
    /// For v0.1 manifests, returns an empty list (no ABI edges in v0.1).
    pub fn get_abi_edges(&self) -> Vec<Result<chimera_component::AbiEdge>> {
        if self.is_v2() {
            self.abi_edges.iter().map(|e| e.to_abi_edge()).collect()
        } else {
            Vec::new()
        }
    }

    /// Lower v0.1 sources to components (compatibility mode).
    fn lower_sources_to_components(&self) -> Vec<Result<chimera_component::ComponentSpec>> {
        use chimera_component::{ComponentKind, Language};

        self.sources
            .iter()
            .enumerate()
            .map(|(idx, source)| {
                let id = format!("{}_{}", self.name.to_lowercase().replace('-', "_"), idx);

                let lang: Language = source
                    .language
                    .to_lowercase()
                    .parse()
                    .unwrap_or(Language::Unknown);
                let kind = match lang {
                    Language::Rust => ComponentKind::CargoPackage,
                    Language::Zig => ComponentKind::ZigExe,
                    Language::C => ComponentKind::CSource,
                    Language::Unknown => ComponentKind::ChimeraModule,
                };

                let mut spec = chimera_component::ComponentSpec::new(
                    chimera_component::ComponentId::new(id),
                    lang,
                    kind,
                );

                spec.add_root(std::path::PathBuf::from(&source.path));

                if let Some(ref package_path) = source.package_path {
                    // Use the path as the manifest/package
                    spec.set_manifest(std::path::PathBuf::from(package_path));
                }

                if let Some(ref package) = source.package_path {
                    // Extract package name from path
                    let name = std::path::Path::new(package)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    spec.set_package(name);
                }

                // Set features
                spec.features.clone_from(&source.features);

                // Set panic policy
                if let Some(ref pp) = source.panic_policy {
                    let pp_lower = pp.to_lowercase();
                    spec.panic_policy = Some(if pp_lower == "abort" {
                        chimera_component::PanicPolicy::Abort
                    } else {
                        chimera_component::PanicPolicy::Unwind
                    });
                }

                Ok(spec)
            })
            .collect()
    }

    /// Migrate a v0.1 manifest to v0.2 component format.
    ///
    /// Returns a new `ProjectManifest` with v0.2.0 version and components
    /// derived from the v0.1 sources. Issues warnings for lossy conversions.
    pub fn migrate_to_v2(&self) -> (ProjectManifest, Vec<String>) {
        let mut warnings = Vec::new();

        if self.is_v2() {
            warnings.push("Manifest is already v0.2.0, no migration needed.".to_string());
            return (self.clone(), warnings);
        }

        let mut new_manifest = ProjectManifest {
            version: "0.2.0".to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            chimera_version: self.chimera_version.clone(),
            components: Vec::new(),
            abi_edges: Vec::new(),
            sources: Vec::new(),
            imports: self.imports.clone(),
            targets: self.targets.clone(),
            runtime_mode: self.runtime_mode,
            output: self.output,
            toolchains: self.toolchains.clone(),
        };

        for (idx, source) in self.sources.iter().enumerate() {
            let comp_id = format!(
                "{}_{}",
                self.name.to_lowercase().replace('-', "_").replace(' ', "_"),
                idx
            );

            let lang = source.language.to_lowercase();
            let kind = match lang.as_str() {
                "rust" => "cargo-package",
                "zig" => "zig-exe",
                "c" => "c-source",
                _ => {
                    warnings.push(format!(
                        "Source '{}' has unknown language '{}', treating as chimera-module",
                        source.path, source.language
                    ));
                    "chimera-module"
                }
            };

            let mut comp = component::ComponentEntry {
                id: comp_id.clone(),
                language: lang,
                kind: Some(kind.to_string()),
                roots: vec![source.path.clone()],
                manifest: source.package_path.clone(),
                package: source.package_path.as_ref().map(|p| {
                    std::path::Path::new(p)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                }),
                crate_types: source.crate_type.iter().cloned().collect(),
                features: source.features.clone(),
                panic_policy: source.panic_policy.clone(),
                target: None,
                opt_level: None,
                debug: None,
                lto: None,
                modules: Vec::new(),
                imports: HashMap::new(),
                include_dirs: Vec::new(),
                defines: Vec::new(),
                exported_symbols: Vec::new(),
                imported_symbols: Vec::new(),
                entry_symbol: None,
                unified_entry_builtin: None,
            };

            // Warn about fields that don't map cleanly
            if source.edition.is_some() {
                warnings.push(format!(
                    "Source '{}': edition '{}' not preserved in migration (set in Cargo.toml)",
                    source.path,
                    source.edition.as_ref().unwrap()
                ));
            }

            new_manifest.components.push(comp);
        }

        if new_manifest.components.is_empty() {
            warnings.push("No sources found to migrate.".to_string());
        }

        (new_manifest, warnings)
    }
}

/// Error types for manifest operations
#[derive(Debug, Clone)]
pub enum ManifestError {
    UnsupportedVersion(String),
    InvalidProjectName,
    UnknownLanguage(String),
    InvalidCallingConvention(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::UnsupportedVersion(v) => {
                write!(f, "unsupported manifest version: {}", v)
            }
            ManifestError::InvalidProjectName => write!(f, "invalid project name"),
            ManifestError::UnknownLanguage(lang) => {
                write!(f, "unknown source language: {}", lang)
            }
            ManifestError::InvalidCallingConvention(cconv) => {
                write!(f, "unknown calling convention: {}", cconv)
            }
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_manifest() {
        let toml_str = r#"
version = "0.1.0"
name = "my-project"
description = "A test project"

[[sources]]
path = "src/lib.rs"
language = "rust"

[[sources]]
path = "src/main.c"
language = "c"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = []

[runtime]
mode = "std"
output = "executable"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.name, "my-project");
        assert_eq!(manifest.sources.len(), 2);
        assert_eq!(manifest.targets.len(), 1);
    }

    #[test]
    fn test_validate_version_mismatch() {
        let toml_str = r#"
version = "99.0.0"
name = "test"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let toml_str = r#"
version = "0.1.0"
name = ""
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_unknown_language() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/main.rs"
language = "go"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_cconv() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[imports]]
name = "my_func"
path = "my_func.so"
cconv = "invalid_cconv"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_runtime_mode() {
        let manifest = ProjectManifest {
            version: "0.1.0".to_string(),
            name: "test".to_string(),
            description: None,
            chimera_version: None,
            components: vec![],
            abi_edges: vec![],
            sources: vec![],
            imports: vec![],
            targets: vec![],
            runtime_mode: RuntimeMode::default(),
            output: OutputKind::default(),
            toolchains: ToolchainConfig::default(),
        };
        assert_eq!(manifest.runtime_mode, RuntimeMode::NoStd);
        assert_eq!(manifest.output, OutputKind::StaticLib);
    }

    #[test]
    fn test_has_language() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert!(manifest.has_language("rust"));
        assert!(manifest.has_language("Rust"));
        assert!(!manifest.has_language("c"));
    }

    #[test]
    fn test_roundtrip() {
        let toml_str = r#"
version = "0.1.0"
name = "test-project"
description = "A test project"

[[sources]]
path = "src/lib.rs"
language = "rust"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = []
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.name, manifest.name);
    }

    #[test]
    fn test_v2_manifest_roundtrip_components_only() {
        let toml_str = r#"
version = "0.2.0"
name = "v2-project"
description = "v2 component-only project"

[[components]]
id = "rust-lib"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
package = "my-lib"

[[components]]
id = "zig-exe"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]
modules = [
  { name = "helper", path = "src/helper.zig" }
]
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert!(manifest.is_v2());
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.name, manifest.name);
        assert_eq!(reparsed.components.len(), 2);
    }

    #[test]
    fn test_v2_manifest_roundtrip_abi_edges() {
        let toml_str = r#"
version = "0.2.0"
name = "abi-project"
description = "v2 with ABI edges"

[[components]]
id = "consumer"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]

[[components]]
id = "provider"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
package = "provider-lib"

[[abi_edges]]
consumer = "consumer"
provider = "provider"
symbols = ["rust_function"]
mode = "direct-link"
wrapper = "auto"
proof = "required"
visibility = "pub"
failure_policy = "error"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert!(manifest.is_v2());
        assert_eq!(manifest.components.len(), 2);
        assert_eq!(manifest.abi_edges.len(), 1);

        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.name, manifest.name);
        assert_eq!(reparsed.abi_edges.len(), 1);
        let edge = &reparsed.abi_edges[0];
        assert_eq!(edge.consumer, "consumer");
        assert_eq!(edge.provider, "provider");
        assert!(edge.symbols.contains(&"rust_function".to_string()));
    }

    #[test]
    fn test_v2_manifest_roundtrip_all_link_modes() {
        for (mode_str, symbols) in &[
            ("direct-link", vec!["sym1"]),
            ("static-link", vec!["sym2"]),
            ("dynamic-link", vec!["sym3"]),
            ("runtime-dlopen", vec!["sym4"]),
            ("generated-wrapper", vec!["sym5"]),
        ] {
            let toml_str = format!(
                r#"
version = "0.2.0"
name = "link-mode-{mode}"
description = "link mode test"

[[components]]
id = "consumer"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]

[[components]]
id = "provider"
language = "c"
kind = "c-source"
roots = ["src/lib.c"]

[[abi_edges]]
consumer = "consumer"
provider = "provider"
symbols = {symbols}
mode = "{mode}"
wrapper = "none"
proof = "disabled"
visibility = "pub"
failure_policy = "error"
"#,
                mode = mode_str,
                symbols = format!("{:?}", symbols)
            );
            let manifest = ProjectManifest::parse(&toml_str).unwrap();
            let output = manifest.to_toml().unwrap();
            let reparsed = ProjectManifest::parse(&output).unwrap();
            assert_eq!(reparsed.components.len(), 2);
            assert_eq!(reparsed.abi_edges.len(), 1);
            assert_eq!(
                reparsed.abi_edges[0].mode, *mode_str,
                "link mode round-trips: {}",
                mode_str
            );
        }
    }

    #[test]
    fn test_v2_manifest_roundtrip_multiple_edges() {
        let toml_str = r#"
version = "0.2.0"
name = "multi-edge"
description = "multiple ABI edges"

[[components]]
id = "app"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]

[[components]]
id = "math"
language = "c"
kind = "c-source"
roots = ["src/math.c"]

[[components]]
id = "io"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
package = "io-lib"

[[abi_edges]]
consumer = "app"
provider = "math"
symbols = ["add", "sub"]
mode = "direct-link"
wrapper = "none"
proof = "disabled"

[[abi_edges]]
consumer = "app"
provider = "io"
symbols = ["read_file"]
mode = "runtime-dlopen"
wrapper = "none"
proof = "required"
runtime_arg = "libio.so"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.components.len(), 3);
        assert_eq!(manifest.abi_edges.len(), 2);

        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.abi_edges.len(), 2);
        // Verify first edge (direct-link with math)
        let math_edge = &reparsed.abi_edges[0];
        assert_eq!(math_edge.mode, "direct-link");
        // Verify second edge (runtime-dlopen with io)
        let io_edge = &reparsed.abi_edges[1];
        assert_eq!(io_edge.mode, "runtime-dlopen");
        assert_eq!(io_edge.runtime_arg.as_deref(), Some("libio.so"));
    }

    #[test]
    fn test_v2_manifest_roundtrip_with_modules_and_imports() {
        let toml_str = r#"
version = "0.2.0"
name = "zig-modules"
description = "Zig with named modules and imports"

[[components]]
id = "main"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]
modules = [
  { name = "lib1", path = "src/lib1.zig" },
  { name = "lib2", path = "src/lib2.zig" }
]
imports = [
  { name = "std", path = "/zig/std" }
]
target = { triple = "x86_64-linux", features = [] }
opt_level = "ReleaseFast"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.components.len(), 1);
        let comp = &reparsed.components[0];
        assert_eq!(comp.modules.len(), 2);
        assert_eq!(comp.modules[0].name, "lib1");
        assert_eq!(comp.modules[1].name, "lib2");
        assert_eq!(comp.imports.len(), 1);
    }

    #[test]
    fn test_v2_manifest_json_roundtrip() {
        use std::collections::BTreeMap;

        let toml_str = r#"
version = "0.2.0"
name = "json-test"
description = "JSON round-trip test"

[[components]]
id = "c-lib"
language = "c"
kind = "c-source"
roots = ["src/lib.c"]
include_dirs = ["include"]
defines = [{ name = "NDEBUG", value = "1" }]

[[abi_edges]]
consumer = "c-lib"
provider = "c-lib"
symbols = ["init"]
mode = "direct-link"
wrapper = "none"
proof = "disabled"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();

        // Convert to canonical JSON representation
        let json = serde_json::to_string(&manifest).unwrap();
        let from_json: ProjectManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(from_json.name, manifest.name);
        assert_eq!(from_json.components.len(), 1);
        assert_eq!(from_json.components[0].include_dirs.len(), 1);
        assert_eq!(from_json.components[0].defines.len(), 1);
    }

    #[test]
    fn test_v2_manifest_roundtrip_c_source_with_all_fields() {
        let toml_str = r#"
version = "0.2.0"
name = "c-full"
description = "C component with all fields"

[[components]]
id = "c-lib"
language = "c"
kind = "c-source"
roots = ["src/lib.c", "src/helper.c"]
include_dirs = ["include", "src"]
defines = [
  { name = "NDEBUG", value = "1" },
  { name = "USE_FEATURE_X" }
]
target = { triple = "x86_64-linux", features = ["sse2", "avx2"] }
opt_level = "ReleaseFast"
debug = 2
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        let comp = &reparsed.components[0];
        assert_eq!(comp.roots.len(), 2);
        assert_eq!(comp.include_dirs.len(), 2);
        assert_eq!(comp.defines.len(), 2);
        assert!(comp.target.is_some());
    }

    #[test]
    fn test_v2_manifest_roundtrip_preserves_component_ids() {
        let toml_str = r#"
version = "0.2.0"
name = "id-preserve"
description = "verify component IDs survive round-trip"

[[components]]
id = "rust-lib"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
package = "my-crate"

[[components]]
id = "zig-main"
language = "zig"
kind = "zig-exe"
roots = ["src/main.zig"]

[[abi_edges]]
consumer = "zig-main"
provider = "rust-lib"
symbols = ["helper"]
mode = "direct-link"
wrapper = "auto"
proof = "required"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        assert_eq!(reparsed.components[0].id, "rust-lib");
        assert_eq!(reparsed.components[1].id, "zig-main");
        assert_eq!(reparsed.abi_edges[0].consumer, "zig-main");
        assert_eq!(reparsed.abi_edges[0].provider, "rust-lib");
    }

    #[test]
    fn test_v2_manifest_roundtrip_exported_imported_symbols() {
        let toml_str = r#"
version = "0.2.0"
name = "sym-test"
description = "symbol round-trip"

[[components]]
id = "lib"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
package = "lib"
exported_symbols = ["pub_fn", "pub_const"]
imported_symbols = ["c_extern_fn"]
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let output = manifest.to_toml().unwrap();
        let reparsed = ProjectManifest::parse(&output).unwrap();
        let comp = &reparsed.components[0];
        assert_eq!(comp.exported_symbols.len(), 2);
        assert!(comp.exported_symbols.contains(&"pub_fn".to_string()));
        assert!(comp.exported_symbols.contains(&"pub_const".to_string()));
    }

    #[test]
    fn test_validate_invalid_target_triple() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[targets]]
triple = "invalid"
features = []
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid target triple"));
    }

    #[test]
    fn test_validate_empty_source_path() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = ""
language = "c"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Source path cannot be empty"));
    }

    #[test]
    fn test_validate_empty_import_name() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[imports]]
name = ""
path = "lib.so"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Import name cannot be empty"));
    }

    #[test]
    fn test_validate_invalid_project_name() {
        let toml_str = r#"
version = "0.1.0"
name = "123-invalid-name"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid project name"));
    }

    #[test]
    fn test_validate_target_feature_with_space() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = ["sse4", "invalid feature"]
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid target feature"));
    }

    #[test]
    fn test_validate_invalid_chimera_version() {
        let toml_str = r#"
version = "0.1.0"
name = "test"
chimera_version = "not-a-version"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid chimera_version"));
    }

    #[test]
    fn test_validate_valid_chimera_version() {
        // Valid semver formats
        for version in &["1.0", "0.1.0", "2.5.3", "100.200.300"] {
            let toml_str = format!(
                r#"
version = "0.1.0"
name = "test"
chimera_version = "{}"
"#,
                version
            );
            let result = ProjectManifest::parse(&toml_str);
            assert!(result.is_ok(), "Expected {} to be valid", version);
        }
    }

    #[test]
    fn test_validate_wasm_target() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[targets]]
triple = "wasm32-wasi"
features = []
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.targets[0].triple, "wasm32-wasi");
    }

    #[test]
    fn test_validate_aarch64_target() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[targets]]
triple = "aarch64-unknown-linux-gnu"
features = ["neon"]
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.targets[0].triple, "aarch64-unknown-linux-gnu");
        assert_eq!(manifest.targets[0].features[0], "neon");
    }

    #[test]
    fn test_target_entry_validate_triple() {
        let valid_targets = vec![
            "x86_64-unknown-linux-gnu",
            "wasm32-wasi",
            "aarch64-apple-darwin",
            "riscv64gc-unknown-linux-gnu",
        ];

        for triple in valid_targets {
            let target = TargetEntry {
                triple: triple.to_string(),
                features: vec![],
            };
            assert!(
                target.validate_triple().is_ok(),
                "Expected {} to be valid",
                triple
            );
        }

        let invalid_targets = vec!["invalid", "x86_64-", "-unknown-linux", ""];

        for triple in invalid_targets {
            let target = TargetEntry {
                triple: triple.to_string(),
                features: vec![],
            };
            assert!(
                target.validate_triple().is_err(),
                "Expected {} to be invalid",
                triple
            );
        }
    }

    #[test]
    fn test_rust_source_with_all_fields() {
        let toml_str = r#"
version = "0.1.0"
name = "test-rust-project"

[[sources]]
path = "crates/mylib/src/lib.rs"
language = "rust"
package_path = "crates/mylib"
crate_type = "library"
edition = "2021"
features = ["default", "serde"]
panic_policy = "abort"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let source = &manifest.sources[0];
        assert_eq!(source.language, "rust");
        assert_eq!(source.package_path.as_ref().unwrap(), "crates/mylib");
        assert_eq!(source.crate_type.as_ref().unwrap(), "library");
        assert_eq!(source.edition.as_ref().unwrap(), "2021");
        assert_eq!(source.features.len(), 2);
        assert_eq!(source.panic_policy.as_ref().unwrap(), "abort");
    }

    #[test]
    fn test_rust_source_minimal() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/main.rs"
language = "rust"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.sources.len(), 1);
        let source = &manifest.sources[0];
        assert_eq!(source.language, "rust");
        assert!(source.package_path.is_none());
        assert!(source.crate_type.is_none());
        assert!(source.edition.is_none());
        assert!(source.features.is_empty());
        assert!(source.panic_policy.is_none());
    }

    #[test]
    fn test_validate_invalid_crate_type() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
crate_type = "invalid_type"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid crate_type"));
    }

    #[test]
    fn test_validate_invalid_edition() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
edition = "2022"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid edition"));
    }

    #[test]
    fn test_validate_invalid_panic_policy() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
panic_policy = "terminate"
"#;
        let result = ProjectManifest::parse(toml_str);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid panic_policy"));
    }

    #[test]
    fn test_validate_proc_macro_crate_type() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
crate_type = "proc-macro"
edition = "2018"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(
            manifest.sources[0].crate_type.as_ref().unwrap(),
            "proc-macro"
        );
        assert_eq!(manifest.sources[0].edition.as_ref().unwrap(), "2018");
    }

    #[test]
    fn test_validate_binary_crate_type() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/main.rs"
language = "rust"
crate_type = "binary"
edition = "2021"
panic_policy = "unwind"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.sources[0].crate_type.as_ref().unwrap(), "binary");
        assert_eq!(manifest.sources[0].panic_policy.as_ref().unwrap(), "unwind");
    }

    #[test]
    fn test_rust_source_features_validation() {
        let toml_str = r#"
version = "0.1.0"
name = "test"

[[sources]]
path = "src/lib.rs"
language = "rust"
features = ["default", "derive"]
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.sources[0].features.len(), 2);
    }

    // Migration tests

    #[test]
    fn test_migrate_v1_to_v2_basic() {
        let toml_str = r#"
version = "0.1.0"
name = "my-project"

[[sources]]
path = "src/lib.rs"
language = "rust"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.version, "0.2.0");
        assert!(migrated.is_v2());
        assert_eq!(migrated.components.len(), 1);
        assert_eq!(migrated.components[0].id, "my_project_0");
        assert_eq!(migrated.components[0].language, "rust");
        // Warnings about edition come only when edition is set
        assert!(warnings.is_empty() || warnings.iter().any(|w| w.contains("edition")));
    }

    #[test]
    fn test_migrate_v1_to_v2_multiple_sources() {
        let toml_str = r#"
version = "0.1.0"
name = "test-project"

[[sources]]
path = "src/lib.rs"
language = "rust"

[[sources]]
path = "src/main.c"
language = "c"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, _warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.version, "0.2.0");
        assert_eq!(migrated.components.len(), 2);
        assert_eq!(migrated.components[0].language, "rust");
        assert_eq!(
            migrated.components[0].kind.as_ref().unwrap(),
            "cargo-package"
        );
        assert_eq!(migrated.components[1].language, "c");
        assert_eq!(migrated.components[1].kind.as_ref().unwrap(), "c-source");
    }

    #[test]
    fn test_migrate_v1_to_v2_preserves_targets() {
        let toml_str = r#"
version = "0.1.0"
name = "test"
description = "A test project"

[[sources]]
path = "src/lib.rs"
language = "rust"

[[targets]]
triple = "x86_64-unknown-linux-gnu"
features = []
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, _warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.description, Some("A test project".to_string()));
        assert_eq!(migrated.targets.len(), 1);
        assert_eq!(migrated.targets[0].triple, "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_migrate_v1_to_v2_rust_with_all_fields() {
        let toml_str = r#"
version = "0.1.0"
name = "my-lib"

[[sources]]
path = "crates/mylib/src/lib.rs"
language = "rust"
package_path = "crates/mylib"
crate_type = "library"
edition = "2021"
features = ["default", "serde"]
panic_policy = "abort"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.version, "0.2.0");
        let comp = &migrated.components[0];
        assert_eq!(comp.language, "rust");
        assert_eq!(comp.kind.as_ref().unwrap(), "cargo-package");
        assert_eq!(comp.manifest.as_ref().unwrap(), "crates/mylib");
        assert!(comp.crate_types.contains(&"library".to_string()));
        assert!(comp.features.contains(&"default".to_string()));
        assert!(comp.features.contains(&"serde".to_string()));
        assert_eq!(comp.panic_policy.as_ref().unwrap(), "abort");
        // edition warning should be present
        assert!(warnings.iter().any(|w| w.contains("edition")));
    }

    #[test]
    fn test_migrate_already_v2_returns_same() {
        let toml_str = r#"
version = "0.2.0"
name = "test"

[[components]]
id = "mycomp"
language = "rust"
kind = "cargo-package"
roots = ["src/lib.rs"]
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (_migrated, warnings) = manifest.migrate_to_v2();

        assert!(warnings.iter().any(|w| w.contains("already v0.2.0")));
    }

    #[test]
    fn test_migrate_zig_source() {
        let toml_str = r#"
version = "0.1.0"
name = "zig-app"

[[sources]]
path = "src/main.zig"
language = "zig"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, _warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.components[0].language, "zig");
        assert_eq!(migrated.components[0].kind.as_ref().unwrap(), "zig-exe");
    }

    #[test]
    fn test_migrate_empty_sources() {
        let toml_str = r#"
version = "0.1.0"
name = "empty-project"
"#;
        let manifest = ProjectManifest::parse(toml_str).unwrap();
        let (migrated, warnings) = manifest.migrate_to_v2();

        assert_eq!(migrated.components.len(), 0);
        assert!(warnings.iter().any(|w| w.contains("No sources found")));
    }
}
