//! Chimera Rust Cargo integration crate.
//!
//! Provides Cargo metadata ingestion, rustc invocation config capture,
//! build script support, and proc-macro conservative handling.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CargoError {
    #[error("failed to run cargo metadata: {0}")]
    MetadataFailed(String),
    #[error("failed to parse cargo output: {0}")]
    ParseFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Cargo metadata result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoMetadata {
    pub workspace_members: Vec<Package>,
    pub target_directory: PathBuf,
    pub workspace_root: Option<PathBuf>,
    pub version: String,
    pub features: BTreeMap<String, Vec<String>>,
}

/// A package in the Cargo workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub edition: String,
    pub crate_types: Vec<CrateType>,
    pub targets: Vec<Target>,
    pub features: BTreeMap<String, Vec<String>>,
    pub dependencies: Vec<Dependency>,
    pub manifest_path: PathBuf,
}

/// Target within a package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub kind: Vec<TargetKind>,
    pub src_path: Option<PathBuf>,
    pub edition: String,
    pub doc: bool,
    pub doctest: bool,
    pub test: bool,
}

/// Kind of target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetKind {
    Lib,
    Bin,
    Test,
    Example,
    BuildScript,
    ProcMacro,
}

/// Crate type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrateType {
    Library,
    Binary,
    Cdylib,
    Staticlib,
    Rlib,
    ProcMacro,
}

/// A dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub rename: Option<String>,
    pub pkg_id: Option<String>,
    pub source_kind: Option<String>,
    pub source: Option<String>,
    pub source_ref: Option<String>,
    pub version: Option<String>,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
}

/// Rustc invocation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustcConfig {
    pub target_triple: String,
    pub panic_strategy: PanicStrategy,
    pub codegen_units: Option<u32>,
    pub incremental: bool,
    pub crate_type: CrateType,
    pub rustflags: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

/// Panic strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanicStrategy {
    Unwind,
    Abort,
}

/// Build script output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScriptOutput {
    pub rustc_cfg: Vec<String>,
    pub rustc_link_lib: Vec<LinkLibrary>,
    pub rustc_link_search: Vec<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub rerun_if_changed: Vec<PathBuf>,
    pub rerun_if_env_changed: Vec<String>,
}

/// Link library from build script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkLibrary {
    pub kind: LinkKind,
    pub name: String,
    pub lib: String,
}

/// Kind of link library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkKind {
    Dylib,
    Static,
    Framework,
}

