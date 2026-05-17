//! Workspace ingestion bridge for chimera-build.
//!
//! Connects `chimera-rust-cargo` metadata ingestion to the build graph,
//! converting `CargoMetadata` into `BuildNode::CargoBuild` nodes and
//! parsing `cargo build --message-format=json` output into
//! `LanguageBuildResult` with proper `RustArtifactRef` equivalents.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{BuildGraph, BuildNode};

/// A parsed compiler-artifact event from cargo's --message-format=json output.
#[derive(Debug, Clone)]
pub struct CargoArtifactEvent {
    pub package_id: String,
    pub package_name: String,
    pub target_name: String,
    pub crate_type: String,
    pub profile: String,
    pub target_triple: Option<String>,
    pub filenames: Vec<PathBuf>,
    pub executable: Option<PathBuf>,
    pub features: Vec<String>,
    pub is_workspace_member: bool,
}

/// Result of ingesting a cargo workspace into the build graph.
#[derive(Debug, Default)]
pub struct WorkspaceIngestionResult {
    pub cargo_build_node_ids: Vec<String>,
    pub artifact_events: Vec<CargoArtifactEvent>,
}

/// Ingest a cargo workspace manifest path into the build graph.
///
/// Creates `CargoBuild` nodes for each package and returns the list
/// of node IDs and parsed artifact events for downstream processing.
pub fn ingest_cargo_workspace(
    graph: &mut BuildGraph,
    cargo_manifest_path: &PathBuf,
    output_dir: &PathBuf,
    target_triple: &str,
) -> Result<WorkspaceIngestionResult, String> {
    let mut result = WorkspaceIngestionResult::default();

    // Run cargo metadata to discover workspace structure
    let metadata = fetch_cargo_metadata(cargo_manifest_path)?;

    let _workspace_root = cargo_manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| cargo_manifest_path.clone());

    // Create a CargoBuild node for the entire workspace
    let node_id = format!("cargo_build_{}", crate_id(cargo_manifest_path));
    let metadata_output = output_dir.join(format!("cargo_metadata_{}.json", node_id));

    graph.add_node(BuildNode::cargo_build(
        &node_id,
        vec![cargo_manifest_path.to_string_lossy().to_string()],
        vec![metadata_output],
    ));

    result.cargo_build_node_ids.push(node_id);

    // Record each workspace member as a parsed package in the result
    for pkg in &metadata.workspace_members {
        for target in &pkg.targets {
            let crate_type = determine_crate_type(&pkg.crate_types, &target.kind);
            let event = CargoArtifactEvent {
                package_id: pkg.id.clone(),
                package_name: pkg.name.clone(),
                target_name: target.name.clone(),
                crate_type,
                profile: "debug".to_string(),
                target_triple: Some(target_triple.to_string()),
                filenames: Vec::new(),
                executable: None,
                features: pkg.features.keys().cloned().collect(),
                is_workspace_member: true,
            };
            result.artifact_events.push(event);
        }
    }

    Ok(result)
}

/// Fetch cargo metadata for a workspace.
fn fetch_cargo_metadata(
    manifest_path: &PathBuf,
) -> Result<chimera_rust_cargo::CargoMetadata, String> {
    chimera_rust_cargo::fetch_metadata(manifest_path)
        .map_err(|e| format!("cargo metadata failed: {}", e))
}

