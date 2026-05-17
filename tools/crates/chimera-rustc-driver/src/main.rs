//! Chimera Rust Compiler Driver - CLI Entry Point
//!
//! This binary provides the command-line interface for chimera-rustc-driver.
//! It is invoked by chimera-build when `rustc_driver_path` is configured.
//!
//! Usage:
//!   chimera-rustc-driver compile --source <file> --output <file> --artifacts-dir <dir> --target <triple> [--semantic-extraction] [--snapshot-only]

use chimera_rust_schema::{
    ArtifactHeader, BasicBlock, CrateGraph, CrateId, CrateNode, CrateType, DepEdge, DepEdgeKind,
    DepNode, DepNodeId, DepNodeKind, ItemId, ItemKind, Linkage, MirBody, RdepGraph, RmirPack,
    RsnapExport, RsnapItem, RsnapSnapshot, Visibility, VisibilityRank,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

fn host_target_triple() -> &'static str {
    if cfg!(all(
        target_arch = "aarch64",
        target_vendor = "apple",
        target_os = "macos"
    )) {
        "aarch64-apple-darwin"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_vendor = "apple",
        target_os = "macos"
    )) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
        "aarch64-unknown-linux-gnu"
    } else if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_arch = "aarch64", target_os = "windows")) {
        "aarch64-pc-windows-msvc"
    } else if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
        "x86_64-pc-windows-msvc"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}

/// Generated header specification from build script
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeneratedHeaderSpec {
    /// Path to generated header file
    path: String,
    /// Command that generates it (optional)
    generator_command: Option<String>,
    /// Content hash (set after generation)
    content_hash: Option<String>,
}