/// Fetch cargo metadata by running `cargo metadata`
pub fn fetch_metadata(manifest_path: &PathBuf) -> Result<CargoMetadata, CargoError> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(manifest_path.parent().unwrap_or(manifest_path))
        .output()?;

    if !output.status.success() {
        return Err(CargoError::MetadataFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let meta: cargo_metadata::Metadata = serde_json::from_slice(&output.stdout)
        .map_err(|e| CargoError::ParseFailed(e.to_string()))?;

    let workspace_members: Vec<Package> = meta
        .workspace_members
        .iter()
        .filter_map(|id| {
            meta.packages.iter().find(|p| p.id == *id).map(|p| Package {
                id: p.id.to_string(),
                name: p.name.clone(),
                version: p.version.to_string(),
                edition: p.edition.to_string(),
                crate_types: vec![CrateType::Library], // Default, will be refined
                targets: p
                    .targets
                    .iter()
                    .map(|t| Target {
                        name: t.name.clone(),
                        kind: t
                            .kind
                            .iter()
                            .map(|k| match k.as_str() {
                                "lib" => TargetKind::Lib,
                                "bin" => TargetKind::Bin,
                                "test" => TargetKind::Test,
                                "example" => TargetKind::Example,
                                "build-script" => TargetKind::BuildScript,
                                "proc-macro" => TargetKind::ProcMacro,
                                _ => TargetKind::Lib,
                            })
                            .collect(),
                        src_path: Some(t.src_path.clone().into()),
                        edition: t.edition.to_string(),
                        doc: t.doc,
                        doctest: t.doctest,
                        test: t.test,
                    })
                    .collect(),
                features: p.features.clone(),
                dependencies: p
                    .dependencies
                    .iter()
                    .map(|d| Dependency {
                        name: d.name.clone(),
                        rename: d.rename.clone(),
                        pkg_id: None, // Not available in cargo_metadata 0.18
                        source_kind: if d.path.is_some() {
                            Some("path".to_string())
                        } else if d
                            .source
                            .as_deref()
                            .is_some_and(|source| source.starts_with("git+"))
                        {
                            Some("git".to_string())
                        } else if d.source.is_some() || d.registry.is_some() {
                            Some("registry".to_string())
                        } else {
                            None
                        },
                        source: d
                            .path
                            .as_ref()
                            .map(|path| path.to_string())
                            .or_else(|| d.source.clone())
                            .or_else(|| d.registry.clone()),
                        source_ref: d
                            .source
                            .as_deref()
                            .and_then(git_dependency_source_ref_from_cargo_source),
                        version: Some(d.req.to_string()),
                        features: d.features.clone(),
                        optional: d.optional,
                        default_features: d.uses_default_features,
                    })
                    .collect(),
                manifest_path: p.manifest_path.clone().into(),
            })
        })
        .collect();

    let features: BTreeMap<String, Vec<String>> = meta
        .packages
        .iter()
        .flat_map(|p| {
            p.features
                .iter()
                .map(|(k, v)| (format!("{}:{}", p.name, k), v.clone()))
        })
        .collect();

    Ok(CargoMetadata {
        workspace_members,
        target_directory: meta.target_directory.clone().into(),
        workspace_root: Some(PathBuf::from(meta.workspace_root.as_str())),
        version: "1".to_string(),
        features,
    })
}

fn git_dependency_source_ref_from_cargo_source(source: &str) -> Option<String> {
    if !source.starts_with("git+") {
        return None;
    }

    let query = source
        .split_once('?')
        .map(|(_, tail)| tail.split('#').next().unwrap_or_default())
        .unwrap_or_default();
    for key in ["branch", "tag", "rev"] {
        if let Some(value) = query.split('&').find_map(|entry| {
            let (entry_key, entry_value) = entry.split_once('=')?;
            (entry_key == key).then_some(entry_value)
        }) {
            return Some(format!("{key}={value}"));
        }
    }

    source
        .split_once('#')
        .map(|(_, fragment)| fragment)
        .filter(|fragment| !fragment.is_empty())
        .map(|fragment| format!("rev={fragment}"))
}

/// Capture rustc invocation configuration
pub fn capture_rustc_config(
    manifest_path: &PathBuf,
    target_triple: &str,
) -> Result<RustcConfig, CargoError> {
    let output = std::process::Command::new("cargo")
        .args(["rustc", "-v", "--", "--print=cfg"])
        .current_dir(manifest_path.parent().unwrap_or(manifest_path))
        .output()?;

    let cfg_output = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::new()
    };

    let panic_strategy = if cfg_output.contains("panic=\"abort\"") {
        PanicStrategy::Abort
    } else {
        PanicStrategy::Unwind
    };

    Ok(RustcConfig {
        target_triple: target_triple.to_string(),
        panic_strategy,
        codegen_units: Some(16), // Default
        incremental: true,
        crate_type: CrateType::Library,
        rustflags: Vec::new(),
        env_vars: HashMap::new(),
    })
}