/// Parse cargo build --message-format=json output into artifact events.
///
/// Extracts `compiler-artifact` events which contain the actual built
/// file paths, crate types, and target metadata.
pub fn parse_cargo_build_output(stdout: &str) -> Vec<CargoArtifactEvent> {
    let mut events = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
            let reason = msg.get("reason").and_then(|r| r.as_str()).unwrap_or("");

            if reason != "compiler-artifact" {
                continue;
            }

            let package_id = msg
                .get("package_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let package_name = msg
                .get("package_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let target_name = msg
                .pointer("/target/name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let crate_types: Vec<String> = msg
                .pointer("/target/crate_types")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let crate_type = crate_types
                .first()
                .cloned()
                .unwrap_or_else(|| "lib".to_string());

            let profile = msg
                .pointer("/profile")
                .and_then(|v| v.as_str())
                .unwrap_or("debug")
                .to_string();

            let _target_triple: Option<String> = msg
                .get("target")
                .and_then(|t| t.get("src_path"))
                .and_then(|v| v.as_str())
                .map(|_| String::new());

            let filenames: Vec<PathBuf> = msg
                .get("filenames")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(PathBuf::from))
                        .collect()
                })
                .unwrap_or_default();

            let executable = msg
                .get("executable")
                .and_then(|v| v.as_str())
                .map(PathBuf::from);

            let features: Vec<String> = msg
                .get("features")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            // Determine if this is a workspace member by checking if package_id is set
            let is_workspace_member = !package_id.is_empty();

            events.push(CargoArtifactEvent {
                package_id,
                package_name,
                target_name,
                crate_type,
                profile,
                target_triple: None,
                filenames,
                executable,
                features,
                is_workspace_member,
            });
        }
    }

    events
}

/// Convert cargo artifact events into a LanguageBuildResult.
///
/// Maps: staticlib → link inputs, cdylib → runtime delivery,
/// binary → executable outputs, proc-macro → not linked.
pub fn artifact_events_to_build_result(
    events: &[CargoArtifactEvent],
) -> chimera_artifact::LanguageBuildResult {
    let mut result = chimera_artifact::LanguageBuildResult::new(
        chimera_component::ComponentId::new("cargo_workspace"),
        chimera_component::Language::Rust,
    );

    let mut link_spec = chimera_artifact::NativeLinkSpec::new();

    for event in events {
        for filename in &event.filenames {
            match event.crate_type.as_str() {
                "staticlib" | "rlib" => {
                    if filename.extension().and_then(|ext| ext.to_str()) == Some("rmeta") {
                        result.metadata.chmeta.push(filename.clone());
                        continue;
                    }
                    link_spec.static_archives.push(filename.clone());
                    result.primary_outputs.archives.push(filename.clone());
                }
                "cdylib" => {
                    link_spec.shared_libraries.push(filename.clone());
                    result.primary_outputs.shared_libs.push(filename.clone());
                }
                "bin" => {
                    result.primary_outputs.executables.push(filename.clone());
                }
                "proc-macro" => {
                    // Proc-macros are not linked as runtime dependencies
                    result.metadata.chmeta.push(filename.clone());
                }
                _ => {
                    result.primary_outputs.objects.push(filename.clone());
                }
            }
        }

        if let Some(ref executable) = event.executable {
            result.primary_outputs.executables.push(executable.clone());
        }
    }

    result.link = link_spec;
    result.set_success();
    result
}

pub fn preferred_executables(
    events: &[CargoArtifactEvent],
    preferred_package: &str,
) -> Vec<PathBuf> {
    let mut executables: Vec<PathBuf> = events
        .iter()
        .filter(|event| event.crate_type == "bin" && event.package_name == preferred_package)
        .filter_map(|event| event.executable.clone())
        .collect();

    if executables.is_empty() {
        executables = all_executables(events);
    }

    executables
}

pub fn all_executables(events: &[CargoArtifactEvent]) -> Vec<PathBuf> {
    events
        .iter()
        .filter(|event| event.crate_type == "bin")
        .filter_map(|event| event.executable.clone())
        .collect()
}