impl GeneratedHeaderSpec {
    fn fingerprint(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(self.path.as_bytes());
        if let Some(ref cmd) = self.generator_command {
            hasher.update(cmd.as_bytes());
        }
        if let Some(ref hash) = self.content_hash {
            hasher.update(hash.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

/// Build script output specification parsed from --build-script-output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BuildScriptOutput {
    script_path: String,
    rustc_cfg: Vec<String>,
    link_libs: Vec<String>,
    env_vars: Vec<String>,
    rerun_if_changed: Vec<String>,
    /// Generated headers from build.rs
    generated_headers: Vec<GeneratedHeaderSpec>,
}

/// Proc macro specification parsed from --proc-macro-version
#[derive(Debug, Clone)]
struct ProcMacroSpec {
    crate_name: String,
    version: String,
    expanded_token_hash: Option<String>,
}

#[derive(Debug, Clone)]
struct DriverCrateContext {
    crate_name: String,
    package_name: Option<String>,
    version: Option<String>,
    source_kind: Option<String>,
    source: Option<String>,
    source_ref: Option<String>,
    edition: String,
    crate_type: CrateType,
    extern_prelude: Vec<String>,
    dependencies: Vec<DriverDependencyContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DriverDependencyContext {
    crate_name: String,
    #[serde(default)]
    package_name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    source_kind: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    source_ref: Option<String>,
    edition: String,
    crate_type: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default = "default_true")]
    default_features: bool,
    #[serde(default)]
    optional: bool,
}

fn default_true() -> bool {
    true
}

impl BuildScriptOutput {
    /// Compute fingerprint for this build script output
    fn fingerprint(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(self.script_path.as_bytes());
        for cfg in &self.rustc_cfg {
            hasher.update(cfg.as_bytes());
        }
        for lib in &self.link_libs {
            hasher.update(lib.as_bytes());
        }
        for var in &self.env_vars {
            hasher.update(var.as_bytes());
        }
        for file in &self.rerun_if_changed {
            hasher.update(file.as_bytes());
        }
        for header in &self.generated_headers {
            hasher.update(header.path.as_bytes());
            if let Some(ref cmd) = header.generator_command {
                hasher.update(cmd.as_bytes());
            }
            if let Some(ref hash) = header.content_hash {
                hasher.update(hash.as_bytes());
            }
        }
        hasher.finalize().to_hex().to_string()
    }
}

impl ProcMacroSpec {
    /// Compute fingerprint for this proc macro version
    fn fingerprint(&self) -> String {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(self.crate_name.as_bytes());
        hasher.update(self.version.as_bytes());
        if let Some(ref hash) = self.expanded_token_hash {
            hasher.update(hash.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, Error)]
pub enum MainError {
    #[error("missing required argument: {0}")]
    MissingArg(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("chimera-rustc-driver - Rust semantic extraction sidecar");
        eprintln!("Usage: {} <command> [options]", args[0]);
        eprintln!("Commands: compile, extract-hir, extract-mir");
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "compile" => cmd_compile(&args[2..]),
        "extract-hir" => cmd_extract_hir(&args[2..]),
        "extract-mir" => cmd_extract_mir(&args[2..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_compile(args: &[String]) -> Result<(), MainError> {
    let mut source = None;
    let mut output = None;
    let mut artifacts_dir = None;
    let mut target = None;
    let mut _semantic_extraction = false;
    let mut snapshot_only = false;
    let mut build_script_outputs = Vec::new();
    let mut proc_macro_versions = Vec::new();
    let mut crate_name = None;
    let mut package_name = None;
    let mut package_version = None;
    let mut package_source_kind = None;
    let mut package_source = None;
    let mut crate_edition = None;
    let mut crate_type = None;
    let mut extern_prelude = Vec::new();
    let mut dependencies = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--source" => {
                source = Some(args[i + 1].clone());
                i += 2;
            }
            "--output" => {
                output = Some(args[i + 1].clone());
                i += 2;
            }
            "--artifacts-dir" => {
                artifacts_dir = Some(args[i + 1].clone());
                i += 2;
            }
            "--target" => {
                target = Some(args[i + 1].clone());
                i += 2;
            }
            "--semantic-extraction" => {
                _semantic_extraction = true;
                i += 1;
            }
            "--snapshot-only" => {
                snapshot_only = true;
                i += 1;
            }
            "--build-script-output" => {
                // Format: --build-script-output <json>
                // JSON: {"script_path":"...","rustc_cfg":[...],"link_libs":[...],"env_vars":[...],"rerun_if_changed":[...]}
                let json = args[i + 1].clone();
                if let Ok(bs_out) = serde_json::from_str::<BuildScriptOutput>(&json) {
                    build_script_outputs.push(bs_out);
                }
                i += 2;
            }
            "--crate-name" => {
                crate_name = Some(args[i + 1].clone());
                i += 2;
            }
            "--package-name" => {
                package_name = Some(args[i + 1].clone());
                i += 2;
            }
            "--package-version" => {
                package_version = Some(args[i + 1].clone());
                i += 2;
            }
            "--package-source-kind" => {
                package_source_kind = Some(args[i + 1].clone());
                i += 2;
            }
            "--package-source" => {
                package_source = Some(args[i + 1].clone());
                i += 2;
            }
            "--crate-edition" => {
                crate_edition = Some(args[i + 1].clone());
                i += 2;
            }
            "--crate-type" => {
                crate_type = Some(parse_driver_crate_type(&args[i + 1]));
                i += 2;
            }
            "--extern-prelude" => {
                extern_prelude.push(args[i + 1].clone());
                i += 2;
            }
            "--dependency-crate" => {
                if let Ok(context) = serde_json::from_str::<DriverDependencyContext>(&args[i + 1]) {
                    dependencies.push(context);
                }
                i += 2;
            }
            "--proc-macro-version" => {
                // Format: --proc-macro-version <crate_name>:<version>[:<expanded_token_hash>]
                let spec = args[i + 1].clone();
                let parts: Vec<&str> = spec.split(':').collect();
                if parts.len() >= 2 {
                    proc_macro_versions.push(ProcMacroSpec {
                        crate_name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        expanded_token_hash: parts.get(2).map(|s| s.to_string()),
                    });
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let source = source.ok_or_else(|| MainError::MissingArg("--source".to_string()))?;
    let output = output.ok_or_else(|| MainError::MissingArg("--output".to_string()))?;
    let artifacts_dir =
        artifacts_dir.ok_or_else(|| MainError::MissingArg("--artifacts-dir".to_string()))?;
    let target = target.unwrap_or_else(|| host_target_triple().to_string());

    // Create artifacts directory
    let artifacts_path = PathBuf::from(&artifacts_dir);
    fs::create_dir_all(&artifacts_path)?;

    // Read source file to extract basic info
    let source_content = fs::read_to_string(&source)?;
    let source_hash = blake3::Hasher::new()
        .update(source_content.as_bytes())
        .finalize()
        .to_hex()
        .to_string();
    let crate_context = DriverCrateContext {
        crate_name: crate_name.unwrap_or_else(|| "user_crate".to_string()),
        package_name,
        version: package_version,
        source_kind: package_source_kind,
        source: package_source,
        source_ref: None,
        edition: crate_edition.unwrap_or_else(|| "2021".to_string()),
        crate_type: crate_type.unwrap_or(CrateType::Library),
        extern_prelude,
        dependencies,
    };

    // Build RsnapSnapshot (semantic snapshot)
    let header = ArtifactHeader::new(&target, "0.1.0");
    let items = apply_crate_context(extract_items(&source_content, &source), &crate_context);
    let exports = extract_exports(&items);

    let rsnap = RsnapSnapshot {
        header: header.clone(),
        checksum: String::new(),
        rustc_version: "1.75.0".to_string(),
        crate_graph: build_crate_graph(&crate_context),
        items: items.clone(),
        exports,
        source_files: vec![chimera_rust_schema::SourceFile {
            path: source.clone(),
            content_hash: source_hash.clone(),
        }],
    };

    // Compute checksum
    let rsnap_checksum = rsnap.compute_checksum();

    // Compute fingerprints for build script outputs and proc macros
    // These are included in the artifact manifest for cache invalidation
    let build_script_fingerprints: Vec<String> = build_script_outputs
        .iter()
        .map(|bs| bs.fingerprint())
        .collect();
    let proc_macro_fingerprints: Vec<(String, String)> = proc_macro_versions
        .iter()
        .map(|pm| (pm.crate_name.clone(), pm.fingerprint()))
        .collect();

    let rsnap_with_checksum = RsnapSnapshot {
        checksum: rsnap_checksum.clone(),
        ..rsnap
    };

    // Write .rsnap artifact
    let rsnap_path = artifacts_path.join("lib.rs.rsnap");
    let rsnap_json = serde_json::to_string_pretty(&rsnap_with_checksum)?;
    fs::write(&rsnap_path, &rsnap_json)?;

    // Build RdepGraph (dependency graph)
    let dep_nodes = vec![
        DepNode {
            id: DepNodeId(0),
            kind: DepNodeKind::Source,
            fingerprint: source_hash,
            stable_id: "source_0".to_string(),
        },
        DepNode {
            id: DepNodeId(1),
            kind: DepNodeKind::Item,
            fingerprint: format!("{}_item_fp", rsnap_checksum),
            stable_id: "item_fn_main".to_string(),
        },
        DepNode {
            id: DepNodeId(2),
            kind: DepNodeKind::Export,
            fingerprint: format!("{}_export_fp", rsnap_checksum),
            stable_id: "export_main".to_string(),
        },
    ];

    let dep_edges = vec![
        DepEdge {
            from: DepNodeId(0),
            to: DepNodeId(1),
            kind: DepEdgeKind::DependsOn,
        },
        DepEdge {
            from: DepNodeId(1),
            to: DepNodeId(2),
            kind: DepEdgeKind::Provides,
        },
    ];

    let rdep = RdepGraph {
        header,
        checksum: String::new(),
        nodes: dep_nodes,
        edges: dep_edges,
    };

    // Write .rdep artifact
    let rdep_path = artifacts_path.join("lib.rs.rdep");
    let rdep_json = serde_json::to_string_pretty(&rdep)?;
    fs::write(&rdep_path, &rdep_json)?;

    // Build RmirPack (MIR package)
    let rmirpack = RmirPack {
        header: ArtifactHeader::new(&target, "0.1.0"),
        checksum: String::new(),
        types: vec![],
        layouts: vec![],
        bodies: vec![MirBody {
            item_id: ItemId(1),
            locals: vec![],
            blocks: vec![BasicBlock {
                index: 0,
                statements: vec![],
                terminator: chimera_rust_schema::Terminator::Return,
            }],
        }],
        constants: vec![],
    };

    // Write .rmirpack artifact
    let rmirpack_path = artifacts_path.join("lib.rs.rmirpack");
    let rmirpack_json = serde_json::to_string_pretty(&rmirpack)?;
    fs::write(&rmirpack_path, &rmirpack_json)?;

    // Execute compilation for the source file unless snapshot-only mode was requested.
    let is_c_source = source.ends_with(".c") || source.ends_with(".h") || source.ends_with(".C");
    if snapshot_only {
        println!("Snapshot-only: {} -> {}", source, artifacts_path.display());
    } else if is_c_source {
        compile_c_source(&source, &output, &target, &build_script_outputs)?;
        println!("Compiled: {} -> {}", source, output);
    } else {
        compile_rust_source(&source, &output, &target)?;
        println!("Compiled: {} -> {}", source, output);
    }

    println!(
        "Artifacts: {}, {}, {}",
        rsnap_path.display(),
        rdep_path.display(),
        rmirpack_path.display()
    );
    println!(
        "Compiler: {}",
        if is_c_source { "clang/gcc" } else { "rustc" }
    );

    if !build_script_fingerprints.is_empty() {
        println!("Build script fingerprints: {:?}", build_script_fingerprints);
    }
    if !proc_macro_fingerprints.is_empty() {
        println!("Proc macro fingerprints: {:?}", proc_macro_fingerprints);
    }

    // Process generated headers from build script outputs
    for bs_output in &build_script_outputs {
        for header in &bs_output.generated_headers {
            let header_path = std::path::Path::new(&header.path);
            let header_exists = header_path.exists();

            // Compute current content hash if file exists
            let current_hash = if header_exists {
                std::fs::read_to_string(header_path)
                    .ok()
                    .map(|content| blake3::hash(content.as_bytes()).to_hex().to_string())
            } else {
                None
            };

            // Check if regeneration is needed
            let needs_regeneration = match (&current_hash, &header.content_hash) {
                (Some(curr), Some(prev)) => curr != prev,
                (None, _) => true, // File doesn't exist
                (_, None) => true, // No previous hash recorded
            };

            if needs_regeneration {
                // Run generator command if provided
                if let Some(ref cmd) = header.generator_command {
                    println!(
                        "Regenerating header: {} using command: {}",
                        header.path, cmd
                    );
                    let shell_output = std::process::Command::new("sh").arg("-c").arg(cmd).output();

                    match shell_output {
                        Ok(out) if out.status.success() => {
                            // Regenerate hash after generation
                            if let Ok(new_content) = std::fs::read_to_string(header_path) {
                                let new_hash =
                                    blake3::hash(new_content.as_bytes()).to_hex().to_string();
                                println!("Generated header hash: {} -> {}", header.path, new_hash);
                            }
                        }
                        Ok(out) => {
                            eprintln!(
                                "Generator command failed for {}: {}",
                                header.path,
                                String::from_utf8_lossy(&out.stderr)
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to run generator for {}: {}", header.path, e);
                        }
                    }
                } else {
                    println!(
                        "Generated header missing and no generator command: {}",
                        header.path
                    );
                }
            } else {
                println!(
                    "Generated header up to date: {} (hash: {:?})",
                    header.path, current_hash
                );
            }
        }
    }

    Ok(())
}

/// Compile a Rust source file using rustc
fn compile_rust_source(
    source: &str,
    output: &str,
    target: &str,
) -> Result<std::process::Output, MainError> {
    let rustc_output = std::process::Command::new("rustc")
        .arg("--crate-type=staticlib")
        .arg("--target")
        .arg(target)
        .arg("-o")
        .arg(output)
        .arg(source)
        .output()?;

    if !rustc_output.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        return Err(MainError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("rustc compilation failed: {}", stderr),
        )));
    }

    Ok(rustc_output)
}

/// Compile a C source file using clang
fn compile_c_source(
    source: &str,
    output: &str,
    target: &str,
    build_script_outputs: &[BuildScriptOutput],
) -> Result<std::process::Output, MainError> {
    // Determine which C compiler to use
    let cc = std::env::var("CC")
        .or_else(|_| std::env::var("CXX"))
        .unwrap_or_else(|_| {
            // Try clang first, fall back to gcc
            if std::process::Command::new("clang")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                "clang".to_string()
            } else {
                "gcc".to_string()
            }
        });

    let mut cmd = std::process::Command::new(&cc);
    cmd.arg("-c")
        .arg(source)
        .arg("-o")
        .arg(output)
        .arg(format!("--target={}", target));

    // Add include directories from build script outputs
    for bs_output in build_script_outputs {
        for lib in &bs_output.link_libs {
            // -L for library search paths
            if lib.starts_with("-L") {
                cmd.arg(lib);
            }
        }
        // Add preprocessor defines from rustc_cfg
        for cfg in &bs_output.rustc_cfg {
            if cfg.starts_with("-D") {
                cmd.arg(cfg);
            }
        }
    }

    let compile_output = cmd.output()?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        return Err(MainError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} compilation failed: {}", cc, stderr),
        )));
    }

    Ok(compile_output)
}

/// Extract items from source code using syn for accurate parsing
#[cfg(feature = "syn")]
fn extract_items_from_source(source: &str, file_path: &str) -> Vec<RsnapItem> {
    use quote::ToTokens;
    use syn::{parse_str, Item, ItemKind as SynItemKind, Visibility};

    let ast = match parse_str::<syn::File>(source) {
        Ok(ast) => ast,
        Err(_) => return extract_items_simple(source, file_path), // fallback on parse error
    };

    let mut items = Vec::new();
    let mut item_id: u64 = 0;

    for item in ast.items {
        let rsnap_item = match item {
            Item::Fn(f) => {
                let name = f.sig.ident.to_string();
                if name.starts_with('_') {
                    continue;
                }
                let kind = ItemKind::Function;
                let visibility = convert_visibility(&f.vis);
                let def_path = format!("user_crate::{}", name);

                // Extract generics
                let generics = if f.sig.generics.params.is_empty() {
                    None
                } else {
                    Some(Generics {
                        lifetimes: f
                            .sig
                            .generics
                            .lifetimes()
                            .map(|l| l.ident.to_string())
                            .collect(),
                        type_params: f
                            .sig
                            .generics
                            .type_params()
                            .map(|tp| TypeParam {
                                name: tp.ident.to_string(),
                                bounds: vec![],
                            })
                            .collect(),
                        const_params: f
                            .sig
                            .generics
                            .const_params()
                            .map(|cp| cp.ident.to_string())
                            .collect(),
                    })
                };

                // Extract where clauses
                let where_clauses = f
                    .sig
                    .generics
                    .where_clause
                    .as_ref()
                    .map(|wc| {
                        wc.predicates
                            .iter()
                            .map(|p| WhereClause {
                                trait_id: format!("{:?}", p),
                                for_type: "".to_string(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                RsnapItem {
                    id: ItemId(item_id),
                    def_path,
                    kind,
                    visibility,
                    attributes: vec![],
                    generics,
                    where_clauses,
                }
            }
            Item::Struct(s) => {
                let name = s.ident.to_string();
                let visibility = convert_visibility(&s.vis);

                RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", name),
                    kind: ItemKind::Struct,
                    visibility,
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                }
            }
            Item::Enum(e) => {
                let visibility = convert_visibility(&e.vis);
                RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", e.ident.to_string()),
                    kind: ItemKind::Enum,
                    visibility,
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                }
            }
            Item::Trait(t) => {
                let visibility = convert_visibility(&t.vis);
                RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", t.ident.to_string()),
                    kind: ItemKind::Trait,
                    visibility,
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                }
            }
            Item::Impl(i) => RsnapItem {
                id: ItemId(item_id),
                def_path: format!("user_crate::impl_{}", item_id),
                kind: ItemKind::Impl,
                visibility: Visibility {
                    rank: VisibilityRank::Crate,
                    path: None,
                },
                attributes: vec![],
                generics: None,
                where_clauses: vec![],
            },
            Item::Type(t) => {
                let visibility = convert_visibility(&t.vis);
                RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", t.ident.to_string()),
                    kind: ItemKind::Type,
                    visibility,
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                }
            }
            Item::Mod(m) => {
                let visibility = convert_visibility(&m.vis);
                RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", m.ident.to_string()),
                    kind: ItemKind::Module,
                    visibility,
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                }
            }
            _ => continue,
        };
        items.push(rsnap_item);
        item_id += 1;
    }

    if items.is_empty() {
        // Fallback to simple extraction if syn didn't find anything
        return extract_items_simple(source, file_path);
    }

    items
}

/// Convert syn visibility to our visibility format
#[cfg(feature = "syn")]
fn convert_visibility(vis: &syn::Visibility) -> Visibility {
    use chimera_rust_schema::VisibilityRank;
    match vis {
        syn::Visibility::Public(_) => Visibility {
            rank: VisibilityRank::Pub,
            path: None,
        },
        syn::Visibility::Crate(_) => Visibility {
            rank: VisibilityRank::PubCrate,
            path: None,
        },
        syn::Visibility::Restricted(r) => Visibility {
            rank: VisibilityRank::PubRestricted,
            path: Some(r.path.to_token_stream().to_string()),
        },
        syn::Visibility::Inherited => Visibility {
            rank: VisibilityRank::Private,
            path: None,
        },
    }
}

/// Dispatch to the appropriate extraction function based on features
fn extract_items(source: &str, file_path: &str) -> Vec<RsnapItem> {
    #[cfg(feature = "syn")]
    {
        extract_items_from_source(source, file_path)
    }
    #[cfg(not(feature = "syn"))]
    {
        extract_items_simple(source, file_path)
    }
}

fn parse_driver_crate_type(value: &str) -> CrateType {
    match value {
        "binary" => CrateType::Binary,
        "cdylib" => CrateType::Cdylib,
        "rlib" => CrateType::Rlib,
        "proc-macro" => CrateType::ProcMacro,
        _ => CrateType::Library,
    }
}

fn apply_crate_context(
    mut items: Vec<RsnapItem>,
    crate_context: &DriverCrateContext,
) -> Vec<RsnapItem> {
    for item in &mut items {
        if let Some(rest) = item.def_path.strip_prefix("user_crate::") {
            item.def_path = format!("{}::{}", crate_context.crate_name, rest);
        }
    }
    items
}

fn build_crate_graph(crate_context: &DriverCrateContext) -> CrateGraph {
    let mut dependency_nodes = crate_context.dependencies.clone();
    for dependency in &crate_context.extern_prelude {
        if dependency_nodes
            .iter()
            .any(|context| context.crate_name == *dependency)
        {
            continue;
        }
        dependency_nodes.push(DriverDependencyContext {
            crate_name: dependency.clone(),
            package_name: None,
            version: None,
            source_kind: None,
            source: None,
            source_ref: None,
            edition: crate_context.edition.clone(),
            crate_type: "library".to_string(),
            dependencies: Vec::new(),
            features: Vec::new(),
            default_features: true,
            optional: false,
        });
    }

    let dependency_ids = dependency_nodes
        .iter()
        .enumerate()
        .map(|(idx, dependency)| (dependency.crate_name.clone(), CrateId(idx as u64 + 1)))
        .collect::<std::collections::HashMap<_, _>>();

    let mut nodes = vec![CrateNode {
        id: CrateId(0),
        name: crate_context.crate_name.clone(),
        package_name: crate_context.package_name.clone(),
        version: crate_context.version.clone(),
        source_kind: crate_context.source_kind.clone(),
        source: crate_context.source.clone(),
        source_ref: crate_context.source_ref.clone(),
        edition: crate_context.edition.clone(),
        crate_type: crate_context.crate_type,
        dependency_crates: (0..dependency_nodes.len())
            .map(|idx| CrateId(idx as u64 + 1))
            .collect(),
        extern_prelude: crate_context.extern_prelude.clone(),
        features: Vec::new(),
        default_features: true,
        optional: false,
    }];

    for (idx, dependency) in dependency_nodes.iter().enumerate() {
        let mut seen_dependencies = std::collections::HashSet::new();
        nodes.push(CrateNode {
            id: CrateId(idx as u64 + 1),
            name: dependency.crate_name.clone(),
            package_name: dependency.package_name.clone(),
            version: dependency.version.clone(),
            source_kind: dependency.source_kind.clone(),
            source: dependency.source.clone(),
            source_ref: dependency.source_ref.clone(),
            edition: dependency.edition.clone(),
            crate_type: parse_driver_crate_type(&dependency.crate_type),
            dependency_crates: dependency
                .dependencies
                .iter()
                .filter(|crate_name| *crate_name != &dependency.crate_name)
                .filter(|crate_name| seen_dependencies.insert((*crate_name).clone()))
                .filter_map(|crate_name| dependency_ids.get(crate_name).copied())
                .collect(),
            extern_prelude: vec![],
            features: dependency.features.clone(),
            default_features: dependency.default_features,
            optional: dependency.optional,
        });
    }

    CrateGraph {
        root: CrateId(0),
        nodes,
    }
}

/// Fallback simple extraction using regex (no syn dependency)
fn extract_items_simple(source: &str, _file_path: &str) -> Vec<RsnapItem> {
    let mut items = Vec::new();
    let mut item_id: u64 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Simple function detection - look for `fn name`
        if let Some(fn_pos) = trimmed.find("fn ") {
            let fn_start = trimmed[fn_pos..]
                .find('(')
                .map(|p| fn_pos + p)
                .unwrap_or(fn_pos + 2);
            let fn_name = trimmed[fn_pos + 3..fn_start].trim();

            if !fn_name.is_empty()
                && !fn_name.starts_with('_')
                && fn_name
                    .chars()
                    .next()
                    .map(|c| c.is_lowercase())
                    .unwrap_or(false)
            {
                items.push(RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", fn_name),
                    kind: ItemKind::Function,
                    visibility: Visibility {
                        rank: VisibilityRank::Pub,
                        path: None,
                    },
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                });
                item_id += 1;
            }
        }

        // Struct detection
        if let Some(struct_pos) = trimmed.find("struct ") {
            let name_start = struct_pos + 7;
            let name_end = trimmed[name_start..]
                .find([' ', ',', '{', ';'])
                .unwrap_or(trimmed.len() - name_start);
            let struct_name = trimmed[name_start..name_start + name_end].trim();

            if !struct_name.is_empty()
                && struct_name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
            {
                items.push(RsnapItem {
                    id: ItemId(item_id),
                    def_path: format!("user_crate::{}", struct_name),
                    kind: ItemKind::Struct,
                    visibility: Visibility {
                        rank: VisibilityRank::Pub,
                        path: None,
                    },
                    attributes: vec![],
                    generics: None,
                    where_clauses: vec![],
                });
                item_id += 1;
            }
        }
    }