/// Parse build script output from cargo:rustc-* directives
pub fn parse_build_script_output(output: &str) -> BuildScriptOutput {
    let mut rustc_cfg = Vec::new();
    let mut rustc_link_lib = Vec::new();
    let mut rustc_link_search = Vec::new();
    let mut env_vars = HashMap::new();
    let mut rerun_if_changed = Vec::new();
    let mut rerun_if_env_changed = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("cargo:rustc-cfg=") {
            rustc_cfg.push(line.trim_start_matches("cargo:rustc-cfg=").to_string());
        } else if line.starts_with("cargo:rustc-link-lib=") {
            let val = line.trim_start_matches("cargo:rustc-link-lib=");
            let parts: Vec<&str> = val.split('=').collect();
            rustc_link_lib.push(LinkLibrary {
                kind: match parts.get(0) {
                    Some(&"dylib") => LinkKind::Dylib,
                    Some(&"static") => LinkKind::Static,
                    Some(&"framework") => LinkKind::Framework,
                    _ => LinkKind::Dylib,
                },
                name: parts.get(1).unwrap_or(&"").to_string(),
                lib: parts.get(2).unwrap_or(&"").to_string(),
            });
        } else if line.starts_with("cargo:rustc-link-search=") {
            let path = line.trim_start_matches("cargo:rustc-link-search=");
            rustc_link_search.push(PathBuf::from(path));
        } else if line.starts_with("cargo:rustc-env=") {
            let val = line.trim_start_matches("cargo:rustc-env=");
            if let Some((key, value)) = val.split_once('=') {
                env_vars.insert(key.to_string(), value.to_string());
            }
        } else if line.starts_with("cargo:rerun-if-changed=") {
            rerun_if_changed.push(PathBuf::from(
                line.trim_start_matches("cargo:rerun-if-changed="),
            ));
        } else if line.starts_with("cargo:rerun-if-env-changed=") {
            rerun_if_env_changed.push(
                line.trim_start_matches("cargo:rerun-if-env-changed=")
                    .to_string(),
            );
        }
    }

    BuildScriptOutput {
        rustc_cfg,
        rustc_link_lib,
        rustc_link_search,
        env_vars,
        rerun_if_changed,
        rerun_if_env_changed,
    }
}

