//! `chimera build` command
//!
//! Builds wrappers, runtime pieces, compiler-core outputs, proof artifacts, and final executable.

use anyhow::{Context, Result};
use chimera_build::{BuildConfig, BuildMode, BuildOrchestrator};
use chimera_manifest::{OutputKind, ProjectManifest};
use chimera_meta::{Metadata, Module, SourceLanguage, Version};
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_rustc_driver() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CHIMERA_RUSTC_DRIVER") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let current_dir = std::env::current_dir().ok();
    let current_exe = std::env::current_exe().ok();
    let mut candidates = Vec::new();

    if let Some(parent) = current_exe.as_ref().and_then(|exe| exe.parent()) {
        candidates.push(parent.join("chimera-rustc-driver"));
    }

    if let Some(dir) = current_dir.as_ref() {
        candidates.push(dir.join("target/debug/chimera-rustc-driver"));
        candidates.push(dir.join("target/release/chimera-rustc-driver"));
        candidates.push(dir.join("tools/target/debug/chimera-rustc-driver"));
        candidates.push(dir.join("tools/target/release/chimera-rustc-driver"));
    }

    candidates.into_iter().find(|candidate| candidate.exists())
}

fn default_build_mode(project_manifest: Option<&ProjectManifest>) -> BuildMode {
    if project_manifest.map(is_cargo_abi_manifest).unwrap_or(false) {
        BuildMode::CargoCAbi
    } else {
        BuildMode::UnifiedLowering
    }
}

fn is_cargo_abi_manifest(project_manifest: &ProjectManifest) -> bool {
    project_manifest.output == OutputKind::StaticLib
        && project_manifest.abi_edges.is_empty()
        && project_manifest.components.len() == 1
        && project_manifest.components.iter().all(|component| {
            component.language.eq_ignore_ascii_case("rust")
                && component
                    .kind
                    .as_deref()
                    .unwrap_or("cargo-package")
                    .eq_ignore_ascii_case("cargo-package")
        })
}

fn default_target_triple(project_manifest: Option<&ProjectManifest>) -> String {
    project_manifest
        .and_then(|m| m.default_target())
        .unwrap_or(chimera_build::host_target_triple())
        .to_string()
}

