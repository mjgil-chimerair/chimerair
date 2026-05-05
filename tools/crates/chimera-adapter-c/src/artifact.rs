use chimera_artifact::{
    Diagnostic, DiagnosticSeverity, Fingerprint, LanguageBuildResult, NativeLinkSpec, PublicSurface,
};
use chimera_c_schema::CsnapSnapshot;
use chimera_component::{ComponentId, Language, Symbol};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBuildOutput {
    pub object_files: Vec<PathBuf>,
    pub metadata_files: Vec<PathBuf>,
    pub dep_graph_files: Vec<PathBuf>,
    pub chmeta_files: Vec<PathBuf>,
    pub headers: Vec<PathBuf>,
    pub sources: Vec<PathBuf>,
    pub compile_commands: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
    pub link_spec: NativeLinkSpec,
}

impl CBuildOutput {
    pub fn new() -> Self {
        Self {
            object_files: Vec::new(),
            metadata_files: Vec::new(),
            dep_graph_files: Vec::new(),
            chmeta_files: Vec::new(),
            headers: Vec::new(),
            sources: Vec::new(),
            compile_commands: Vec::new(),
            diagnostics: Vec::new(),
            link_spec: NativeLinkSpec::new(),
        }
    }

    pub fn has_object_files(&self) -> bool {
        !self.object_files.is_empty()
    }

    pub fn object_count(&self) -> usize {
        self.object_files.len()
    }
}

impl Default for CBuildOutput {
    fn default() -> Self {
        Self::new()
    }
}

pub fn build_result_from_output(
    output: CBuildOutput,
    component_id: ComponentId,
) -> LanguageBuildResult {
    let mut result = LanguageBuildResult::new(component_id, Language::C);

    for obj in output.object_files {
        result.primary_outputs.add_object(obj.clone());
        if obj.extension().and_then(|e| e.to_str()) == Some("a") {
            result.primary_outputs.add_archive(obj);
        }
    }

    for snap in &output.metadata_files {
        result.metadata.csnap.push(snap.clone());
    }
    for dep in &output.dep_graph_files {
        result.metadata.cdep.push(dep.clone());
    }
    for chm in &output.chmeta_files {
        result.metadata.chmeta.push(chm.clone());
    }
    for cmd in &output.compile_commands {
        result.metadata.compile_commands.push(cmd.clone());
    }

    // Assign native link spec
    result.link = output.link_spec;

    for diag in output.diagnostics {
        result.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Info,
            code: "C".to_string(),
            message: diag,
            location: None,
            suggestions: Vec::new(),
        });
    }

    result
}

pub fn build_result_from_snapshot(
    _snapshot: &CsnapSnapshot,
    object_files: Vec<PathBuf>,
    component_id: ComponentId,
) -> LanguageBuildResult {
    let mut output = CBuildOutput::new();
    output.object_files = object_files;
    output.metadata_files = vec![PathBuf::from(format!("{}.csnap", component_id))];
    output.dep_graph_files = vec![PathBuf::from(format!("{}.cdep", component_id))];

    build_result_from_output(output, component_id)
}

