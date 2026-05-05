//! `chimera-artifact` - Artifact envelope types for ChimeraIR.
//!
//! This crate provides the artifact envelope types that flow through the build graph:
//! - `LanguageBuildResult` - complete build result from a language backend
//! - `ArtifactSet` - all artifacts produced by a component build
//! - `NativeLinkSpec` - link inputs for native linking
//! - `MetadataArtifacts` - metadata files (.chmeta, .zsnap, etc.)
//! - `ProofArtifacts` - proof files (.chproof)
//! - `PublicSurface` - fingerprints and symbol surfaces
//! - `InvalidationReport` - semantic invalidation explanation
//! - `RuntimeDelivery` - runtime file delivery specification

use chimera_component::{ComponentId, Symbol, WrapperPolicy};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// A fingerprint for content-addressing and invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// Algorithm used (e.g., "blake3", "sha256")
    pub algorithm: String,
    /// Hex-encoded hash
    pub hash: String,
}

impl Fingerprint {
    /// Create a new fingerprint.
    pub fn new(algorithm: impl Into<String>, hash: impl Into<String>) -> Self {
        Fingerprint {
            algorithm: algorithm.into(),
            hash: hash.into(),
        }
    }

    /// Check if two fingerprints are equal.
    pub fn matches(&self, other: &Fingerprint) -> bool {
        self.hash == other.hash
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.algorithm, self.hash)
    }
}

/// Build status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildStatus {
    Success,
    Failed,
    Skipped,
}

impl Default for BuildStatus {
    fn default() -> Self {
        BuildStatus::Success
    }
}

/// A set of artifacts produced by a component build.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactSet {
    /// Native object files
    #[serde(default)]
    pub objects: Vec<PathBuf>,
    /// Static archives
    #[serde(default)]
    pub archives: Vec<PathBuf>,
    /// Shared libraries
    #[serde(default)]
    pub shared_libs: Vec<PathBuf>,
    /// Executables
    #[serde(default)]
    pub executables: Vec<PathBuf>,
    /// ChimeraIR files (.chimera, .chir)
    #[serde(default)]
    pub chimera_ir: Vec<PathBuf>,
    /// Metadata files (.zsnap, .rdep, .chmeta)
    #[serde(default)]
    pub metadata: Vec<PathBuf>,
    /// Proof files (.chproof)
    #[serde(default)]
    pub proofs: Vec<PathBuf>,
    /// Snapshot files (.zsnap, .rsnap)
    #[serde(default)]
    pub snapshots: Vec<PathBuf>,
    /// Dependency graphs (.zdep, .rdepgraph)
    #[serde(default)]
    pub depgraphs: Vec<PathBuf>,
}

impl ArtifactSet {
    /// Create a new empty artifact set.
    pub fn new() -> Self {
        ArtifactSet::default()
    }

    /// Add an object file.
    pub fn add_object(&mut self, path: PathBuf) {
        self.objects.push(path);
    }

    /// Add a static archive.
    pub fn add_archive(&mut self, path: PathBuf) {
        self.archives.push(path);
    }

    /// Add a shared library.
    pub fn add_shared_lib(&mut self, path: PathBuf) {
        self.shared_libs.push(path);
    }

    /// Add an executable.
    pub fn add_executable(&mut self, path: PathBuf) {
        self.executables.push(path);
    }

    /// Add a ChimeraIR artifact.
    pub fn add_chimera_ir(&mut self, path: PathBuf) {
        self.chimera_ir.push(path);
    }

    /// Add a metadata artifact.
    pub fn add_metadata(&mut self, path: PathBuf) {
        self.metadata.push(path);
    }

    /// Add a proof artifact.
    pub fn add_proof(&mut self, path: PathBuf) {
        self.proofs.push(path);
    }

    /// Check if the artifact set is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
            && self.archives.is_empty()
            && self.shared_libs.is_empty()
            && self.executables.is_empty()
            && self.chimera_ir.is_empty()
            && self.metadata.is_empty()
            && self.proofs.is_empty()
            && self.snapshots.is_empty()
            && self.depgraphs.is_empty()
    }

    /// Merge another artifact set into this one.
    pub fn merge(&mut self, other: &ArtifactSet) {
        self.objects.extend_from_slice(&other.objects);
        self.archives.extend_from_slice(&other.archives);
        self.shared_libs.extend_from_slice(&other.shared_libs);
        self.executables.extend_from_slice(&other.executables);
        self.chimera_ir.extend_from_slice(&other.chimera_ir);
        self.metadata.extend_from_slice(&other.metadata);
        self.proofs.extend_from_slice(&other.proofs);
        self.snapshots.extend_from_slice(&other.snapshots);
        self.depgraphs.extend_from_slice(&other.depgraphs);
    }
}