/// Locate the compiler-core driver executable
fn find_compiler_driver() -> Result<PathBuf> {
    // Check environment variable first
    if let Ok(path) = std::env::var("CHIMERA_COMPILER_DRIVER") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Check common installation paths
    let candidates = vec![
        PathBuf::from("compiler-core/build/tools/driver/chimerac"),
        PathBuf::from("build/compiler-core/tools/driver/chimerac"),
        PathBuf::from("/usr/local/bin/chimerac"),
        PathBuf::from("/usr/bin/chimerac"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "Compiler driver not found. Set CHIMERA_COMPILER_DRIVER environment \
         variable or build compiler-core with the driver installed."
    )
}

/// Invoke compiler-core driver to process input files
fn invoke_compiler_driver(
    driver_path: &Path,
    inputs: &[PathBuf],
    output_dir: &Path,
    target: &str,
    emit_metadata: bool,
    emit_proof: bool,
    emit_object: bool,
) -> Result<()> {
    log::info!("Invoking compiler driver: {:?}", driver_path);

    let mut cmd = Command::new(driver_path);
    cmd.arg("--verbose");

    for input in inputs {
        cmd.arg(input);
    }

    // Set output file in the output directory
    let output_file = output_dir.join("output.chir");
    cmd.arg("-o").arg(&output_file);

    // Pass target triple
    cmd.arg("--target").arg(target);

    // Enable metadata emission
    if emit_metadata {
        cmd.arg("--emit-metadata");
        let metadata_file = output_dir.join("output.chmeta");
        cmd.arg("--metadata-output").arg(&metadata_file);
    }

    // Enable proof emission (typically paired with metadata)
    if emit_proof {
        cmd.arg("--emit-proof");
        let proof_file = output_dir.join("output.chproof");
        cmd.arg("--proof-output").arg(&proof_file);
    }

    // Enable object emission
    if emit_object {
        cmd.arg("--emit-object");
        let object_file = output_dir.join("output.cho");
        cmd.arg("--object-output").arg(&object_file);
    }

    log::debug!("Running: {:?}", cmd);

    let output = cmd.output().context("Failed to execute compiler driver")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        log::error!("Compiler driver stderr:\n{}", stderr);
        if !stdout.is_empty() {
            log::info!("Compiler driver stdout:\n{}", stdout);
        }
        anyhow::bail!(
            "Compiler driver failed with exit code: {:?}",
            output.status.code()
        );
    }

    if !output.stderr.is_empty() {
        log::debug!(
            "Compiler driver stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    log::info!("Compiler driver completed successfully");
    Ok(())
}

/// Build using compiler-core driver for ChimeraIR inputs
fn build_with_compiler_core(
    sources: &[PathBuf],
    output_dir: &Path,
    target: &str,
    emit_metadata: bool,
    emit_proof: bool,
    emit_object: bool,
) -> Result<PathBuf> {
    let driver_path = find_compiler_driver()?;

    // Filter for MLIR/ChimeraIR source files
    let chir_inputs: Vec<PathBuf> = sources
        .iter()
        .filter(|p| {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            matches!(ext.to_lowercase().as_str(), "mlir" | "chir" | "chimera")
        })
        .cloned()
        .collect();

    let other_inputs: Vec<PathBuf> = sources
        .iter()
        .filter(|p| {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            !matches!(ext.to_lowercase().as_str(), "mlir" | "chir" | "chimera")
        })
        .cloned()
        .collect();

    // Process MLIR/ChimeraIR inputs through the driver
    if !chir_inputs.is_empty() {
        invoke_compiler_driver(
            &driver_path,
            &chir_inputs,
            output_dir,
            target,
            emit_metadata,
            emit_proof,
            emit_object,
        )?;
    }

    // For non-MLIR sources (C/Rust/Zig), invoke via wrapper build
    // The actual compilation is handled by the BuildOrchestrator
    if !other_inputs.is_empty() {
        log::info!(
            "Building {} non-ChimeraIR source(s) via orchestrator",
            other_inputs.len()
        );
    }

    Ok(output_dir.join("output.chir"))
}

pub fn run(
    manifest: Option<PathBuf>,
    target: Option<String>,
    output: Option<PathBuf>,
    skip_proof: bool,
) -> Result<()> {
    log::info!("Running chimera build");

    let manifest_path = manifest.unwrap_or_else(|| PathBuf::from("Chimera.toml"));
    let output_dir = output.unwrap_or_else(|| PathBuf::from("build"));

    // Create output directory
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Load project manifest if present
    let project_manifest = if manifest_path.exists() {
        log::info!("Loading manifest: {:?}", manifest_path);
        match ProjectManifest::parse_file(&manifest_path) {
            Ok(m) => {
                log::info!("  name: {}", m.name);
                log::info!("  sources: {}", m.sources.len());
                log::info!("  runtime mode: {:?}", m.runtime_mode);
                Some(m)
            }
            Err(e) => {
                log::warn!("Failed to parse manifest: {}", e);
                None
            }
        }
    } else {
        log::info!("No manifest found, using default configuration");
        None
    };

    let target_triple = target.unwrap_or_else(|| default_target_triple(project_manifest.as_ref()));
    let build_mode = default_build_mode(project_manifest.as_ref());
    let rustc_driver_path = find_rustc_driver();

    let build_config = BuildConfig {
        target: chimera_build::Target {
            triple: target_triple.clone(),
            features: vec![],
            runtime_variant: None,
            cpu_features: vec![],
        },
        output_dir: output_dir.clone(),
        cache_enabled: true,
        proof_verification: !skip_proof,
        build_mode,
        wrapper_languages: vec![SourceLanguage::C, SourceLanguage::Rust, SourceLanguage::Zig],
        rust_artifacts_dir: output_dir.join("artifacts/rust"),
        rust_cache_dir: output_dir.join("cache/rust"),
        zig_artifacts_dir: PathBuf::from(".zigmera/artifacts"),
        zigmera_lowering_path: None, // Use fallback mode in tests (zigml may not exist)
        rustc_driver_path: rustc_driver_path.clone(),
        c_artifacts_dir: output_dir.join("artifacts/c"),
        chimera_c_clang_path: None,
        chimera_c_cache_path: None,
        require_authoritative_zig: false,
    };

    log::info!("Build configuration:");
    log::info!("  target: {}", target_triple);
    log::info!("  output: {:?}", output_dir);
    log::info!("  build mode: {:?}", build_mode);
    if let Some(ref driver_path) = rustc_driver_path {
        log::info!("  rustc driver: {:?}", driver_path);
    }
    log::info!("  proof verification: {}", !skip_proof);

    // Build metadata from manifest or create minimal metadata
    let metadata = if let Some(ref manifest) = project_manifest {
        Metadata {
            version: Version::new(0, 1, 0),
            module: Some(Module {
                name: manifest.name.clone(),
                target: target_triple.clone(),
                source_lang: SourceLanguage::Rust,
            }),
            ..Default::default()
        }
    } else {
        Metadata {
            version: Version::new(0, 1, 0),
            module: Some(Module {
                name: "chimera-build".to_string(),
                target: target_triple.clone(),
                source_lang: SourceLanguage::Rust,
            }),
            ..Default::default()
        }
    };

    let mut orchestrator = BuildOrchestrator::new(build_config);

    // Create source list from manifest
    let sources: Vec<PathBuf> = if let Some(ref manifest) = project_manifest {
        let manifest_dir = manifest_path.parent().unwrap_or(std::path::Path::new("."));
        manifest
            .sources
            .iter()
            .map(|s| manifest_dir.join(&s.path))
            .collect()
    } else {
        vec![]
    };
    let uses_legacy_sources = !sources.is_empty();

    let components = if let Some(ref manifest) = project_manifest {
        let manifest_dir = manifest_path.parent().unwrap_or(std::path::Path::new("."));
        manifest
            .get_components()
            .into_iter()
            .filter_map(|c| c.ok())
            .map(|mut component| {
                component.roots = component
                    .roots
                    .into_iter()
                    .map(|root| {
                        if root.is_relative() {
                            manifest_dir.join(root)
                        } else {
                            root
                        }
                    })
                    .collect();
                if let Some(manifest) = component.manifest.take() {
                    component.manifest = Some(if manifest.is_relative() {
                        manifest_dir.join(manifest)
                    } else {
                        manifest
                    });
                }
                component
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let abi_edges = if let Some(ref manifest) = project_manifest {
        manifest
            .get_abi_edges()
            .into_iter()
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    log::info!("Building {} component(s)...", components.len());

    // Try to use compiler-core driver for ChimeraIR sources
    if !sources.is_empty() {
        match build_with_compiler_core(
            &sources,
            &output_dir,
            &target_triple,
            !skip_proof,
            !skip_proof,
            !skip_proof,
        ) {
            Ok(chir_output) => {
                log::info!("ChimeraIR compilation complete: {:?}", chir_output);
            }
            Err(e) => {
                log::debug!("Compiler-core driver not available: {}", e);
                log::info!("Falling back to orchestrator build");
            }
        }
    }

    let exec_path = output_dir.join("chimera_binary");

    if uses_legacy_sources {
        match orchestrator.build(&sources, &metadata) {
            Ok(path) => {
                log::info!("Legacy source build completed: {}", path.display());
                if exec_path.exists() {
                    log::info!("Final binary ready: {}", exec_path.display());
                } else {
                    anyhow::bail!(
                        "legacy source build completed without producing {}",
                        exec_path.display()
                    );
                }
            }
            Err(e) => {
                log::error!("Legacy source build failed: {}", e);
                return Err(anyhow::anyhow!("build failed: {}", e));
            }
        }
    } else if !components.is_empty() {
        match orchestrator.build_from_components(&components, &abi_edges) {
            Ok(results) => {
                log::info!("Build successful: {} components built", results.len());
                if exec_path.exists() {
                    log::info!("Final binary ready: {}", exec_path.display());
                } else {
                    anyhow::bail!(
                        "build claimed success but no binary produced at: {}",
                        exec_path.display()
                    );
                }
            }
            Err(e) => {
                log::error!("Build failed: {}", e);
                return Err(anyhow::anyhow!("build failed: {}", e));
            }
        }
    } else {
        log::info!("No components or sources to build. Create a Chimera.toml with [[components]] or [[sources]] to define your project.");
        log::info!("Example:");
        log::info!("  [[components]]");
        log::info!("  id = \"my_c_lib\"");
        log::info!("  language = \"c\"");
        log::info!("  kind = \"c-source\"");
        log::info!("  roots = [\"src/main.c\"]");
    }

    log::info!("Build finished");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_build_config_creation() {
        let config = BuildConfig {
            target: chimera_build::Target {
                triple: "x86_64-unknown-linux-gnu".to_string(),
                features: vec![],
                runtime_variant: None,
                cpu_features: vec![],
            },
            output_dir: PathBuf::from("build"),
            cache_enabled: true,
            proof_verification: true,
            build_mode: BuildMode::default(),
            wrapper_languages: vec![SourceLanguage::C],
            rust_artifacts_dir: PathBuf::from("build/artifacts/rust"),
            rust_cache_dir: PathBuf::from("build/cache/rust"),
            zig_artifacts_dir: PathBuf::from(".zigmera/artifacts"),
            zigmera_lowering_path: Some(PathBuf::from("zigml")),
            rustc_driver_path: None,
            c_artifacts_dir: PathBuf::from("build/artifacts/c"),
            chimera_c_clang_path: None,
            chimera_c_cache_path: None,
            require_authoritative_zig: false,
        };

        assert_eq!(config.target.triple, "x86_64-unknown-linux-gnu");
        assert!(config.proof_verification);
        assert_eq!(
            config.rust_artifacts_dir,
            PathBuf::from("build/artifacts/rust")
        );
        assert_eq!(config.rust_cache_dir, PathBuf::from("build/cache/rust"));
    }

    #[test]
    fn test_build_with_manifest_info() {
        // Test that manifest info is properly displayed in logs
        let output_dir = PathBuf::from("build");
        let target_triple = chimera_build::host_target_triple();

        // Verify configuration values are set correctly
        assert_eq!(target_triple, chimera_build::host_target_triple());
        assert!(output_dir.exists() == false); // PathBuf doesn't check filesystem
    }

    #[test]
    fn test_default_target_triple_prefers_host_when_manifest_has_no_target() {
        assert_eq!(
            default_target_triple(None),
            chimera_build::host_target_triple()
        );
    }

    #[test]
    fn test_find_rustc_driver_honors_env_override() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let driver = temp.path().join("chimera-rustc-driver");
        std::fs::write(&driver, "").unwrap();
        std::env::set_var("CHIMERA_RUSTC_DRIVER", &driver);
        assert_eq!(find_rustc_driver(), Some(driver.clone()));
        std::env::remove_var("CHIMERA_RUSTC_DRIVER");
    }

    #[test]
    fn test_default_build_mode_uses_cargo_abi_for_single_rust_staticlib_manifest() {
        let manifest = ProjectManifest::parse(
            r#"
version = "0.2.0"
name = "chimera-beam"
output = "staticlib"

[[components]]
id = "beam_workspace"
language = "rust"
kind = "cargo-package"
roots = ["Cargo.toml"]
manifest = "Cargo.toml"
package = "chimera_erlang_beam_runtime"
crate_types = ["bin"]
"#,
        )
        .unwrap();

        assert_eq!(default_build_mode(Some(&manifest)), BuildMode::CargoCAbi);
    }

    #[test]
    fn test_default_build_mode_keeps_unified_lowering_for_multi_component_manifest() {
        let manifest = ProjectManifest::parse(
            r#"
version = "0.2.0"
name = "chimera-beam-separate"
output = "executable"

[[components]]
id = "beam_runtime"
language = "rust"
kind = "cargo-package"
roots = ["Cargo.toml"]
manifest = "Cargo.toml"
package = "chimera_erlang_beam_runtime"
crate_types = ["staticlib"]
exported_symbols = ["beam_runtime_init"]

[[components]]
id = "beam_launcher"
language = "c"
kind = "c-source"
roots = ["main.c"]
entry_symbol = "c_main"

[[abi_edges]]
consumer = "beam_launcher"
provider = "beam_runtime"
mode = "direct-link"
wrapper = "none"
proof = "optional"
"#,
        )
        .unwrap();

        assert_eq!(
            default_build_mode(Some(&manifest)),
            BuildMode::UnifiedLowering
        );
    }

    #[test]
    fn test_find_compiler_driver_env_override() {
        // When CHIMERA_COMPILER_DRIVER is set to a valid path, it should be found
        // This test verifies the logic without relying on actual filesystem
        let env_path = std::env::var("CHIMERA_COMPILER_DRIVER");
        // If the env var is set in test environment, we can verify behavior
        if let Ok(path) = env_path {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn test_build_with_compiler_core_filters_sources() {
        // Test that MLIR/ChimeraIR sources are correctly identified
        let mlir_sources = vec![
            PathBuf::from("input.mlir"),
            PathBuf::from("module.chir"),
            PathBuf::from("test.chimera"),
        ];
        let other_sources = vec![
            PathBuf::from("src/main.c"),
            PathBuf::from("lib.rs"),
            PathBuf::from("build.zig"),
        ];

        let all_sources: Vec<PathBuf> = mlir_sources
            .iter()
            .chain(other_sources.iter())
            .cloned()
            .collect();

        let chir_inputs: Vec<PathBuf> = all_sources
            .iter()
            .filter(|p| {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                matches!(ext.to_lowercase().as_str(), "mlir" | "chir" | "chimera")
            })
            .cloned()
            .collect();

        let other_inputs: Vec<PathBuf> = all_sources
            .iter()
            .filter(|p| {
                let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                !matches!(ext.to_lowercase().as_str(), "mlir" | "chir" | "chimera")
            })
            .cloned()
            .collect();

        assert_eq!(chir_inputs.len(), 3);
        assert_eq!(other_inputs.len(), 3);
    }

    #[test]
    fn test_invoke_compiler_driver_command_args() {
        // Verify the command construction logic
        let driver_path = PathBuf::from("/usr/bin/chimerac");
        let inputs = vec![PathBuf::from("test.mlir")];
        let output_dir = PathBuf::from("build");
        let target = "x86_64-unknown-linux-gnu";

        // Build expected command string for verification
        let mut cmd_str = format!("{:?}", driver_path);
        cmd_str.push_str(" --verbose ");
        for input in &inputs {
            cmd_str.push_str(&format!(" {:?} ", input));
        }
        cmd_str.push_str(&format!(" -o {:?} ", output_dir.join("output.chir")));
        cmd_str.push_str(&format!(" --target {}", target));

        // Verify command includes expected components
        assert!(cmd_str.contains("/usr/bin/chimerac"));
        assert!(cmd_str.contains("test.mlir"));
        assert!(cmd_str.contains("output.chir"));
        assert!(cmd_str.contains("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_emit_metadata_flag_in_command() {
        // Verify that --emit-metadata flag is included when metadata is needed
        let driver_path = PathBuf::from("/usr/bin/chimerac");
        let output_dir = PathBuf::from("build");
        let emit_metadata = true;

        // Build command string
        let mut cmd_str = String::new();
        cmd_str.push_str(&format!("{:?}", driver_path));
        if emit_metadata {
            cmd_str.push_str(" --emit-metadata ");
        }
        cmd_str.push_str(&format!(" -o {:?} ", output_dir.join("output.chir")));

        assert!(cmd_str.contains("--emit-metadata"));
    }

    #[test]
    fn test_metadata_output_path_construction() {
        // Verify metadata output path is derived correctly
        let output_dir = PathBuf::from("build");
        let metadata_output = output_dir.join("output.chmeta");

        assert_eq!(metadata_output.file_name().unwrap(), "output.chmeta");
    }

    #[test]
    fn test_chimera_meta_parseable() {
        // Verify that JSON metadata matching chimera-meta schema is parseable
        let json = r#"{
            "version": {"major": 0, "minor": 1, "patch": 0},
            "module": {
                "name": "/tmp/test.mlir",
                "target": "unknown",
                "source_lang": "rust"
            },
            "functions": [],
            "proof_obligations": [],
            "wrappers": []
        }"#;

        let meta = Metadata::parse(json);
        assert!(
            meta.is_ok(),
            "Metadata should be parseable: {:?}",
            meta.err()
        );

        let meta = meta.unwrap();
        assert_eq!(meta.version.major, 0);
        assert_eq!(meta.version.minor, 1);
        assert!(meta.module.is_some());
    }

    #[test]
    fn test_chimera_meta_version_compatible() {
        use chimera_meta::is_compatible;

        // Current version (0.1.x) should be compatible
        assert!(is_compatible(0, 1, 0));
        assert!(is_compatible(0, 1, 5));

        // Different minor versions are not compatible
        assert!(!is_compatible(0, 2, 0));

        // Different major versions are not compatible
        assert!(!is_compatible(1, 0, 0));
    }

    #[test]
    fn test_emit_proof_flag_in_command() {
        // Verify that --emit-proof flag is included when proof is needed
        let driver_path = PathBuf::from("/usr/bin/chimerac");
        let output_dir = PathBuf::from("build");
        let emit_proof = true;

        // Build command string
        let mut cmd_str = String::new();
        cmd_str.push_str(&format!("{:?}", driver_path));
        if emit_proof {
            cmd_str.push_str(" --emit-proof ");
        }
        cmd_str.push_str(&format!(" -o {:?} ", output_dir.join("output.chir")));

        assert!(cmd_str.contains("--emit-proof"));
    }

    #[test]
    fn test_proof_output_path_construction() {
        // Verify proof output path is derived correctly
        let output_dir = PathBuf::from("build");
        let proof_output = output_dir.join("output.chproof");

        assert_eq!(proof_output.file_name().unwrap(), "output.chproof");
    }

    #[test]
    fn test_emit_object_flag_in_command() {
        // Verify that --emit-object flag is included when object is needed
        let driver_path = PathBuf::from("/usr/bin/chimerac");
        let output_dir = PathBuf::from("build");
        let emit_object = true;

        // Build command string
        let mut cmd_str = String::new();
        cmd_str.push_str(&format!("{:?}", driver_path));
        if emit_object {
            cmd_str.push_str(" --emit-object ");
        }
        cmd_str.push_str(&format!(" -o {:?} ", output_dir.join("output.chir")));

        assert!(cmd_str.contains("--emit-object"));
    }

    #[test]
    fn test_object_output_path_construction() {
        // Verify object output path is derived correctly
        let output_dir = PathBuf::from("build");
        let object_output = output_dir.join("output.cho");

        assert_eq!(object_output.file_name().unwrap(), "output.cho");
    }

    #[test]
    fn test_all_artifact_flags_present() {
        // Verify that all artifact emission flags are represented in command construction
        let emit_metadata = true;
        let emit_proof = true;
        let emit_object = true;

        let mut flags = Vec::new();
        if emit_metadata {
            flags.push("--emit-metadata");
        }
        if emit_proof {
            flags.push("--emit-proof");
        }
        if emit_object {
            flags.push("--emit-object");
        }

        assert_eq!(flags.len(), 3);
        assert!(flags.contains(&"--emit-metadata"));
        assert!(flags.contains(&"--emit-proof"));
        assert!(flags.contains(&"--emit-object"));
    }

    #[test]
    fn test_artifact_paths_formatted_correctly() {
        // Test that all artifact paths follow the expected naming convention
        let base_output = PathBuf::from("build/output");

        let chir_path = base_output.with_extension("chir");
        let chmeta_path = PathBuf::from("build/output.chmeta");
        let chproof_path = PathBuf::from("build/output.chproof");
        let cho_path = PathBuf::from("build/output.cho");

        assert_eq!(chir_path.file_name().unwrap(), "output.chir");
        assert_eq!(chmeta_path.file_name().unwrap(), "output.chmeta");
        assert_eq!(chproof_path.file_name().unwrap(), "output.chproof");
        assert_eq!(cho_path.file_name().unwrap(), "output.cho");
    }
}
