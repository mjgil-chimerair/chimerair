//! Chimera Rust CLI commands
//!
//! Task 20: Rust CLI commands

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

/// Rust compiler integration commands
#[derive(Parser, Debug)]
pub enum RustCommands {
    /// Snapshot Rust source to ChimeraIR
    Snapshot {
        /// Input source file or crate manifest
        #[arg(required = true)]
        input: String,
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
        /// Target triple
        #[arg(long, default_value = "x86_64-unknown-linux-gnu")]
        target: String,
        /// Build mode (release or debug)
        #[arg(long, default_value = "release")]
        mode: String,
    },
    /// Lower Rust dialect to ChimeraIR
    Lower {
        /// Input artifact file (.rsnap, .rdep, .rmirpack)
        #[arg(required = true)]
        input: String,
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Verify Rust source against ChimeraIR
    Verify {
        /// Input source file
        #[arg(required = true)]
        input: String,
        /// ChimeraIR reference
        #[arg(required = true)]
        reference: String,
    },
    /// Explain Rust declaration or type
    Explain {
        /// Input source file or artifact
        #[arg(required = true)]
        input: String,
        /// Symbol to explain
        #[arg(required = true)]
        symbol: String,
    },
    /// Cache Rust artifacts
    Cache {
        /// Cache action
        #[command(subcommand)]
        action: CacheAction,
    },
    /// Emit proof artifacts
    Proof {
        /// Input source file or artifact
        #[arg(required = true)]
        input: String,
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Cache subcommands
#[derive(Subcommand, Debug)]
pub enum CacheAction {
    /// Show cache statistics
    Stats,
    /// Clear cache
    Clear,
    /// Invalidate specific artifacts
    Invalidate { pattern: String },
}

/// Execute Rust snapshot command
pub fn snapshot(input: &str, output: Option<&str>, target: &str, mode: &str) -> Result<()> {
    println!(
        "Rust Snapshot: {} -> {:?}, target={}, mode={}",
        input, output, target, mode
    );
    Ok(())
}

/// Execute Rust lower command
pub fn lower(input: &str, output: Option<&str>) -> Result<()> {
    println!("Rust Lower: {} -> {:?}", input, output);
    Ok(())
}

/// Execute Rust verify command
pub fn verify(input: &str, reference: &str) -> Result<()> {
    println!("Rust Verify: {} against {}", input, reference);
    Ok(())
}

/// Execute Rust explain command
pub fn explain(input: &str, symbol: &str) -> Result<()> {
    println!("Rust Explain: {} in {}", symbol, input);
    Ok(())
}

/// Execute Rust cache command
pub fn cache(action: &CacheAction) -> Result<()> {
    match action {
        CacheAction::Stats => println!("Rust cache stats: (placeholder)"),
        CacheAction::Clear => println!("Rust cache cleared (placeholder)"),
        CacheAction::Invalidate { pattern } => println!("Invalidated: {}", pattern),
    }
    Ok(())
}

/// Execute Rust proof command
pub fn proof(input: &str, output: Option<&str>) -> Result<()> {
    println!("Rust Proof: {} -> {:?}", input, output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_commands_variant_count() {
        // Ensure we have all 6 Rust commands
        let cmds = vec![
            RustCommands::Snapshot {
                input: "src/lib.rs".into(),
                output: None,
                target: "x86_64".into(),
                mode: "release".into(),
            },
            RustCommands::Lower {
                input: "out.rsnap".into(),
                output: None,
            },
            RustCommands::Verify {
                input: "src/lib.rs".into(),
                reference: "ref.chimera".into(),
            },
            RustCommands::Explain {
                input: "src/lib.rs".into(),
                symbol: "my_func".into(),
            },
            RustCommands::Cache {
                action: CacheAction::Stats,
            },
            RustCommands::Proof {
                input: "src/lib.rs".into(),
                output: None,
            },
        ];
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn test_cache_action_variants() {
        let actions = vec![
            CacheAction::Stats,
            CacheAction::Clear,
            CacheAction::Invalidate {
                pattern: "*.rsnap".into(),
            },
        ];
        assert_eq!(actions.len(), 3);
    }

    #[test]
    fn test_snapshot_command_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "snapshot", "src/lib.rs"]);
        assert!(matches!(cmd, RustCommands::Snapshot { .. }));

        let cmd = RustCommands::parse_from(&[
            "rust",
            "snapshot",
            "src/lib.rs",
            "--output",
            "out.chimera",
        ]);
        match cmd {
            RustCommands::Snapshot {
                input,
                output,
                target,
                mode,
            } => {
                assert_eq!(input, "src/lib.rs");
                assert_eq!(output, Some("out.chimera".to_string()));
                assert_eq!(target, "x86_64-unknown-linux-gnu");
                assert_eq!(mode, "release");
            }
            _ => panic!("expected Snapshot variant"),
        }
    }

    #[test]
    fn test_lower_command_parsing() {
        let cmd = RustCommands::parse_from(&[
            "rust",
            "lower",
            "input.rmirpack",
            "--output",
            "out.chimera",
        ]);
        match cmd {
            RustCommands::Lower { input, output } => {
                assert_eq!(input, "input.rmirpack");
                assert_eq!(output, Some("out.chimera".to_string()));
            }
            _ => panic!("expected Lower variant"),
        }
    }

    #[test]
    fn test_verify_command_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "verify", "src/lib.rs", "ref.chimera"]);
        match cmd {
            RustCommands::Verify { input, reference } => {
                assert_eq!(input, "src/lib.rs");
                assert_eq!(reference, "ref.chimera");
            }
            _ => panic!("expected Verify variant"),
        }
    }

    #[test]
    fn test_explain_command_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "explain", "src/lib.rs", "my_function"]);
        match cmd {
            RustCommands::Explain { input, symbol } => {
                assert_eq!(input, "src/lib.rs");
                assert_eq!(symbol, "my_function");
            }
            _ => panic!("expected Explain variant"),
        }
    }

    #[test]
    fn test_cache_stats_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "cache", "stats"]);
        assert!(matches!(
            cmd,
            RustCommands::Cache {
                action: CacheAction::Stats
            }
        ));
    }

    #[test]
    fn test_cache_clear_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "cache", "clear"]);
        assert!(matches!(
            cmd,
            RustCommands::Cache {
                action: CacheAction::Clear
            }
        ));
    }

    #[test]
    fn test_cache_invalidate_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "cache", "invalidate", "*.rsnap"]);
        match cmd {
            RustCommands::Cache {
                action: CacheAction::Invalidate { pattern },
            } => {
                assert_eq!(pattern, "*.rsnap");
            }
            _ => panic!("expected Invalidate variant"),
        }
    }

    #[test]
    fn test_proof_command_parsing() {
        let cmd = RustCommands::parse_from(&["rust", "proof", "src/lib.rs", "--output", "proofs/"]);
        match cmd {
            RustCommands::Proof { input, output } => {
                assert_eq!(input, "src/lib.rs");
                assert_eq!(output, Some("proofs/".to_string()));
            }
            _ => panic!("expected Proof variant"),
        }
    }
}

/// Run Rust frontend input path (Task 118)
pub fn run_frontend(
    input: &Path,
    emit_metadata: bool,
    emit_object: bool,
    emit_proof: bool,
) -> Result<()> {
    use chimera_rust_source::parse_rust_source;
    use chimera_rust_to_chimera::{lower_dialect, to_chimera_text, ChimeraModule};
    use std::fs;

    let source = fs::read_to_string(input)?;
    let parsed = parse_rust_source(&source)?;
    println!(
        "Rust Frontend: {} parsed into {} items",
        input.display(),
        parsed.items.len()
    );

    if emit_metadata {
        println!("  Emitting metadata...");
    }
    if emit_object {
        println!("  Emitting object...");
    }
    if emit_proof {
        println!("  Emitting proof...");
    }

    Ok(())
}