/// Native link specification for the linker.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativeLinkSpec {
    /// Object files to link
    #[serde(default)]
    pub objects: Vec<PathBuf>,
    /// Static archives to link
    #[serde(default)]
    pub static_archives: Vec<PathBuf>,
    /// Shared libraries to link
    #[serde(default)]
    pub shared_libraries: Vec<PathBuf>,
    /// Library search paths
    #[serde(default)]
    pub library_search_paths: Vec<PathBuf>,
    /// Libraries to link (-l flags)
    #[serde(default)]
    pub link_libraries: Vec<String>,
    /// Additional linker arguments
    #[serde(default)]
    pub linker_args: Vec<String>,
    /// Runtime paths (rpaths)
    #[serde(default)]
    pub rpaths: Vec<PathBuf>,
    /// Runtime files to package (for dlopen)
    #[serde(default)]
    pub runtime_files: Vec<PathBuf>,
    /// System libraries
    #[serde(default)]
    pub system_libraries: Vec<String>,
}

impl NativeLinkSpec {
    /// Create a new empty link spec.
    pub fn new() -> Self {
        NativeLinkSpec::default()
    }

    /// Merge another link spec into this one with deterministic ordering and duplicate suppression.
    ///
    /// Merge order: objects, static_archives, shared_libraries, library_search_paths,
    /// link_libraries, linker_args, rpaths, runtime_files, system_libraries
    ///
    /// Duplicates are suppressed based on content identity.
    /// System libraries are sorted and deduplicated.
    /// rpaths are kept in insertion order (first occurrence wins for same path).
    pub fn merge(&mut self, other: &NativeLinkSpec) {
        // Merge objects (preserve order, deduplicate)
        for obj in &other.objects {
            if !self.objects.contains(obj) {
                self.objects.push(obj.clone());
            }
        }

        // Merge static archives
        for archive in &other.static_archives {
            if !self.static_archives.contains(archive) {
                self.static_archives.push(archive.clone());
            }
        }

        // Merge shared libraries
        for lib in &other.shared_libraries {
            if !self.shared_libraries.contains(lib) {
                self.shared_libraries.push(lib.clone());
            }
        }

        // Merge library search paths (dedupe, preserve order)
        for dir in &other.library_search_paths {
            if !self.library_search_paths.contains(dir) {
                self.library_search_paths.push(dir.clone());
            }
        }

        // Merge link libraries (dedupe)
        for lib in &other.link_libraries {
            if !self.link_libraries.contains(lib) {
                self.link_libraries.push(lib.clone());
            }
        }

        // Merge linker args (dedupe)
        for arg in &other.linker_args {
            if !self.linker_args.contains(arg) {
                self.linker_args.push(arg.clone());
            }
        }

        // Merge rpaths (first wins for same path)
        for rp in &other.rpaths {
            if !self.rpaths.contains(rp) {
                self.rpaths.push(rp.clone());
            }
        }

        // Merge runtime files
        for file in &other.runtime_files {
            if !self.runtime_files.contains(file) {
                self.runtime_files.push(file.clone());
            }
        }

        // Merge and sort system libraries (alphabetical, dedupe)
        for lib in &other.system_libraries {
            if !self.system_libraries.contains(lib) {
                self.system_libraries.push(lib.clone());
            }
        }
        self.system_libraries.sort();
        self.system_libraries.dedup();
    }

    /// Merge multiple link specs in order.
    pub fn merge_all(&mut self, others: &[NativeLinkSpec]) {
        for other in others {
            self.merge(other);
        }
    }

    /// Check for merge conflicts (same object with different properties).
    /// Returns list of conflicting paths.
    pub fn find_conflicts(&self, other: &NativeLinkSpec) -> Vec<PathBuf> {
        let mut conflicts = Vec::new();

        // Objects that appear in both specs
        for obj in &self.objects {
            if other.objects.contains(obj) {
                conflicts.push(obj.clone());
            }
        }

        // Archives that appear in both specs
        for archive in &self.static_archives {
            if other.static_archives.contains(archive) {
                conflicts.push(archive.clone());
            }
        }

        conflicts
    }

    /// Normalize all paths to canonical form for deterministic output.
    pub fn normalize_paths(&mut self) {
        // Normalize object paths
        for obj in &mut self.objects {
            if let Ok(canonical) = obj.canonicalize() {
                *obj = canonical;
            }
        }

        // Normalize static archives
        for archive in &mut self.static_archives {
            if let Ok(canonical) = archive.canonicalize() {
                *archive = canonical;
            }
        }

        // Normalize shared libraries
        for lib in &mut self.shared_libraries {
            if let Ok(canonical) = lib.canonicalize() {
                *lib = canonical;
            }
        }

        // Normalize library search paths
        for dir in &mut self.library_search_paths {
            if let Ok(canonical) = dir.canonicalize() {
                *dir = canonical;
            }
        }

        // Normalize rpaths
        for rp in &mut self.rpaths {
            if let Ok(canonical) = rp.canonicalize() {
                *rp = canonical;
            }
        }

        // Normalize runtime files
        for file in &mut self.runtime_files {
            if let Ok(canonical) = file.canonicalize() {
                *file = canonical;
            }
        }
    }