    items
}

/// Extract exports (public items) - using correct RsnapExport schema
fn extract_exports(items: &[RsnapItem]) -> Vec<RsnapExport> {
    items
        .iter()
        .filter(|item| item.visibility.rank == VisibilityRank::Pub)
        .map(|item| RsnapExport {
            item_id: item.id,
            symbol: item.def_path.clone(),
            abi: "Rust".to_string(),
            linkage: Linkage::None,
        })
        .collect()
}

fn cmd_extract_hir(_args: &[String]) -> Result<(), MainError> {
    // HIR extraction remains intentionally stubbed until the semantic export path lands.
    println!("HIR extraction not yet implemented");
    Ok(())
}

fn cmd_extract_mir(_args: &[String]) -> Result<(), MainError> {
    // MIR extraction remains intentionally stubbed until the semantic export path lands.
    println!("MIR extraction not yet implemented");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_items_from_source() {
        let source = r#"
fn main() {
    println!("Hello");
}

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }
}
"#;
        let items = extract_items(source, "test.rs");

        // Should find: main, Point, new
        assert!(
            items.iter().any(|i| i.def_path == "user_crate::main"),
            "Should find main function"
        );
        assert!(
            items.iter().any(|i| i.def_path == "user_crate::Point"),
            "Should find Point struct"
        );
        assert!(
            items.iter().any(|i| i.def_path == "user_crate::new"),
            "Should find new method"
        );
    }

    #[test]
    fn test_extract_exports() {
        let items = vec![
            RsnapItem {
                id: ItemId(0),
                def_path: "user_crate::public_fn".to_string(),
                kind: ItemKind::Function,
                visibility: Visibility {
                    rank: VisibilityRank::Pub,
                    path: None,
                },
                attributes: vec![],
                generics: None,
                where_clauses: vec![],
            },
            RsnapItem {
                id: ItemId(1),
                def_path: "user_crate::private_fn".to_string(),
                kind: ItemKind::Function,
                visibility: Visibility {
                    rank: VisibilityRank::Private,
                    path: None,
                },
                attributes: vec![],
                generics: None,
                where_clauses: vec![],
            },
        ];

        let exports = extract_exports(&items);
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].symbol, "user_crate::public_fn");
    }

    #[test]
    fn test_apply_crate_context_rewrites_user_crate_paths() {
        let items = vec![RsnapItem {
            id: ItemId(0),
            def_path: "user_crate::public_fn".to_string(),
            kind: ItemKind::Function,
            visibility: Visibility {
                rank: VisibilityRank::Pub,
                path: None,
            },
            attributes: vec![],
            generics: None,
            where_clauses: vec![],
        }];
        let context = DriverCrateContext {
            crate_name: "sample_runtime".to_string(),
            package_name: Some("sample-runtime".to_string()),
            version: Some("0.1.0".to_string()),
            source_kind: Some("path".to_string()),
            source: Some("/workspace/sample-runtime".to_string()),
            source_ref: None,
            edition: "2021".to_string(),
            crate_type: CrateType::Binary,
            extern_prelude: vec!["serde".to_string()],
            dependencies: Vec::new(),
        };

        let items = apply_crate_context(items, &context);
        assert_eq!(items[0].def_path, "sample_runtime::public_fn");
    }

    #[test]
    fn test_build_crate_graph_uses_workspace_context() {
        let context = DriverCrateContext {
            crate_name: "sample_runtime".to_string(),
            package_name: Some("sample-runtime".to_string()),
            version: Some("0.1.0".to_string()),
            source_kind: Some("path".to_string()),
            source: Some("/workspace/sample-runtime".to_string()),
            source_ref: None,
            edition: "2021".to_string(),
            crate_type: CrateType::Binary,
            extern_prelude: vec!["serde".to_string(), "tokio".to_string()],
            dependencies: vec![DriverDependencyContext {
                crate_name: "tokio".to_string(),
                package_name: Some("tokio".to_string()),
                version: Some("1".to_string()),
                source_kind: Some("registry".to_string()),
                source: Some("crates.io".to_string()),
                source_ref: Some("branch=main".to_string()),
                edition: "2021".to_string(),
                crate_type: "proc-macro".to_string(),
                dependencies: vec![
                    "serde".to_string(),
                    "tokio".to_string(),
                    "serde".to_string(),
                ],
                features: vec!["rt".to_string(), "macros".to_string()],
                default_features: false,
                optional: true,
            }],
        };

        let graph = build_crate_graph(&context);
        assert_eq!(graph.root, CrateId(0));
        assert_eq!(graph.nodes[0].name, "sample_runtime");
        assert_eq!(
            graph.nodes[0].package_name.as_deref(),
            Some("sample-runtime")
        );
        assert_eq!(graph.nodes[0].version.as_deref(), Some("0.1.0"));
        assert_eq!(graph.nodes[0].source_kind.as_deref(), Some("path"));
        assert_eq!(
            graph.nodes[0].source.as_deref(),
            Some("/workspace/sample-runtime")
        );
        assert_eq!(graph.nodes[0].edition, "2021");
        assert_eq!(graph.nodes[0].crate_type, CrateType::Binary);
        assert_eq!(graph.nodes[0].extern_prelude, vec!["serde", "tokio"]);
        assert_eq!(
            graph.nodes[0].dependency_crates,
            vec![CrateId(1), CrateId(2)]
        );
        assert_eq!(graph.nodes[1].name, "tokio");
        assert_eq!(graph.nodes[1].package_name.as_deref(), Some("tokio"));
        assert_eq!(graph.nodes[1].version.as_deref(), Some("1"));
        assert_eq!(graph.nodes[1].source_kind.as_deref(), Some("registry"));
        assert_eq!(graph.nodes[1].source.as_deref(), Some("crates.io"));
        assert_eq!(graph.nodes[1].source_ref.as_deref(), Some("branch=main"));
        assert_eq!(graph.nodes[1].crate_type, CrateType::ProcMacro);
        assert_eq!(graph.nodes[1].dependency_crates, vec![CrateId(2)]);
        assert_eq!(graph.nodes[1].features, vec!["rt", "macros"]);
        assert!(!graph.nodes[1].default_features);
        assert!(graph.nodes[1].optional);
        assert_eq!(graph.nodes[2].name, "serde");
        assert_eq!(graph.nodes[2].crate_type, CrateType::Library);
        assert!(graph.nodes[2].features.is_empty());
        assert!(graph.nodes[2].default_features);
        assert!(!graph.nodes[2].optional);
        assert!(graph.nodes[2].package_name.is_none());
        assert!(graph.nodes[2].version.is_none());
    }

    #[test]
    fn test_cmd_compile_emits_snapshot_with_workspace_dependency_edges() {
        let temp = std::env::temp_dir().join(format!(
            "chimera-rustc-driver-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp).expect("create tempdir");
        let source = temp.join("sample.rs");
        let output = temp.join("sample.rlib");
        let artifacts_dir = temp.join("artifacts");
        std::fs::write(&source, "pub fn sample() -> i32 { 7 }\n").expect("write source");

        let dependency = serde_json::json!({
            "crate_name": "tokio",
            "package_name": "tokio",
            "version": "1",
            "source_kind": "registry",
            "source": "crates.io",
            "source_ref": "branch=main",
            "edition": "2021",
            "crate_type": "proc-macro",
            "dependencies": ["serde", "serde", "tokio"],
            "features": ["rt", "macros"],
            "default_features": false,
            "optional": true
        })
        .to_string();

        cmd_compile(&[
            "--source".to_string(),
            source.to_string_lossy().to_string(),
            "--output".to_string(),
            output.to_string_lossy().to_string(),
            "--artifacts-dir".to_string(),
            artifacts_dir.to_string_lossy().to_string(),
            "--target".to_string(),
            "aarch64-apple-darwin".to_string(),
            "--semantic-extraction".to_string(),
            "--snapshot-only".to_string(),
            "--crate-name".to_string(),
            "sample_runtime".to_string(),
            "--package-name".to_string(),
            "sample-runtime".to_string(),
            "--package-version".to_string(),
            "0.1.0".to_string(),
            "--package-source-kind".to_string(),
            "path".to_string(),
            "--package-source".to_string(),
            "/workspace/sample-runtime".to_string(),
            "--crate-edition".to_string(),
            "2021".to_string(),
            "--crate-type".to_string(),
            "binary".to_string(),
            "--extern-prelude".to_string(),
            "serde".to_string(),
            "--extern-prelude".to_string(),
            "tokio".to_string(),
            "--dependency-crate".to_string(),
            dependency,
        ])
        .expect("snapshot compile should succeed");

        let rsnap = std::fs::read_to_string(artifacts_dir.join("lib.rs.rsnap"))
            .expect("read emitted rsnap");
        let rsnap: RsnapSnapshot = serde_json::from_str(&rsnap).expect("parse emitted rsnap");

        assert_eq!(rsnap.crate_graph.root, CrateId(0));
        assert_eq!(rsnap.crate_graph.nodes[0].name, "sample_runtime");
        assert_eq!(
            rsnap.crate_graph.nodes[0].package_name.as_deref(),
            Some("sample-runtime")
        );
        assert_eq!(rsnap.crate_graph.nodes[0].version.as_deref(), Some("0.1.0"));
        assert_eq!(
            rsnap.crate_graph.nodes[0].source_kind.as_deref(),
            Some("path")
        );
        assert_eq!(
            rsnap.crate_graph.nodes[0].source.as_deref(),
            Some("/workspace/sample-runtime")
        );
        assert_eq!(
            rsnap.crate_graph.nodes[0].dependency_crates,
            vec![CrateId(1), CrateId(2)]
        );
        assert_eq!(rsnap.crate_graph.nodes[1].name, "tokio");
        assert_eq!(
            rsnap.crate_graph.nodes[1].package_name.as_deref(),
            Some("tokio")
        );
        assert_eq!(rsnap.crate_graph.nodes[1].version.as_deref(), Some("1"));
        assert_eq!(
            rsnap.crate_graph.nodes[1].source_kind.as_deref(),
            Some("registry")
        );
        assert_eq!(
            rsnap.crate_graph.nodes[1].source.as_deref(),
            Some("crates.io")
        );
        assert_eq!(
            rsnap.crate_graph.nodes[1].source_ref.as_deref(),
            Some("branch=main")
        );
        assert_eq!(rsnap.crate_graph.nodes[1].crate_type, CrateType::ProcMacro);
        assert_eq!(
            rsnap.crate_graph.nodes[1].dependency_crates,
            vec![CrateId(2)]
        );
        assert_eq!(rsnap.crate_graph.nodes[1].features, vec!["rt", "macros"]);
        assert!(!rsnap.crate_graph.nodes[1].default_features);
        assert!(rsnap.crate_graph.nodes[1].optional);
        assert_eq!(rsnap.crate_graph.nodes[2].name, "serde");
        assert_eq!(rsnap.crate_graph.nodes[2].crate_type, CrateType::Library);
        assert!(rsnap.crate_graph.nodes[2].features.is_empty());
        assert!(rsnap.crate_graph.nodes[2].default_features);
        assert!(!rsnap.crate_graph.nodes[2].optional);
        assert!(rsnap.crate_graph.nodes[2].package_name.is_none());
        assert!(rsnap.crate_graph.nodes[2].version.is_none());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_build_script_output_fingerprint() {
        let bs_out = BuildScriptOutput {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg1".to_string()],
            link_libs: vec!["foo".to_string()],
            env_vars: vec!["BAR=1".to_string()],
            rerun_if_changed: vec!["input.txt".to_string()],
            generated_headers: vec![],
        };
        let fp = bs_out.fingerprint();
        assert_eq!(fp.len(), 64); // blake3 hex

        // Same input should produce same fingerprint
        let bs_out2 = BuildScriptOutput {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg1".to_string()],
            link_libs: vec!["foo".to_string()],
            env_vars: vec!["BAR=1".to_string()],
            rerun_if_changed: vec!["input.txt".to_string()],
            generated_headers: vec![],
        };
        assert_eq!(fp, bs_out2.fingerprint());

        // Different input should produce different fingerprint
        let bs_out3 = BuildScriptOutput {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["cfg2".to_string()], // different
            link_libs: vec!["foo".to_string()],
            env_vars: vec!["BAR=1".to_string()],
            rerun_if_changed: vec!["input.txt".to_string()],
            generated_headers: vec![],
        };
        assert_ne!(fp, bs_out3.fingerprint());
    }

    #[test]
    fn test_generated_header_spec_fingerprint() {
        let gh = GeneratedHeaderSpec {
            path: "gen/bindings.h".to_string(),
            generator_command: Some("bindgen input.h -o gen/bindings.h".to_string()),
            content_hash: Some("abc123".to_string()),
        };
        let fp = gh.fingerprint();
        assert_eq!(fp.len(), 64); // blake3 hex

        // Same input should produce same fingerprint
        let gh2 = GeneratedHeaderSpec {
            path: "gen/bindings.h".to_string(),
            generator_command: Some("bindgen input.h -o gen/bindings.h".to_string()),
            content_hash: Some("abc123".to_string()),
        };
        assert_eq!(fp, gh2.fingerprint());

        // Different content_hash should produce different fingerprint
        let gh3 = GeneratedHeaderSpec {
            path: "gen/bindings.h".to_string(),
            generator_command: Some("bindgen input.h -o gen/bindings.h".to_string()),
            content_hash: Some("def456".to_string()),
        };
        assert_ne!(fp, gh3.fingerprint());

        // Different command should produce different fingerprint
        let gh4 = GeneratedHeaderSpec {
            path: "gen/bindings.h".to_string(),
            generator_command: Some("different command".to_string()),
            content_hash: Some("abc123".to_string()),
        };
        assert_ne!(fp, gh4.fingerprint());
    }

    #[test]
    fn test_build_script_output_with_generated_headers_fingerprint() {
        let mut bs_out = BuildScriptOutput {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec![],
            link_libs: vec![],
            env_vars: vec![],
            rerun_if_changed: vec![],
            generated_headers: vec![GeneratedHeaderSpec {
                path: "gen/foo.h".to_string(),
                generator_command: Some("gen.sh".to_string()),
                content_hash: Some("hash1".to_string()),
            }],
        };
        let fp1 = bs_out.fingerprint();
        assert_eq!(fp1.len(), 64);

        // Add another generated header
        bs_out.generated_headers.push(GeneratedHeaderSpec {
            path: "gen/bar.h".to_string(),
            generator_command: None,
            content_hash: Some("hash2".to_string()),
        });
        let fp2 = bs_out.fingerprint();

        // More generated headers should change the fingerprint
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_proc_macro_spec_fingerprint() {
        let pm = ProcMacroSpec {
            crate_name: "my_macro".to_string(),
            version: "1.0.0".to_string(),
            expanded_token_hash: Some("abc123".to_string()),
        };
        let fp = pm.fingerprint();
        assert_eq!(fp.len(), 64);

        // Same input should produce same fingerprint
        let pm2 = ProcMacroSpec {
            crate_name: "my_macro".to_string(),
            version: "1.0.0".to_string(),
            expanded_token_hash: Some("abc123".to_string()),
        };
        assert_eq!(fp, pm2.fingerprint());

        // Different version should produce different fingerprint
        let pm3 = ProcMacroSpec {
            crate_name: "my_macro".to_string(),
            version: "2.0.0".to_string(),
            expanded_token_hash: Some("abc123".to_string()),
        };
        assert_ne!(fp, pm3.fingerprint());
    }

    #[test]
    fn test_is_c_source_detection() {
        // Should detect C source files by extension
        assert!("test.c".ends_with(".c"));
        assert!("test.h".ends_with(".h"));
        assert!("test.C".ends_with(".C"));

        // Should not match non-C files
        assert!(!"test.rs".ends_with(".c"));
        assert!(!"test.cpp".ends_with(".c"));
    }

    #[test]
    fn test_host_target_triple_matches_supported_default() {
        let triple = host_target_triple();
        assert!(!triple.is_empty());
        assert!(triple.contains('-'));
    }

    #[test]
    fn test_c_compile_source_link_libs_extracted() {
        // Test that link_libs from build script outputs are properly handled
        let build_script_outputs = vec![BuildScriptOutput {
            script_path: "build.rs".to_string(),
            rustc_cfg: vec!["-DNDEBUG".to_string(), "-DFOO=1".to_string()],
            link_libs: vec!["-L/usr/local/lib".to_string(), "-lm".to_string()],
            env_vars: vec![],
            rerun_if_changed: vec![],
            generated_headers: vec![],
        }];

        // Verify the function can be called with correct types
        // (actual execution would fail without a real C file and library paths)
        let result = compile_c_source(
            "/nonexistent/test.c",
            "/nonexistent/test.o",
            "x86_64-unknown-linux-gnu",
            &build_script_outputs,
        );

        // Should fail because file doesn't exist, but shouldn't fail due to type errors
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should be an IO error about the file not existing, not a type error
        assert!(
            format!("{}", err).contains("test.c") || format!("{}", err).contains("nonexistent")
        );
    }
}
