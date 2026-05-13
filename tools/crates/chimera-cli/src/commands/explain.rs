//! `chimera explain` command
//!
//! Explains proof failures, trust assumptions, and rejected contracts.

use anyhow::{Context, Result};
use chimera_diagnostics::{Code, Diagnostic, DiagnosticBag, OutputFormat, Renderer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub fn run(file: PathBuf, level: Option<String>) -> Result<()> {
    log::info!("Running chimera explain on {:?}", file);

    let content =
        std::fs::read_to_string(&file).with_context(|| format!("Failed to read {:?}", file))?;

    let explain_input = parse_explain_input(&content)?;

    let format = OutputFormat::Plain;
    let renderer = Renderer::new(format);

    match explain_input {
        ExplainInput::Diagnostics(diags) => {
            for diag in diags.iter() {
                println!("{}", renderer.render(diag));
            }

            if let Some(explain_level) = level.as_deref() {
                if explain_level == "verbose" || explain_level == "detailed" {
                    for diag in diags.iter() {
                        print_detailed_explanation(diag);
                    }
                }
            }
        }
        ExplainInput::Cache(explanation) => print_cache_explanation(&explanation, level.as_deref()),
        ExplainInput::Component(explanation) => print_component_explanation(&explanation),
    }

    Ok(())
}

enum ExplainInput {
    Diagnostics(DiagnosticBag),
    Cache(CacheExplanation),
    Component(ComponentExplanation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentExplanation {
    pub component_id: String,
    pub status: String,
    pub explanation: String,
    pub artifacts: Vec<PathBuf>,
}

fn print_component_explanation(expl: &ComponentExplanation) {
    println!("Component: {}", expl.component_id);
    println!("Status: {}", expl.status);
    println!("Explanation: {}", expl.explanation);
    if !expl.artifacts.is_empty() {
        println!("Artifacts:");
        for art in &expl.artifacts {
            println!("  - {}", art.display());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyComponents {
    file: String,
    name: String,
    line: u32,
    column: u32,
    args_hash: String,
    target: String,
    builtins_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheDependencyFingerprint {
    kind: String,
    id: String,
    content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheReuseChecks {
    cached_entry_valid: bool,
    dep_graph_hash: String,
    build_options_hash: String,
    dependency_fingerprints: Vec<CacheDependencyFingerprint>,
    embed_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheExplainStatus {
    Hit,
    Miss,
    Rebuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CacheExplainReason {
    CacheHit,
    NoEntry,
    InvalidatedEntry,
    DependencyChanged {
        dependency_kind: String,
        dependency_id: String,
    },
    EmbedChanged {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheExplanation {
    artifact_kind: String,
    cache_key: String,
    status: CacheExplainStatus,
    reason: CacheExplainReason,
    key_components: CacheKeyComponents,
    reuse_checks: CacheReuseChecks,
}

fn parse_explain_input(content: &str) -> Result<ExplainInput> {
    if let Ok(explanation) = serde_json::from_str::<CacheExplanation>(content) {
        return Ok(ExplainInput::Cache(explanation));
    }

    if let Ok(explanation) = serde_json::from_str::<ComponentExplanation>(content) {
        return Ok(ExplainInput::Component(explanation));
    }

    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<Vec<Diagnostic>>(content) {
        let mut bag = DiagnosticBag::new();
        for diag in parsed {
            bag.push(diag);
        }
        return Ok(ExplainInput::Diagnostics(bag));
    }

    // Otherwise return empty bag (not a diagnostic file)
    Ok(ExplainInput::Diagnostics(DiagnosticBag::new()))
}

/// Generate a cache explanation for a C cache entry
pub fn explain_cache_entry(
    artifact_kind: &str,
    cache_key: &str,
    is_hit: bool,
    dependency_changed: Option<(&str, &str)>,
) -> CacheExplanation {
    let (status, reason) = if is_hit {
        (CacheExplainStatus::Hit, CacheExplainReason::CacheHit)
    } else if let Some((kind, id)) = dependency_changed {
        (
            CacheExplainStatus::Rebuild,
            CacheExplainReason::DependencyChanged {
                dependency_kind: kind.to_string(),
                dependency_id: id.to_string(),
            },
        )
    } else {
        (CacheExplainStatus::Miss, CacheExplainReason::NoEntry)
    };

    CacheExplanation {
        artifact_kind: artifact_kind.to_string(),
        cache_key: cache_key.to_string(),
        status,
        reason,
        key_components: CacheKeyComponents {
            file: String::new(),
            name: String::new(),
            line: 0,
            column: 0,
            args_hash: String::new(),
            target: String::new(),
            builtins_hash: String::new(),
        },
        reuse_checks: CacheReuseChecks {
            cached_entry_valid: is_hit,
            dep_graph_hash: String::new(),
            build_options_hash: String::new(),
            dependency_fingerprints: vec![],
            embed_files: vec![],
        },
    }
}

/// Format cache explanation as JSON for export
pub fn format_cache_explanation_json(explanation: &CacheExplanation) -> String {
    serde_json::to_string_pretty(explanation).unwrap_or_default()
}

fn print_cache_explanation(explanation: &CacheExplanation, level: Option<&str>) {
    println!("Cache status: {}", render_status(&explanation.status));
    println!("Artifact kind: {}", explanation.artifact_kind);
    println!("Cache key: {}", explanation.cache_key);
    println!("Reason: {}", render_reason(&explanation.reason));

    if matches!(level, Some("verbose" | "detailed")) {
        println!("\n=== Cache Key Components ===");
        println!("file: {}", explanation.key_components.file);
        println!("name: {}", explanation.key_components.name);
        println!("line: {}", explanation.key_components.line);
        println!("column: {}", explanation.key_components.column);
        println!("args_hash: {}", explanation.key_components.args_hash);
        println!("target: {}", explanation.key_components.target);
        println!(
            "builtins_hash: {}",
            explanation.key_components.builtins_hash
        );

        println!("\n=== Reuse Checks ===");
        println!(
            "cached_entry_valid: {}",
            explanation.reuse_checks.cached_entry_valid
        );
        println!(
            "dep_graph_hash: {}",
            explanation.reuse_checks.dep_graph_hash
        );
        println!(
            "build_options_hash: {}",
            explanation.reuse_checks.build_options_hash
        );

        if explanation.reuse_checks.dependency_fingerprints.is_empty() {
            println!("dependency_fingerprints: <none>");
        } else {
            for dep in &explanation.reuse_checks.dependency_fingerprints {
                println!(
                    "dependency_fingerprint: {}:{}={}",
                    dep.kind, dep.id, dep.content_hash
                );
            }
        }

        if explanation.reuse_checks.embed_files.is_empty() {
            println!("embed_files: <none>");
        } else {
            for embed in &explanation.reuse_checks.embed_files {
                println!("embed_file: {}", embed);
            }
        }
    }
}

fn render_status(status: &CacheExplainStatus) -> &'static str {
    match status {
        CacheExplainStatus::Hit => "hit",
        CacheExplainStatus::Miss => "miss",
        CacheExplainStatus::Rebuild => "rebuild",
    }
}

fn render_reason(reason: &CacheExplainReason) -> String {
    match reason {
        CacheExplainReason::CacheHit => "cache hit".to_string(),
        CacheExplainReason::NoEntry => "no cache entry matched the requested key".to_string(),
        CacheExplainReason::InvalidatedEntry => "cached entry was already invalidated".to_string(),
        CacheExplainReason::DependencyChanged {
            dependency_kind,
            dependency_id,
        } => format!("dependency changed: {dependency_kind}:{dependency_id}"),
        CacheExplainReason::EmbedChanged { path } => {
            format!("embedded file changed: {path}")
        }
    }
}

fn print_detailed_explanation(diag: &Diagnostic) {
    println!("\n=== Explanation ===");
    println!("Code: {} (E{})", diag.code.name(), diag.code.code());
    println!("Severity: {}", diag.severity);

    match diag.code {
        Code::ParseUnknownType => {
            println!("This error indicates that the parser encountered a type name it does not recognize.");
            println!("Possible causes:");
            println!("  - Typo in type name");
            println!("  - Missing import for the type's dialect");
            println!("  - Type not yet defined in the current scope");
        }
        Code::TypeMismatch => {
            println!("This error indicates a type incompatibility in the IR.");
            println!("Possible causes:");
            println!("  - Function return type mismatch");
            println!("  - Argument type mismatch in call");
            println!("  - Assignment with incompatible types");
        }
        Code::OwnershipDoubleBorrow => {
            println!("This error indicates that a value is being borrowed while it already has an active borrow.");
            println!("In Chimera's ownership model, a value can only be borrowed once at a time.");
            println!("Possible causes:");
            println!("  - Nested borrow without moving the first borrow");
            println!("  - Concurrent borrow of the same value");
        }
        _ => {
            println!("No detailed explanation available for this error code.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_diagnostics() {
        let input = parse_explain_input("").unwrap();
        match input {
            ExplainInput::Diagnostics(diags) => assert!(diags.is_empty()),
            ExplainInput::Cache(_) => panic!("expected diagnostics"),
            ExplainInput::Component(_) => panic!("expected diagnostics"),
        }
    }

    #[test]
    fn test_parse_json_diagnostics() {
        let json =
            r#"[{"code": "ParseUnknownType", "severity": "error", "message": "test error"}]"#;
        let input = parse_explain_input(json).unwrap();
        match input {
            ExplainInput::Diagnostics(diags) => assert_eq!(diags.len(), 1),
            ExplainInput::Cache(_) => panic!("expected diagnostics"),
            ExplainInput::Component(_) => panic!("expected diagnostics"),
        }
    }

    #[test]
    fn test_parse_cache_explanation_json() {
        let json = serde_json::to_string(&CacheExplanation {
            artifact_kind: "comptime".to_string(),
            cache_key: "comptime_abc".to_string(),
            status: CacheExplainStatus::Hit,
            reason: CacheExplainReason::CacheHit,
            key_components: CacheKeyComponents {
                file: "test.zig".to_string(),
                name: "compute".to_string(),
                line: 10,
                column: 2,
                args_hash: "arg-hash".to_string(),
                target: "x86_64-linux-gnu".to_string(),
                builtins_hash: "builtin-hash".to_string(),
            },
            reuse_checks: CacheReuseChecks {
                cached_entry_valid: true,
                dep_graph_hash: "graph-hash".to_string(),
                build_options_hash: "build-hash".to_string(),
                dependency_fingerprints: vec![CacheDependencyFingerprint {
                    kind: "Type".to_string(),
                    id: "Point".to_string(),
                    content_hash: "dep-hash".to_string(),
                }],
                embed_files: vec!["assets/point.bin".to_string()],
            },
        })
        .unwrap();

        let input = parse_explain_input(&json).unwrap();
        match input {
            ExplainInput::Cache(explanation) => {
                assert_eq!(explanation.status, CacheExplainStatus::Hit);
                assert_eq!(explanation.key_components.name, "compute");
            }
            ExplainInput::Diagnostics(_) => panic!("expected cache explanation"),
            ExplainInput::Component(_) => panic!("expected cache explanation"),
        }
    }

    #[test]
    fn test_parse_component_explanation_json() {
        let json = serde_json::to_string(&ComponentExplanation {
            component_id: "beam_runtime".to_string(),
            status: "built".to_string(),
            explanation: "semantic lowering succeeded".to_string(),
            artifacts: vec![PathBuf::from("build/beam_runtime.chimera")],
        })
        .unwrap();

        let input = parse_explain_input(&json).unwrap();
        match input {
            ExplainInput::Component(explanation) => {
                assert_eq!(explanation.component_id, "beam_runtime");
                assert_eq!(explanation.status, "built");
            }
            ExplainInput::Diagnostics(_) => panic!("expected component explanation"),
            ExplainInput::Cache(_) => panic!("expected component explanation"),
        }
    }

    #[test]
    fn test_explain_cache_entry_hit() {
        let explanation = explain_cache_entry("comptime", "key123", true, None);
        assert!(matches!(explanation.status, CacheExplainStatus::Hit));
        assert!(matches!(explanation.reason, CacheExplainReason::CacheHit));
        assert!(explanation.reuse_checks.cached_entry_valid);
    }

    #[test]
    fn test_explain_cache_entry_miss() {
        let explanation = explain_cache_entry("object", "key456", false, None);
        assert!(matches!(explanation.status, CacheExplainStatus::Miss));
        assert!(matches!(explanation.reason, CacheExplainReason::NoEntry));
        assert!(!explanation.reuse_checks.cached_entry_valid);
    }

    #[test]
    fn test_explain_cache_entry_dependency_changed() {
        let explanation = explain_cache_entry(
            "chimera_ir",
            "key789",
            false,
            Some(("header", "my_header.h")),
        );
        assert!(matches!(explanation.status, CacheExplainStatus::Rebuild));
        match &explanation.reason {
            CacheExplainReason::DependencyChanged {
                dependency_kind,
                dependency_id,
            } => {
                assert_eq!(dependency_kind, "header");
                assert_eq!(dependency_id, "my_header.h");
            }
            _ => panic!("expected DependencyChanged"),
        }
    }

    #[test]
    fn test_format_cache_explanation_json() {
        let explanation = explain_cache_entry("test_artifact", "test_key", true, None);
        let json = format_cache_explanation_json(&explanation);
        assert!(json.contains("test_artifact"));
        assert!(json.contains("test_key"));
        assert!(json.contains("hit"));
    }
}