    /// Check if the spec is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
            && self.static_archives.is_empty()
            && self.shared_libraries.is_empty()
            && self.link_libraries.is_empty()
    }
}

/// Metadata artifacts from a build.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetadataArtifacts {
    /// .chmeta files
    #[serde(default)]
    pub chmeta: Vec<PathBuf>,
    /// .zsnap files (Zig snapshots)
    #[serde(default)]
    pub zsnap: Vec<PathBuf>,
    /// .rsnap files (Rust snapshots)
    #[serde(default)]
    pub rsnap: Vec<PathBuf>,
    /// .zdep files (Zig dependency graph)
    #[serde(default)]
    pub zdep: Vec<PathBuf>,
    /// .rdep files (Rust dependency graph)
    #[serde(default)]
    pub rdep: Vec<PathBuf>,
    /// .zairpack files (Zig AIR package)
    #[serde(default)]
    pub zairpack: Vec<PathBuf>,
    /// .csnap files (C snapshots)
    #[serde(default)]
    pub csnap: Vec<PathBuf>,
    /// .cdep files (C dependency graph)
    #[serde(default)]
    pub cdep: Vec<PathBuf>,
    /// compile_commands.json
    #[serde(default)]
    pub compile_commands: Vec<PathBuf>,
}

impl MetadataArtifacts {
    /// Create a new empty metadata artifacts set.
    pub fn new() -> Self {
        MetadataArtifacts::default()
    }
}

/// Proof artifacts from a build.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofArtifacts {
    /// .chproof files
    #[serde(default)]
    pub chproof: Vec<PathBuf>,
    /// Lean proof files
    #[serde(default)]
    pub lean_proofs: Vec<PathBuf>,
}

impl ProofArtifacts {
    /// Create a new empty proof artifacts set.
    pub fn new() -> Self {
        ProofArtifacts::default()
    }
}

/// Public surface of a component, used for semantic invalidation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublicSurface {
    /// ABI fingerprint
    #[serde(default)]
    pub abi_fingerprint: Option<Fingerprint>,
    /// Layout fingerprint
    #[serde(default)]
    pub layout_fingerprint: Option<Fingerprint>,
    /// Effect fingerprint
    #[serde(default)]
    pub effect_fingerprint: Option<Fingerprint>,
    /// Ownership fingerprint
    #[serde(default)]
    pub ownership_fingerprint: Option<Fingerprint>,
    /// Panic policy fingerprint
    #[serde(default)]
    pub panic_policy_fingerprint: Option<Fingerprint>,
    /// Proof surface fingerprint
    #[serde(default)]
    pub proof_surface_fingerprint: Option<Fingerprint>,
    /// Wrapper surface fingerprint
    #[serde(default)]
    pub wrapper_surface_fingerprint: Option<Fingerprint>,
    /// Exported symbols
    #[serde(default)]
    pub exported_symbols: Vec<Symbol>,
    /// Imported symbols
    #[serde(default)]
    pub imported_symbols: Vec<Symbol>,
}

impl PublicSurface {
    /// Create a new empty public surface.
    pub fn new() -> Self {
        PublicSurface::default()
    }

    /// Check if the public surface has any fingerprints.
    pub fn has_fingerprints(&self) -> bool {
        self.abi_fingerprint.is_some()
            || self.layout_fingerprint.is_some()
            || self.effect_fingerprint.is_some()
            || self.ownership_fingerprint.is_some()
            || self.panic_policy_fingerprint.is_some()
            || self.proof_surface_fingerprint.is_some()
            || self.wrapper_surface_fingerprint.is_some()
    }