pub fn emit_artifact_manifest(
    output: &CBuildOutput,
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

pub fn public_surface_from_snapshot(snapshot: &CsnapSnapshot) -> PublicSurface {
    let exports: Vec<Symbol> = snapshot
        .exports
        .iter()
        .map(|e| Symbol::new(e.symbol.clone()))
        .collect();

    let imports: Vec<Symbol> = snapshot
        .imports
        .iter()
        .map(|e| Symbol::new(e.symbol.clone()))
        .collect();

    let checksum = snapshot.compute_checksum();

    let mut surface = PublicSurface {
        abi_fingerprint: Some(Fingerprint::new("blake3", format!("{:x?}", checksum))),
        layout_fingerprint: None,
        effect_fingerprint: None,
        ownership_fingerprint: None,
        panic_policy_fingerprint: None,
        proof_surface_fingerprint: None,
        wrapper_surface_fingerprint: None,
        exported_symbols: exports,
        imported_symbols: imports,
    };

    // We compute the remaining fingerprints from exported symbols
    surface.compute_layout_fingerprint();
    surface.compute_ownership_fingerprint();

    surface
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_c_schema::{
        ArtifactHeader, CTarget, CsnapSnapshot, DeclId, ExportSymbol, ImportSymbol, Linkage,
        SourceLanguage,
    };

    #[test]
    fn test_build_output_creation() {
        let output = CBuildOutput::new();
        assert!(!output.has_object_files());
        assert_eq!(output.object_count(), 0);
    }

    #[test]
    fn test_build_output_with_objects() {
        let mut output = CBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.object_files.push(PathBuf::from("lib.a"));
        assert!(output.has_object_files());
        assert_eq!(output.object_count(), 2);
    }

    #[test]
    fn test_build_result_from_output_basic() {
        let mut output = CBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.metadata_files.push(PathBuf::from("snap.csnap"));
        output.diagnostics.push("no issues".to_string());

        let result = build_result_from_output(output, ComponentId::new("test-component"));
        assert!(result.is_success());
        assert!(!result.primary_outputs.objects.is_empty());
        assert!(!result.metadata.csnap.is_empty());
    }

    #[test]
    fn test_emit_artifact_manifest() {
        let mut output = CBuildOutput::new();
        output.object_files.push(PathBuf::from("main.o"));
        output.metadata_files.push(PathBuf::from("snap.csnap"));

        let manifest = emit_artifact_manifest(&output, ComponentId::new("test-comp")).unwrap();
        assert_eq!(manifest.component_id.to_string(), "test-comp");
        assert!(!manifest.artifacts.objects.is_empty());
    }

    #[test]
    fn test_public_surface_from_snapshot() {
        let mut snapshot = CsnapSnapshot {
            header: ArtifactHeader {
                magic: [0x43, 0x48, 0x49, 0x52],
                schema_version: 1,
                producer_version: "0.1.0".to_string(),
                target: "x86_64-linux".to_string(),
                source_language: "c".to_string(),
            },
            checksum: "0000".to_string(),
            clang_version: "17".to_string(),
            target: CTarget {
                triple: "x86_64-linux".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: chimera_c_schema::CStandard::C17,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };
        snapshot.exports = vec![ExportSymbol {
            symbol: "c_main".to_string(),
            decl_id: DeclId(0),
            linkage: Linkage::External,
            abi: "C".to_string(),
        }];
        snapshot.imports = vec![ImportSymbol {
            symbol: "some_import".to_string(),
            signature: "void()".to_string(),
            source_lang: SourceLanguage::C,
            abi: "C".to_string(),
        }];

        let surface = public_surface_from_snapshot(&snapshot);
        assert!(surface.exported_symbols.iter().any(|s| s.name == "c_main"));
        assert!(surface
            .imported_symbols
            .iter()
            .any(|s| s.name == "some_import"));
    }

    #[test]
    fn test_build_result_from_snapshot() {
        let snapshot = CsnapSnapshot {
            header: ArtifactHeader {
                magic: [0x43, 0x48, 0x49, 0x52],
                schema_version: 1,
                producer_version: "0.1.0".to_string(),
                target: "x86_64-linux".to_string(),
                source_language: "c".to_string(),
            },
            checksum: "0000".to_string(),
            clang_version: "17".to_string(),
            target: CTarget {
                triple: "x86_64-linux".to_string(),
                arch: "x86_64".to_string(),
                os: "linux".to_string(),
                env: "gnu".to_string(),
                libc: Some("glibc".to_string()),
                clang_version: Some("17".to_string()),
                resource_dir: None,
                sysroot: None,
                pointer_width: 64,
                size_of_ptr: 8,
                size_of_int: 4,
                size_of_long: 8,
                size_of_long_long: 8,
                size_of_float: 4,
                size_of_double: 8,
                size_of_long_double: 16,
                size_of_void: 1,
                int64_aligned: 8,
                long_long_aligned: 8,
                double_aligned: 8,
                long_double_aligned: 16,
                long_double_size: 16,
                big_endian: false,
                c_standard: chimera_c_schema::CStandard::C17,
                clang_trust_facts: vec![],
            },
            headers: vec![],
            source_files: vec![],
            declarations: vec![],
            exports: vec![],
            imports: vec![],
            compile_flags: vec![],
            active_macros: vec![],
            conditional_branches: vec![],
        };
        let objects = vec![PathBuf::from("main.o")];
        let result = build_result_from_snapshot(&snapshot, objects, ComponentId::new("test-comp"));
        assert!(result.is_success());
        assert_eq!(result.primary_outputs.objects.len(), 1);
    }
}