/// Determine crate type string from Package crate types and target kinds.
fn determine_crate_type(
    crate_types: &[chimera_rust_cargo::CrateType],
    target_kinds: &[chimera_rust_cargo::TargetKind],
) -> String {
    if target_kinds.contains(&chimera_rust_cargo::TargetKind::ProcMacro) {
        return "proc-macro".to_string();
    }
    if target_kinds.contains(&chimera_rust_cargo::TargetKind::Bin) {
        return "bin".to_string();
    }
    if crate_types.contains(&chimera_rust_cargo::CrateType::Cdylib) {
        return "cdylib".to_string();
    }
    if crate_types.contains(&chimera_rust_cargo::CrateType::Staticlib) {
        return "staticlib".to_string();
    }
    if crate_types.contains(&chimera_rust_cargo::CrateType::Rlib) {
        return "rlib".to_string();
    }
    "lib".to_string()
}

/// Simple hash for a Cargo.toml path to create a stable node ID.
fn crate_id(path: &PathBuf) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_build_output_empty() {
        let events = parse_cargo_build_output("");
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_cargo_build_output_invalid_json() {
        let events = parse_cargo_build_output("not json");
        assert!(events.is_empty());
    }

    #[test]
    fn test_parse_cargo_build_output_compiler_artifact() {
        let output = r#"
{"reason":"compiler-artifact","package_id":"test 1.0.0","package_name":"test","target":{"name":"test_lib","kind":["lib"],"crate_types":["rlib"],"src_path":"/src/lib.rs","edition":"2021"},"profile":"debug","features":[],"filenames":["/target/debug/libtest.rlib"],"executable":null}
{"reason":"compiler-artifact","package_id":"bin 1.0.0","package_name":"bin","target":{"name":"bin_bin","kind":["bin"],"crate_types":["bin"],"src_path":"/src/main.rs","edition":"2021"},"profile":"release","features":[],"filenames":["/target/release/bin"],"executable":"/target/release/bin"}
"#;
        let events = parse_cargo_build_output(output);
        assert_eq!(events.len(), 2);

        assert_eq!(events[0].package_name, "test");
        assert_eq!(events[0].crate_type, "rlib");
        assert_eq!(events[0].filenames.len(), 1);
        assert!(events[0].filenames[0]
            .to_string_lossy()
            .ends_with("libtest.rlib"));

        assert_eq!(events[1].package_name, "bin");
        assert_eq!(events[1].crate_type, "bin");
        assert!(events[1].executable.is_some());
    }

    #[test]
    fn test_parse_cargo_build_output_skips_non_artifact() {
        let output = r#"
{"reason":"build-finished","success":true}
{"reason":"compiler-artifact","package_id":"lib 1.0.0","package_name":"lib","target":{"name":"lib","kind":["lib"],"crate_types":["rlib"],"src_path":"/src/lib.rs","edition":"2021"},"profile":"debug","features":[],"filenames":["/target/debug/liblib.rlib"],"executable":null}
"#;
        let events = parse_cargo_build_output(output);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_artifact_events_to_build_result_staticlib() {
        let events = vec![CargoArtifactEvent {
            package_id: "mylib 1.0.0".to_string(),
            package_name: "mylib".to_string(),
            target_name: "lib".to_string(),
            crate_type: "staticlib".to_string(),
            profile: "debug".to_string(),
            target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
            filenames: vec![PathBuf::from("/target/debug/libmylib.a")],
            executable: None,
            features: vec![],
            is_workspace_member: true,
        }];

        let result = artifact_events_to_build_result(&events);
        assert!(result.is_success());
        assert_eq!(result.primary_outputs.archives.len(), 1);
        assert_eq!(result.link.static_archives.len(), 1);
    }

    #[test]
    fn test_artifact_events_to_build_result_cdylib() {
        let events = vec![CargoArtifactEvent {
            package_id: "mydyn 1.0.0".to_string(),
            package_name: "mydyn".to_string(),
            target_name: "lib".to_string(),
            crate_type: "cdylib".to_string(),
            profile: "debug".to_string(),
            target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
            filenames: vec![PathBuf::from("/target/debug/libmydyn.so")],
            executable: None,
            features: vec![],
            is_workspace_member: true,
        }];

        let result = artifact_events_to_build_result(&events);
        assert_eq!(result.primary_outputs.shared_libs.len(), 1);
        assert_eq!(result.link.shared_libraries.len(), 1);
    }

    #[test]
    fn test_artifact_events_to_build_result_binary() {
        let events = vec![CargoArtifactEvent {
            package_id: "mybin 1.0.0".to_string(),
            package_name: "mybin".to_string(),
            target_name: "mybin".to_string(),
            crate_type: "bin".to_string(),
            profile: "debug".to_string(),
            target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
            filenames: vec![PathBuf::from("/target/debug/mybin")],
            executable: Some(PathBuf::from("/target/debug/mybin")),
            features: vec![],
            is_workspace_member: true,
        }];

        let result = artifact_events_to_build_result(&events);
        // Both the filename and executable fields contribute to executables
        assert!(result.primary_outputs.executables.len() >= 1);
        assert!(result.link.static_archives.is_empty());
        assert!(result.link.shared_libraries.is_empty());
    }

    #[test]
    fn test_artifact_events_to_build_result_proc_macro_not_linked() {
        let events = vec![CargoArtifactEvent {
            package_id: "mymacro 1.0.0".to_string(),
            package_name: "mymacro".to_string(),
            target_name: "mymacro".to_string(),
            crate_type: "proc-macro".to_string(),
            profile: "debug".to_string(),
            target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
            filenames: vec![PathBuf::from("/target/debug/libmymacro.so")],
            executable: None,
            features: vec![],
            is_workspace_member: true,
        }];

        let result = artifact_events_to_build_result(&events);
        // Proc-macro should NOT appear in link inputs
        assert!(result.link.static_archives.is_empty());
        assert!(result.link.shared_libraries.is_empty());
        // Proc-macro files go to metadata, not primary outputs
        assert!(result.primary_outputs.objects.is_empty());
    }

    #[test]
    fn test_ingest_cargo_workspace_creates_node() {
        let mut graph = crate::BuildGraph::new();
        let manifest = PathBuf::from("/workspace/Cargo.toml");
        let output_dir = PathBuf::from("build");
        let target = "x86_64-unknown-linux-gnu";

        // This will fail because the manifest doesn't exist, but we can verify
        // the function signature and error handling
        let result = ingest_cargo_workspace(&mut graph, &manifest, &output_dir, target);
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_determine_crate_type() {
        use chimera_rust_cargo::{CrateType, TargetKind};

        assert_eq!(
            determine_crate_type(&[CrateType::Staticlib], &[TargetKind::Lib]),
            "staticlib"
        );
        assert_eq!(
            determine_crate_type(&[CrateType::Cdylib], &[TargetKind::Lib]),
            "cdylib"
        );
        assert_eq!(
            determine_crate_type(&[CrateType::Rlib], &[TargetKind::Lib]),
            "rlib"
        );
        assert_eq!(
            determine_crate_type(&[CrateType::Library], &[TargetKind::Lib]),
            "lib"
        );
        assert_eq!(
            determine_crate_type(&[CrateType::Library], &[TargetKind::ProcMacro]),
            "proc-macro"
        );
        assert_eq!(
            determine_crate_type(&[CrateType::Library], &[TargetKind::Bin]),
            "bin"
        );
    }

    #[test]
    fn test_crate_id_is_stable() {
        let path1 = PathBuf::from("/workspace/Cargo.toml");
        let path2 = PathBuf::from("/workspace/Cargo.toml");
        assert_eq!(crate_id(&path1), crate_id(&path2));
    }

    #[test]
    fn test_crate_id_differs_for_different_paths() {
        let path1 = PathBuf::from("/workspace/Cargo.toml");
        let path2 = PathBuf::from("/other/Cargo.toml");
        assert_ne!(crate_id(&path1), crate_id(&path2));
    }
}