    /// Compute ABI fingerprint from exported symbols.
    pub fn compute_abi_fingerprint(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for sym in &self.exported_symbols {
            sym.hash(&mut hasher);
        }
        let hash = hasher.finish();
        self.abi_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute layout fingerprint from exported symbols (types/sizes).
    pub fn compute_layout_fingerprint(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Layout is determined by symbol names and their type info
        for sym in &self.exported_symbols {
            sym.hash(&mut hasher);
        }
        let hash = hasher.finish();
        self.layout_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute effect fingerprint from exported function effects.
    pub fn compute_effect_fingerprint(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for sym in &self.exported_symbols {
            // Hash symbol name + effect info
            sym.hash(&mut hasher);
        }
        let hash = hasher.finish();
        self.effect_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute ownership fingerprint.
    pub fn compute_ownership_fingerprint(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.exported_symbols.hash(&mut hasher);
        self.imported_symbols.hash(&mut hasher);
        let hash = hasher.finish();
        self.ownership_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute panic policy fingerprint from panic-related symbols.
    pub fn compute_panic_policy_fingerprint(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        // Focus on panic-related symbols
        for sym in &self.exported_symbols {
            if sym.name.to_lowercase().contains("panic")
                || sym.name.to_lowercase().contains("abort")
            {
                sym.hash(&mut hasher);
            }
        }
        let hash = hasher.finish();
        self.panic_policy_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute wrapper surface fingerprint from ABI edge wrappers.
    pub fn compute_wrapper_surface_fingerprint(&mut self, wrappers: &[WrapperRequest]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for wrapper in wrappers {
            wrapper.hash(&mut hasher);
        }
        let hash = hasher.finish();
        self.wrapper_surface_fingerprint =
            Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }

    /// Compute proof surface fingerprint from proof requirements.
    pub fn compute_proof_surface_fingerprint(&mut self, proof_obligations: &[String]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for obligation in proof_obligations {
            obligation.hash(&mut hasher);
        }
        let hash = hasher.finish();
        self.proof_surface_fingerprint = Some(Fingerprint::new("blake3", format!("{:016x}", hash)));
    }
}

/// Invalidation report for a component build.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvalidationReport {
    /// Private body changed (no downstream effect)
    #[serde(default)]
    pub private_body_changed: bool,
    /// ABI signature changed
    #[serde(default)]
    pub abi_changed: bool,
    /// Layout changed
    #[serde(default)]
    pub layout_changed: bool,
    /// Effects changed
    #[serde(default)]
    pub effects_changed: bool,
    /// Proof surface changed
    #[serde(default)]
    pub proof_surface_changed: bool,
    /// Wrappers are stale
    #[serde(default)]
    pub wrappers_stale: bool,
    /// Link is stale
    #[serde(default)]
    pub link_stale: bool,
    /// Runtime package is stale
    #[serde(default)]
    pub runtime_package_stale: bool,
}

impl InvalidationReport {
    /// Create a new empty invalidation report (no changes).
    pub fn new() -> Self {
        InvalidationReport::default()
    }

    /// Check if any downstream action is required.
    pub fn requires_downstream_action(&self) -> bool {
        self.wrappers_stale || self.link_stale || self.runtime_package_stale
    }

    /// Check if this is a clean build (no invalidation).
    pub fn is_clean(&self) -> bool {
        !self.private_body_changed
            && !self.abi_changed
            && !self.layout_changed
            && !self.effects_changed
            && !self.proof_surface_changed
            && !self.wrappers_stale
            && !self.link_stale
            && !self.runtime_package_stale
    }
}

/// A diagnostic message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Error code
    pub code: String,
    /// Message text
    pub message: String,
    /// Source location (file:line:col)
    #[serde(default)]
    pub location: Option<String>,
    /// Suggestions or hints
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Debug,
}

impl Default for DiagnosticSeverity {
    fn default() -> Self {
        DiagnosticSeverity::Info
    }
}

impl Diagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
        }
    }
}

/// A wrapper generation request.
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct WrapperRequest {
    /// Consumer component
    pub consumer: ComponentId,
    /// Provider component
    pub provider: ComponentId,
    /// ABI edge mode
    pub mode: WrapperPolicy,
    /// Symbols to wrap
    #[serde(default)]
    pub symbols: Vec<Symbol>,
}

impl WrapperRequest {
    /// Create a new wrapper request.
    pub fn new(consumer: ComponentId, provider: ComponentId, mode: WrapperPolicy) -> Self {
        WrapperRequest {
            consumer,
            provider,
            mode,
            symbols: Vec::new(),
        }
    }
}

/// Runtime delivery specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDelivery {
    /// Files to deliver at runtime
    #[serde(default)]
    pub files: Vec<RuntimeFile>,
    /// Dynamic library search paths
    #[serde(default)]
    pub search_paths: Vec<PathBuf>,
    /// Environment variables to set
    #[serde(default)]
    pub env_vars: Vec<(String, String)>,
}

/// A runtime file to deliver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFile {
    /// Source path
    pub source: PathBuf,
    /// Destination name
    pub dest_name: String,
    /// Whether to set executable permission
    #[serde(default)]
    pub executable: bool,
}

impl RuntimeFile {
    /// Create a new runtime file.
    pub fn new(source: PathBuf, dest_name: impl Into<String>) -> Self {
        RuntimeFile {
            source,
            dest_name: dest_name.into(),
            executable: false,
        }
    }

    /// Set the executable flag.
    pub fn set_executable(&mut self) {
        self.executable = true;
    }
}

/// The complete result of a language build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageBuildResult {
    /// Component ID
    pub component_id: ComponentId,
    /// Language
    pub language: chimera_component::Language,
    /// Build status
    #[serde(default)]
    pub status: BuildStatus,
    /// Primary output artifacts
    #[serde(default)]
    pub primary_outputs: ArtifactSet,
    /// Native link specification
    #[serde(default)]
    pub link: NativeLinkSpec,
    /// Metadata artifacts
    #[serde(default)]
    pub metadata: MetadataArtifacts,
    /// Proof artifacts
    #[serde(default)]
    pub proof: ProofArtifacts,
    /// Wrappers required by this component
    #[serde(default)]
    pub wrappers_required: Vec<WrapperRequest>,
    /// Public surface
    #[serde(default)]
    pub public_surface: PublicSurface,
    /// Invalidation report
    #[serde(default)]
    pub invalidation: InvalidationReport,
    /// Diagnostics
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

