use chimera_artifact::{
    ArtifactSet, Diagnostic, DiagnosticSeverity, Fingerprint, LanguageBuildResult,
    MetadataArtifacts, NativeLinkSpec, ProofArtifacts, PublicSurface,
};
use chimera_component::{ComponentId, Language, Symbol};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zigmera_schema::zsnap::SnapSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZigBuildOutput {
    pub object_files: Vec<PathBuf>,
    pub metadata_files: Vec<PathBuf>,
    pub dep_graph_files: Vec<PathBuf>,
    pub airpack_files: Vec<PathBuf>,
    pub chmeta_files: Vec<PathBuf>,
    pub proof_files: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
}

impl ZigBuildOutput {
    pub fn new() -> Self {
        Self {
            object_files: Vec::new(),
            metadata_files: Vec::new(),
            dep_graph_files: Vec::new(),
            airpack_files: Vec::new(),
            chmeta_files: Vec::new(),
            proof_files: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn has_object_files(&self) -> bool {
        !self.object_files.is_empty()
    }

    pub fn object_count(&self) -> usize {
        self.object_files.len()
    }
}

impl Default for ZigBuildOutput {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_result_from_output(
    output: ZigBuildOutput,
    component_id: ComponentId,
) -> LanguageBuildResult {
    let mut result = LanguageBuildResult::new(component_id, Language::Zig);

    for obj in output.object_files {
        result.primary_outputs.add_object(obj.clone());
        if obj.extension().and_then(|e| e.to_str()) == Some("a") {
            result.primary_outputs.add_archive(obj);
        }
    }

    for snap in &output.metadata_files {
        result.metadata.zsnap.push(snap.clone());
    }
    for dep in &output.dep_graph_files {
        result.metadata.zdep.push(dep.clone());
    }
    for air in &output.airpack_files {
        result.metadata.zairpack.push(air.clone());
    }
    for chm in &output.chmeta_files {
        result.metadata.chmeta.push(chm.clone());
    }
    for prof in &output.proof_files {
        result.proof.chproof.push(prof.clone());
    }

    for diag in output.diagnostics {
        result.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Info,
            code: "ZIG".to_string(),
            message: diag,
            location: None,
            suggestions: Vec::new(),
        });
    }

    result
}

pub fn build_result_from_snapshot(
    snapshot: &SnapSchema,
    object_files: Vec<PathBuf>,
    component_id: ComponentId,
) -> LanguageBuildResult {
    let mut output = ZigBuildOutput::new();
    output.object_files = object_files;
    output.metadata_files = vec![PathBuf::from(format!("{}.zsnap", component_id))];
    output.dep_graph_files = vec![PathBuf::from(format!("{}.zdep", component_id))];

    build_result_from_output(output, component_id)
}

pub fn emit_artifact_manifest(
    output: &ZigBuildOutput,
    component_id: ComponentId,
) -> Result<chimera_artifact::ArtifactManifest, ArtifactEmitError> {
    let mut manifest = chimera_artifact::ArtifactManifest::new(component_id);
    for obj in &output.object_files {
        manifest.artifacts.add_object(obj.clone());
    }
    for snap in &output.metadata_files {
        manifest.artifacts.snapshots.push(snap.clone());
    }
    Ok(manifest)
}

fn _compute_file_hash(path: &Path) -> Result<String, ArtifactEmitError> {
    use blake3::Hasher;
    let data = std::fs::read(path).map_err(|e| ArtifactEmitError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let mut hasher = Hasher::new();
    hasher.update(&data);
    Ok(hasher.finalize().to_hex().to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactEmitError {
    #[error("I/O error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub fn public_surface_from_snapshot(snapshot: &SnapSchema) -> PublicSurface {
    let exports: Vec<Symbol> = snapshot
        .exports
        .iter()
        .map(|e| Symbol::new(e.name.clone()))
        .collect();

    let imports: Vec<Symbol> = snapshot
        .decls
        .iter()
        .filter(|d| matches!(d.kind, zigmera_schema::zsnap::DeclKind::Import))
        .map(|d| Symbol::new(d.name.clone()))
        .collect();

    let checksum = snapshot.compute_checksum();

    PublicSurface {
        abi_fingerprint: Some(Fingerprint::new("blake3", format!("{:x?}", checksum))),
        layout_fingerprint: None,
        effect_fingerprint: None,
        ownership_fingerprint: None,
        panic_policy_fingerprint: Some(Fingerprint::new("string", "unwind")),
        proof_surface_fingerprint: None,
        wrapper_surface_fingerprint: None,
        exported_symbols: exports,
        imported_symbols: imports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_output_creation() {
        let output = ZigBuildOutput::new();
        assert!(!output.has_object_files());
        assert_eq!(output.object_count(), 0);
    }

    #[test]
    fn test_build_output_with_objects() {
        let mut output = ZigBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.object_files.push(PathBuf::from("lib.a"));
        assert!(output.has_object_files());
        assert_eq!(output.object_count(), 2);
    }

    #[test]
    fn test_build_result_from_output_basic() {
        let mut output = ZigBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.metadata_files.push(PathBuf::from("snap.zsnap"));
        output.diagnostics.push("no issues".to_string());

        let result = build_result_from_output(output, ComponentId::new("test-component"));
        assert!(result.is_success());
        assert!(!result.primary_outputs.objects.is_empty());
        assert!(!result.metadata.zsnap.is_empty());
    }

    #[test]
    fn test_emit_artifact_manifest() {
        let mut output = ZigBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.metadata_files.push(PathBuf::from("snap.zsnap"));

        let manifest = emit_artifact_manifest(&output, ComponentId::new("test-comp")).unwrap();
        assert_eq!(manifest.component_id.to_string(), "test-comp");
        assert!(!manifest.artifacts.objects.is_empty());
    }

    #[test]
    fn test_public_surface_from_snapshot() {
        use zigmera_schema::zsnap::{
            AccessLevel, DeclKind, DeclRef, ExportSymbol, Linkage, Visibility,
        };

        let mut snapshot = SnapSchema::default();
        snapshot.exports = vec![ExportSymbol {
            name: "zig_main".to_string(),
            decl_id: 0,
            linkage: Linkage::Strong,
            visibility: Visibility::Exported,
            callconv: 0,
            section_hint: None,
        }];
        snapshot.decls = vec![DeclRef {
            id: 1,
            name: "some_import".to_string(),
            kind: DeclKind::Import,
            owner_file: 0,
            access_level: AccessLevel::Pub,
        }];

        let surface = public_surface_from_snapshot(&snapshot);
        assert!(surface
            .exported_symbols
            .iter()
            .any(|s| s.name == "zig_main"));
        assert!(surface
            .imported_symbols
            .iter()
            .any(|s| s.name == "some_import"));
    }

    #[test]
    fn test_build_result_from_snapshot() {
        let snapshot = SnapSchema::default();
        let objects = vec![PathBuf::from("main.o")];
        let result = build_result_from_snapshot(&snapshot, objects, ComponentId::new("test-comp"));
        assert!(result.is_success());
        assert_eq!(result.primary_outputs.objects.len(), 1);
    }
}