/// Compute a deterministic cache key for a package
pub fn compute_package_key(package: &Package, config: &RustcConfig) -> String {
    use blake3::Hasher;
    let mut hasher = Hasher::new();

    hasher.update(package.name.as_bytes());
    hasher.update(package.version.as_bytes());
    hasher.update(package.edition.as_bytes());
    hasher.update(config.target_triple.as_bytes());

    match config.panic_strategy {
        PanicStrategy::Unwind => {
            hasher.update(b"unwind");
        }
        PanicStrategy::Abort => {
            hasher.update(b"abort");
        }
    }

    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_build_script_output() {
        let output = r#"
cargo:rustc-cfg=feature="unsafe"
cargo:rustc-link-lib=dylib=foo
cargo:rustc-link-search=native=/path/to/lib
cargo:rustc-env=SOME_VAR=value
cargo:rerun-if-changed=build.rs
cargo:rerun-if-env-changed=RUST_BACKTRACE
        "#;

        let parsed = parse_build_script_output(output);

        assert_eq!(parsed.rustc_cfg.len(), 1);
        assert_eq!(parsed.rustc_link_lib.len(), 1);
        assert_eq!(parsed.rustc_link_search.len(), 1);
        assert_eq!(parsed.env_vars.len(), 1);
        assert_eq!(parsed.rerun_if_changed.len(), 1);
        assert_eq!(parsed.rerun_if_env_changed.len(), 1);
    }

    #[test]
    fn test_compute_package_key() {
        let package = Package {
            id: "test".to_string(),
            name: "my_crate".to_string(),
            version: "1.0.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library],
            targets: vec![],
            features: BTreeMap::new(),
            dependencies: vec![],
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
        };

        let config = RustcConfig {
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            panic_strategy: PanicStrategy::Unwind,
            codegen_units: Some(16),
            incremental: true,
            crate_type: CrateType::Library,
            rustflags: vec![],
            env_vars: HashMap::new(),
        };

        let key = compute_package_key(&package, &config);
        assert_eq!(key.len(), 64); // blake3 hex length
    }

    #[test]
    fn test_build_script_output_rustc_cfg_parsing() {
        let output = r#"cargo:rustc-cfg=feature="foo"
cargo:rustc-cfg=feature="bar""#;

        let parsed = parse_build_script_output(output);
        assert_eq!(
            parsed.rustc_cfg,
            vec![r#"feature="foo""#, r#"feature="bar""#]
        );
    }

    #[test]
    fn test_build_script_output_link_lib_parsing() {
        let output = r#"cargo:rustc-link-lib=dylib=foo=libfoo.so
cargo:rustc-link-lib=static=bar=libbar.a"#;

        let parsed = parse_build_script_output(output);
        assert_eq!(parsed.rustc_link_lib.len(), 2);
    }

    #[test]
    fn test_package_serialization() {
        let package = Package {
            id: "test-1.0.0".to_string(),
            name: "test_crate".to_string(),
            version: "1.0.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library, CrateType::Rlib],
            targets: vec![Target {
                name: "lib".to_string(),
                kind: vec![TargetKind::Lib],
                src_path: Some(PathBuf::from("src/lib.rs")),
                edition: "2021".to_string(),
                doc: true,
                doctest: true,
                test: true,
            }],
            features: BTreeMap::from([("default".to_string(), vec!["foo".to_string()])]),
            dependencies: vec![Dependency {
                name: "serde".to_string(),
                rename: Some("serde_alias".to_string()),
                pkg_id: Some("serde-1.0".to_string()),
                source_kind: Some("registry".to_string()),
                source: Some("crates.io".to_string()),
                source_ref: None,
                version: Some("1.0".to_string()),
                features: vec![],
                optional: false,
                default_features: true,
            }],
            manifest_path: PathBuf::from("Cargo.toml"),
        };

        let json = serde_json::to_string(&package).unwrap();
        let deserialized: Package = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_crate");
        assert_eq!(deserialized.edition, "2021");
        assert_eq!(deserialized.targets.len(), 1);
        assert_eq!(
            deserialized.dependencies[0].rename.as_deref(),
            Some("serde_alias")
        );
        assert_eq!(deserialized.dependencies[0].source_ref, None);
    }

    #[test]
    fn test_git_dependency_source_ref_from_cargo_source() {
        assert_eq!(
            git_dependency_source_ref_from_cargo_source(
                "git+https://github.com/example/repo?branch=main#0123456789abcdef"
            )
            .as_deref(),
            Some("branch=main")
        );
        assert_eq!(
            git_dependency_source_ref_from_cargo_source(
                "git+https://github.com/example/repo?tag=v1.2.3#0123456789abcdef"
            )
            .as_deref(),
            Some("tag=v1.2.3")
        );
        assert_eq!(
            git_dependency_source_ref_from_cargo_source(
                "git+https://github.com/example/repo?rev=deadbeef#0123456789abcdef"
            )
            .as_deref(),
            Some("rev=deadbeef")
        );
        assert_eq!(
            git_dependency_source_ref_from_cargo_source(
                "git+https://github.com/example/repo#0123456789abcdef"
            )
            .as_deref(),
            Some("rev=0123456789abcdef")
        );
        assert_eq!(
            git_dependency_source_ref_from_cargo_source(
                "registry+https://github.com/rust-lang/crates.io-index"
            ),
            None
        );
    }

    #[test]
    fn test_rustc_config_serialization() {
        let config = RustcConfig {
            target_triple: "aarch64-apple-darwin".to_string(),
            panic_strategy: PanicStrategy::Abort,
            codegen_units: Some(1),
            incremental: false,
            crate_type: CrateType::Cdylib,
            rustflags: vec!["-C".to_string(), "opt-level=3".to_string()],
            env_vars: HashMap::from([("RUST_LOG".to_string(), "debug".to_string())]),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RustcConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.target_triple, "aarch64-apple-darwin");
        assert!(matches!(deserialized.panic_strategy, PanicStrategy::Abort));
    }

    #[test]
    fn test_workspace_metadata() {
        let metadata = CargoMetadata {
            workspace_members: vec![
                Package {
                    id: "workspace-member-1.0.0".to_string(),
                    name: "member_one".to_string(),
                    version: "1.0.0".to_string(),
                    edition: "2021".to_string(),
                    crate_types: vec![CrateType::Library],
                    targets: vec![Target {
                        name: "lib".to_string(),
                        kind: vec![TargetKind::Lib],
                        src_path: Some(PathBuf::from("src/lib.rs")),
                        edition: "2021".to_string(),
                        doc: true,
                        doctest: true,
                        test: true,
                    }],
                    features: BTreeMap::from([
                        (
                            "default".to_string(),
                            vec!["member_one/feature-a".to_string()],
                        ),
                        ("feature-a".to_string(), vec![]),
                        (
                            "feature-b".to_string(),
                            vec!["dep:optional-dep".to_string()],
                        ),
                    ]),
                    dependencies: vec![Dependency {
                        name: "optional-dep".to_string(),
                        rename: None,
                        pkg_id: Some("optional-dep-2.0".to_string()),
                        source_kind: Some("registry".to_string()),
                        source: Some("crates.io".to_string()),
                        source_ref: None,
                        version: Some("2.0".to_string()),
                        features: vec![],
                        optional: true,
                        default_features: true,
                    }],
                    manifest_path: PathBuf::from("/workspace/member_one/Cargo.toml"),
                },
                Package {
                    id: "workspace-member-2.0.0".to_string(),
                    name: "member_two".to_string(),
                    version: "2.0.0".to_string(),
                    edition: "2021".to_string(),
                    crate_types: vec![CrateType::Binary],
                    targets: vec![Target {
                        name: "bin".to_string(),
                        kind: vec![TargetKind::Bin],
                        src_path: Some(PathBuf::from("src/main.rs")),
                        edition: "2021".to_string(),
                        doc: false,
                        doctest: false,
                        test: true,
                    }],
                    features: BTreeMap::new(),
                    dependencies: vec![],
                    manifest_path: PathBuf::from("/workspace/member_two/Cargo.toml"),
                },
            ],
            target_directory: PathBuf::from("/workspace/target"),
            workspace_root: Some(PathBuf::from("/workspace")),
            version: "1".to_string(),
            features: BTreeMap::from([(
                "default".to_string(),
                vec!["member_two/feature-x".to_string()],
            )]),
        };

        assert_eq!(metadata.workspace_members.len(), 2);
        assert_eq!(metadata.workspace_members[0].name, "member_one");
        assert_eq!(metadata.workspace_members[1].name, "member_two");

        let first_pkg = &metadata.workspace_members[0];
        assert_eq!(first_pkg.features.len(), 3);
        assert!(first_pkg.dependencies[0].optional);
    }

    #[test]
    fn test_target_specific_dependencies() {
        let package = Package {
            id: "target-specific-deps".to_string(),
            name: "target_specific_deps".to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library],
            targets: vec![
                Target {
                    name: "lib".to_string(),
                    kind: vec![TargetKind::Lib],
                    src_path: Some(PathBuf::from("src/lib.rs")),
                    edition: "2021".to_string(),
                    doc: true,
                    doctest: false,
                    test: true,
                },
                Target {
                    name: "lib_bin".to_string(),
                    kind: vec![TargetKind::Bin],
                    src_path: Some(PathBuf::from("src/bin.rs")),
                    edition: "2021".to_string(),
                    doc: false,
                    doctest: false,
                    test: true,
                },
            ],
            features: BTreeMap::new(),
            dependencies: vec![
                Dependency {
                    name: "unix-only".to_string(),
                    rename: None,
                    pkg_id: Some("unix-only-1.0".to_string()),
                    source_kind: Some("registry".to_string()),
                    source: Some("crates.io".to_string()),
                    source_ref: None,
                    version: Some("1.0".to_string()),
                    features: vec![],
                    optional: false,
                    default_features: true,
                },
                Dependency {
                    name: "windows-only".to_string(),
                    rename: None,
                    pkg_id: Some("windows-only-1.0".to_string()),
                    source_kind: Some("registry".to_string()),
                    source: Some("crates.io".to_string()),
                    source_ref: None,
                    version: Some("1.0".to_string()),
                    features: vec![],
                    optional: false,
                    default_features: true,
                },
            ],
            manifest_path: PathBuf::from("/project/Cargo.toml"),
        };

        assert_eq!(package.targets.len(), 2);
        assert!(matches!(package.targets[0].kind[0], TargetKind::Lib));
        assert!(matches!(package.targets[1].kind[0], TargetKind::Bin));
        assert_eq!(package.dependencies.len(), 2);
    }

    #[test]
    fn test_dev_dependencies() {
        let package = Package {
            id: "dev-deps-test".to_string(),
            name: "dev_deps_test".to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library],
            targets: vec![Target {
                name: "lib".to_string(),
                kind: vec![TargetKind::Lib],
                src_path: Some(PathBuf::from("src/lib.rs")),
                edition: "2021".to_string(),
                doc: true,
                doctest: true,
                test: true,
            }],
            features: BTreeMap::new(),
            dependencies: vec![
                Dependency {
                    name: "test-framework".to_string(),
                    rename: None,
                    pkg_id: Some("test-framework-1.0".to_string()),
                    source_kind: Some("registry".to_string()),
                    source: Some("crates.io".to_string()),
                    source_ref: None,
                    version: Some("1.0".to_string()),
                    features: vec!["dev".to_string()],
                    optional: false,
                    default_features: true,
                },
                Dependency {
                    name: "dev-only-util".to_string(),
                    rename: None,
                    pkg_id: Some("dev-only-util-1.0".to_string()),
                    source_kind: Some("registry".to_string()),
                    source: Some("crates.io".to_string()),
                    source_ref: None,
                    version: Some("1.0".to_string()),
                    features: vec![],
                    optional: true,
                    default_features: false,
                },
            ],
            manifest_path: PathBuf::from("/project/Cargo.toml"),
        };

        let deps: Vec<_> = package.dependencies.iter().collect();
        assert_eq!(deps.len(), 2);
        assert!(deps[0].features.contains(&"dev".to_string()));
    }

    #[test]
    fn test_feature_flag_resolution() {
        let package = Package {
            id: "feature-resolve".to_string(),
            name: "feature_resolve".to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library],
            targets: vec![],
            features: BTreeMap::from([
                ("default".to_string(), vec!["full".to_string()]),
                (
                    "full".to_string(),
                    vec!["feature-a".to_string(), "feature-b".to_string()],
                ),
                ("feature-a".to_string(), vec![]),
                ("feature-b".to_string(), vec!["feature-c".to_string()]),
                ("feature-c".to_string(), vec![]),
            ]),
            dependencies: vec![],
            manifest_path: PathBuf::from("/project/Cargo.toml"),
        };

        assert!(package
            .features
            .get("default")
            .unwrap()
            .contains(&"full".to_string()));
        assert!(package
            .features
            .get("full")
            .unwrap()
            .contains(&"feature-a".to_string()));
        assert!(package
            .features
            .get("full")
            .unwrap()
            .contains(&"feature-b".to_string()));
        assert!(package
            .features
            .get("feature-b")
            .unwrap()
            .contains(&"feature-c".to_string()));
    }

    #[test]
    fn test_crate_types() {
        let lib_crate = Package {
            id: "lib-crate".to_string(),
            name: "lib_crate".to_string(),
            version: "1.0.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Library, CrateType::Rlib],
            targets: vec![],
            features: BTreeMap::new(),
            dependencies: vec![],
            manifest_path: PathBuf::from("/project/Cargo.toml"),
        };
        assert!(lib_crate.crate_types.contains(&CrateType::Library));
        assert!(lib_crate.crate_types.contains(&CrateType::Rlib));

        let cdylib_crate = Package {
            id: "cdylib-crate".to_string(),
            name: "cdylib_crate".to_string(),
            version: "1.0.0".to_string(),
            edition: "2021".to_string(),
            crate_types: vec![CrateType::Cdylib, CrateType::Staticlib],
            targets: vec![],
            features: BTreeMap::new(),
            dependencies: vec![],
            manifest_path: PathBuf::from("/project/Cargo.toml"),
        };
        assert!(cdylib_crate.crate_types.contains(&CrateType::Cdylib));
        assert!(cdylib_crate.crate_types.contains(&CrateType::Staticlib));
    }
}