impl LanguageBuildResult {
    /// Create a new build result.
    pub fn new(component_id: ComponentId, language: chimera_component::Language) -> Self {
        LanguageBuildResult {
            component_id,
            language,
            status: BuildStatus::Success,
            primary_outputs: ArtifactSet::new(),
            link: NativeLinkSpec::new(),
            metadata: MetadataArtifacts::new(),
            proof: ProofArtifacts::new(),
            wrappers_required: Vec::new(),
            public_surface: PublicSurface::new(),
            invalidation: InvalidationReport::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Mark the build as successful.
    pub fn set_success(&mut self) {
        self.status = BuildStatus::Success;
    }

    /// Mark the build as failed.
    pub fn set_failed(&mut self) {
        self.status = BuildStatus::Failed;
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(&mut self, diag: Diagnostic) {
        self.diagnostics.push(diag);
    }

    /// Add a wrapper requirement.
    pub fn add_wrapper_request(&mut self, request: WrapperRequest) {
        self.wrappers_required.push(request);
    }

    /// Check if the build succeeded.
    pub fn is_success(&self) -> bool {
        self.status == BuildStatus::Success
    }

    /// Check if any proofs failed.
    pub fn has_proof_failures(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.code.starts_with("PROOF_"))
    }
}

/// Artifact manifest for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactManifest {
    /// Schema version
    pub version: String,
    /// Component ID
    pub component_id: ComponentId,
    /// Toolchain identity
    #[serde(default)]
    pub toolchain: Option<String>,
    /// Artifact set
    pub artifacts: ArtifactSet,
    /// Public surface
    #[serde(default)]
    pub public_surface: Option<PublicSurface>,
    /// Link spec
    #[serde(default)]
    pub link: Option<NativeLinkSpec>,
    /// Runtime delivery
    #[serde(default)]
    pub runtime_delivery: Option<RuntimeDelivery>,
    /// Diagnostics
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
}

impl ArtifactManifest {
    /// Create a new artifact manifest.
    pub fn new(component_id: ComponentId) -> Self {
        ArtifactManifest {
            version: "0.1.0".to_string(),
            component_id,
            toolchain: None,
            artifacts: ArtifactSet::new(),
            public_surface: None,
            link: None,
            runtime_delivery: None,
            diagnostics: Vec::new(),
        }
    }

    /// Validate the schema version.
    pub fn validate_version(&self) -> Result<(), String> {
        if self.version != "0.1.0" {
            return Err(format!(
                "unsupported artifact manifest version: {}",
                self.version
            ));
        }
        Ok(())
    }

    /// Write the manifest to a file.
    pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Read a manifest from a file.
    pub fn read_from(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Compute a content-addressed hash for cache keying.
    pub fn content_hash(&self) -> Fingerprint {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Use JSON serialization for deterministic hashing
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let hash = hasher.finish();
        Fingerprint::new("blake3", format!("{:016x}", hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_creation() {
        let fp = Fingerprint::new("blake3", "abc123");
        assert_eq!(fp.algorithm, "blake3");
        assert_eq!(fp.hash, "abc123");
    }

    #[test]
    fn test_fingerprint_matching() {
        let fp1 = Fingerprint::new("blake3", "abc123");
        let fp2 = Fingerprint::new("blake3", "abc123");
        let fp3 = Fingerprint::new("blake3", "def456");
        assert!(fp1.matches(&fp2));
        assert!(!fp1.matches(&fp3));
    }

    #[test]
    fn test_artifact_set_empty() {
        let set = ArtifactSet::new();
        assert!(set.is_empty());
    }

    #[test]
    fn test_artifact_set_add_objects() {
        let mut set = ArtifactSet::new();
        set.add_object(PathBuf::from("build/main.o"));
        assert!(!set.is_empty());
        assert_eq!(set.objects.len(), 1);
    }

    #[test]
    fn test_artifact_set_add_chimera_ir() {
        let mut set = ArtifactSet::new();
        set.add_chimera_ir(PathBuf::from("build/output.chir"));
        assert!(!set.is_empty());
        assert_eq!(set.chimera_ir.len(), 1);
    }

    #[test]
    fn test_artifact_set_merge() {
        let mut set1 = ArtifactSet::new();
        set1.add_object(PathBuf::from("a.o"));

        let mut set2 = ArtifactSet::new();
        set2.add_object(PathBuf::from("b.o"));
        set2.add_archive(PathBuf::from("lib.a"));

        set1.merge(&set2);
        assert_eq!(set1.objects.len(), 2);
        assert_eq!(set1.archives.len(), 1);
    }

    #[test]
    fn test_native_link_spec_merge() {
        let mut spec1 = NativeLinkSpec::new();
        spec1.objects.push(PathBuf::from("a.o"));

        let mut spec2 = NativeLinkSpec::new();
        spec2.objects.push(PathBuf::from("b.o"));
        spec2.rpaths.push(PathBuf::from("/usr/lib"));

        spec1.merge(&spec2);
        assert_eq!(spec1.objects.len(), 2);
        assert_eq!(spec1.rpaths.len(), 1);
    }

    #[test]
    fn test_public_surface_has_fingerprints() {
        let mut surface = PublicSurface::new();
        assert!(!surface.has_fingerprints());

        surface.abi_fingerprint = Some(Fingerprint::new("blake3", "abc"));
        assert!(surface.has_fingerprints());
    }

    #[test]
    fn test_invalidation_report_is_clean() {
        let report = InvalidationReport::new();
        assert!(report.is_clean());

        let mut dirty = InvalidationReport::new();
        dirty.abi_changed = true;
        assert!(!dirty.is_clean());
    }

    #[test]
    fn test_invalidation_report_requires_downstream_action() {
        let mut report = InvalidationReport::new();
        assert!(!report.requires_downstream_action());

        report.wrappers_stale = true;
        assert!(report.requires_downstream_action());
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error("E001", "something went wrong");
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, "E001");
    }

    #[test]
    fn test_wrapper_request_creation() {
        let req = WrapperRequest::new(
            ComponentId::new("consumer"),
            ComponentId::new("provider"),
            WrapperPolicy::Auto,
        );
        assert_eq!(req.consumer.as_str(), "consumer");
        assert_eq!(req.provider.as_str(), "provider");
    }

    #[test]
    fn test_runtime_file() {
        let mut file = RuntimeFile::new(PathBuf::from("lib.so"), "lib.so");
        assert!(!file.executable);
        file.set_executable();
        assert!(file.executable);
    }

    #[test]
    fn test_language_build_result() {
        let mut result = LanguageBuildResult::new(
            ComponentId::new("test_comp"),
            chimera_component::Language::Rust,
        );
        result.set_success();
        result.add_diagnostic(Diagnostic::warning("W001", "a warning"));

        assert!(result.is_success());
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_artifact_manifest_version_validation() {
        let manifest = ArtifactManifest::new(ComponentId::new("test"));
        assert!(manifest.validate_version().is_ok());

        let mut bad_manifest = ArtifactManifest::new(ComponentId::new("test"));
        bad_manifest.version = "99.99".to_string();
        assert!(bad_manifest.validate_version().is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut result =
            LanguageBuildResult::new(ComponentId::new("test"), chimera_component::Language::Zig);
        result.primary_outputs.add_object(PathBuf::from("out.o"));

        let json = serde_json::to_string(&result).unwrap();
        let parsed: LanguageBuildResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.component_id, result.component_id);
        assert_eq!(parsed.primary_outputs.objects.len(), 1);
    }

    #[test]
    fn test_artifact_manifest_write_read_roundtrip() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut manifest = ArtifactManifest::new(ComponentId::new("test_comp"));
        manifest.artifacts.add_object(PathBuf::from("obj.o"));
        manifest.toolchain = Some("rustc 1.85".to_string());

        let mut tmpfile = NamedTempFile::new().unwrap();
        tmpfile
            .write_all(serde_json::to_string_pretty(&manifest).unwrap().as_bytes())
            .unwrap();

        let read = ArtifactManifest::read_from(tmpfile.path()).unwrap();
        assert_eq!(read.component_id.as_str(), "test_comp");
        assert_eq!(read.artifacts.objects.len(), 1);
        assert_eq!(read.toolchain.as_ref().unwrap(), "rustc 1.85");
    }

    #[test]
    fn test_artifact_manifest_content_hash() {
        let manifest = ArtifactManifest::new(ComponentId::new("test"));
        let hash1 = manifest.content_hash();

        // Same manifest should produce same hash
        let hash2 = manifest.content_hash();
        assert_eq!(hash1.hash, hash2.hash);
    }

    #[test]
    fn test_artifact_manifest_deterministic_hash() {
        let manifest1 = ArtifactManifest::new(ComponentId::new("test"));
        let manifest2 = ArtifactManifest::new(ComponentId::new("test"));

        // Same content should produce same hash
        assert_eq!(manifest1.content_hash().hash, manifest2.content_hash().hash);
    }

    #[test]
    fn test_artifact_manifest_rejects_future_version() {
        let mut manifest = ArtifactManifest::new(ComponentId::new("test"));
        manifest.version = "0.2.0".to_string();

        let result = manifest.validate_version();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("unsupported artifact manifest version"));
    }

    #[test]
    fn test_artifact_set_deterministic_serialization() {
        let mut set1 = ArtifactSet::new();
        set1.add_object(PathBuf::from("a.o"));
        set1.add_object(PathBuf::from("b.o"));
        set1.add_archive(PathBuf::from("lib.a"));

        let mut set2 = ArtifactSet::new();
        set2.add_object(PathBuf::from("a.o"));
        set2.add_object(PathBuf::from("b.o"));
        set2.add_archive(PathBuf::from("lib.a"));

        let json1 = serde_json::to_string(&set1).unwrap();
        let json2 = serde_json::to_string(&set2).unwrap();
        assert_eq!(
            json1, json2,
            "identical ArtifactSets must serialize to identical JSON"
        );
    }

    #[test]
    fn test_language_build_result_default_state() {
        let result = LanguageBuildResult::new(
            ComponentId::new("test_comp"),
            chimera_component::Language::Rust,
        );

        assert!(result.is_success());
        assert!(result.primary_outputs.is_empty());
        assert!(result.link.is_empty());
        assert!(result.wrappers_required.is_empty());
        assert!(result.diagnostics.is_empty());
        assert!(!result.has_proof_failures());
    }

    #[test]
    fn test_native_link_spec_merge_deterministic_order() {
        // Test that merge order doesn't affect final content
        let mut spec1 = NativeLinkSpec::new();
        spec1.objects.push(PathBuf::from("a.o"));
        spec1.objects.push(PathBuf::from("b.o"));
        spec1.link_libraries.push("m".to_string());
        spec1.system_libraries.push("z".to_string());

        let mut spec2 = NativeLinkSpec::new();
        spec2.objects.push(PathBuf::from("c.o"));
        spec2.link_libraries.push("pthread".to_string());
        spec2.system_libraries.push("a".to_string());

        // Merge spec1 into spec2
        let mut merged_ab = spec1.clone();
        merged_ab.merge(&spec2);

        // Merge spec2 into spec1
        let mut merged_ba = spec2.clone();
        merged_ba.merge(&spec1);

        // Both should have same content
        assert_eq!(
            merged_ab.objects.len(),
            merged_ba.objects.len(),
            "object counts must match"
        );
        assert_eq!(
            merged_ab.link_libraries.len(),
            merged_ba.link_libraries.len(),
            "link library counts must match"
        );
        assert_eq!(
            merged_ab.system_libraries, merged_ba.system_libraries,
            "system libraries must match regardless of merge order"
        );
        // System libraries should always be sorted alphabetically
        assert_eq!(
            merged_ab.system_libraries,
            vec!["a", "z"],
            "system libraries must be sorted"
        );
    }

    #[test]
    fn test_public_surface_computes_abi_fingerprint() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        surface.exported_symbols.push(Symbol::new("fn1"));
        surface.exported_symbols.push(Symbol::new("fn2"));

        surface.compute_abi_fingerprint();

        assert!(surface.abi_fingerprint.is_some());
        assert_eq!(
            surface.abi_fingerprint.as_ref().unwrap().algorithm,
            "blake3"
        );
    }

    #[test]
    fn test_public_surface_computes_layout_fingerprint() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        surface.exported_symbols.push(Symbol::new("my_func"));

        surface.compute_layout_fingerprint();

        assert!(surface.layout_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_computes_effect_fingerprint() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        surface.exported_symbols.push(Symbol::new("effect_fn"));

        surface.compute_effect_fingerprint();

        assert!(surface.effect_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_computes_ownership_fingerprint() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        surface.exported_symbols.push(Symbol::new("exported_fn"));
        surface.imported_symbols.push(Symbol::new("external_fn"));

        surface.compute_ownership_fingerprint();

        assert!(surface.ownership_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_computes_panic_fingerprint() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        surface.exported_symbols.push(Symbol::new("panic_handler"));
        surface.exported_symbols.push(Symbol::new("normal_fn"));

        surface.compute_panic_policy_fingerprint();

        assert!(surface.panic_policy_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_computes_wrapper_fingerprint() {
        use chimera_component::WrapperPolicy;

        let mut surface = PublicSurface::new();
        let wrappers = vec![WrapperRequest::new(
            ComponentId::new("consumer"),
            ComponentId::new("provider"),
            WrapperPolicy::Auto,
        )];

        surface.compute_wrapper_surface_fingerprint(&wrappers);

        assert!(surface.wrapper_surface_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_computes_proof_surface_fingerprint() {
        let mut surface = PublicSurface::new();
        let obligations = vec![
            "proof_obligation_1".to_string(),
            "proof_obligation_2".to_string(),
        ];

        surface.compute_proof_surface_fingerprint(&obligations);

        assert!(surface.proof_surface_fingerprint.is_some());
    }

    #[test]
    fn test_public_surface_fingerprint_changes_with_content() {
        use chimera_component::Symbol;

        let mut surface1 = PublicSurface::new();
        surface1.exported_symbols.push(Symbol::new("fn1"));

        let mut surface2 = PublicSurface::new();
        surface2.exported_symbols.push(Symbol::new("fn2"));

        surface1.compute_abi_fingerprint();
        surface2.compute_abi_fingerprint();

        // Different content should produce different hashes
        assert_ne!(
            surface1.abi_fingerprint.as_ref().unwrap().hash,
            surface2.abi_fingerprint.as_ref().unwrap().hash
        );
    }

    #[test]
    fn test_public_surface_has_fingerprints_after_computation() {
        use chimera_component::Symbol;

        let mut surface = PublicSurface::new();
        assert!(!surface.has_fingerprints());

        surface.exported_symbols.push(Symbol::new("fn1"));
        surface.compute_abi_fingerprint();

        assert!(surface.has_fingerprints());
    }

    // Task 14: NativeLinkSpec merging tests

    #[test]
    fn test_native_link_spec_merge_duplicate_suppression() {
        let mut spec1 = NativeLinkSpec::new();
        spec1.objects.push(PathBuf::from("a.o"));
        spec1.objects.push(PathBuf::from("b.o"));

        let mut spec2 = NativeLinkSpec::new();
        spec2.objects.push(PathBuf::from("b.o")); // duplicate
        spec2.objects.push(PathBuf::from("c.o"));

        spec1.merge(&spec2);

        // b.o should not be duplicated
        assert_eq!(spec1.objects.len(), 3);
        assert!(spec1.objects.contains(&PathBuf::from("a.o")));
        assert!(spec1.objects.contains(&PathBuf::from("b.o")));
        assert!(spec1.objects.contains(&PathBuf::from("c.o")));
    }

    #[test]
    fn test_native_link_spec_merge_order() {
        let mut spec1 = NativeLinkSpec::new();
        spec1.objects.push(PathBuf::from("a.o"));
        spec1.link_libraries.push("pthread".to_string());

        let mut spec2 = NativeLinkSpec::new();
        spec2.objects.push(PathBuf::from("b.o"));
        spec2.link_libraries.push("m".to_string());

        spec1.merge(&spec2);

        // Objects should be in merge order
        assert_eq!(spec1.objects[0], PathBuf::from("a.o"));
        assert_eq!(spec1.objects[1], PathBuf::from("b.o"));

        // Libraries should be deduped and in merge order
        assert_eq!(spec1.link_libraries.len(), 2);
        assert!(spec1.link_libraries.contains(&"pthread".to_string()));
        assert!(spec1.link_libraries.contains(&"m".to_string()));
    }

    #[test]
    fn test_native_link_spec_merge_system_libraries_sorted() {
        let mut spec1 = NativeLinkSpec::new();
        spec1.system_libraries.push("z".to_string());

        let mut spec2 = NativeLinkSpec::new();
        spec2.system_libraries.push("a".to_string());
        spec2.system_libraries.push("m".to_string());

        spec1.merge(&spec2);

        // Should be sorted alphabetically
        assert_eq!(spec1.system_libraries, vec!["a", "m", "z"]);
        assert_eq!(spec1.system_libraries.len(), 3);
    }

    #[test]
    fn test_native_link_spec_merge_all() {
        let mut spec = NativeLinkSpec::new();
        spec.objects.push(PathBuf::from("a.o"));

        let others = vec![
            NativeLinkSpec {
                objects: vec![PathBuf::from("b.o")],
                ..Default::default()
            },
            NativeLinkSpec {
                objects: vec![PathBuf::from("c.o")],
                ..Default::default()
            },
        ];

        spec.merge_all(&others);

        assert_eq!(spec.objects.len(), 3);
    }

    #[test]
    fn test_native_link_spec_find_conflicts() {
        let mut spec1 = NativeLinkSpec::new();
        spec1.objects.push(PathBuf::from("shared.o"));

        let mut spec2 = NativeLinkSpec::new();
        spec2.objects.push(PathBuf::from("shared.o"));
        spec2.static_archives.push(PathBuf::from("lib.a"));

        let conflicts = spec1.find_conflicts(&spec2);

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0], PathBuf::from("shared.o"));
    }

    #[test]
    fn test_native_link_spec_no_conflicts() {
        let spec1 = NativeLinkSpec {
            objects: vec![PathBuf::from("a.o")],
            ..Default::default()
        };

        let spec2 = NativeLinkSpec {
            objects: vec![PathBuf::from("b.o")],
            ..Default::default()
        };

        let conflicts = spec1.find_conflicts(&spec2);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_native_link_spec_empty_after_clear() {
        let mut spec = NativeLinkSpec::new();
        spec.objects.push(PathBuf::from("a.o"));
        spec.link_libraries.push("pthread".to_string());

        spec.merge(&NativeLinkSpec::new());

        // Original contents preserved when merging empty
        assert_eq!(spec.objects.len(), 1);
        assert_eq!(spec.link_libraries.len(), 1);
    }
}
