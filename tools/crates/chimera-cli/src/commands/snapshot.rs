//! `chimera snapshot` command
//!
//! Reads and validates `.zsnap` binary snapshot files emitted by the patched Zig compiler.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use zigmera_schema::{BinaryParser, ZSNAP_MAGIC};

#[derive(Parser, Debug)]
#[command(
    name = "snapshot",
    about = "Read and validate Zig semantic snapshot (.zsnap) files"
)]
pub enum SnapshotCommand {
    /// Read and display snapshot metadata
    Read {
        /// Path to the `.zsnap` file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Show full details including all declarations and types
        #[arg(short, long)]
        verbose: bool,
    },
    /// Validate snapshot file integrity
    Validate {
        /// Path to the `.zsnap` file
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Output JSON format for machine parsing
        #[arg(short, long)]
        json: bool,
    },
}

/// Run the snapshot command
pub fn run(cmd: SnapshotCommand) -> Result<()> {
    match cmd {
        SnapshotCommand::Read { file, verbose } => run_read(file, verbose),
        SnapshotCommand::Validate { file, json } => run_validate(file, json),
    }
}

fn run_read(file: PathBuf, verbose: bool) -> Result<()> {
    log::info!("Reading snapshot from {:?}", file);

    let mut parser = BinaryParser::new();
    let schema = parser
        .parse_file(&file)
        .with_context(|| format!("Failed to parse snapshot file: {}", file.display()))?;

    println!("Snapshot Metadata:");
    println!("  Magic: {:?}", schema.header.magic);
    println!("  Schema Version: {}", schema.header.schema_version);
    println!("  Target: {}", schema.header.target);
    println!("  Backend: {}", schema.header.backend);
    println!("  Optimize Mode: {}", schema.header.optimize_mode);
    println!("  Timestamp: {}", schema.header.timestamp_ns);
    println!("  Source Files: {}", schema.header.source_file_count);
    println!("  Zig Commit: {:?}", schema.header.zig_commit);

    println!("\nSections:");
    println!("  Source Files: {}", schema.source_files.len());
    println!("  Declarations: {}", schema.decls.len());
    println!("  Analysis Units: {}", schema.analysis_units.len());
    println!("  Types: {}", schema.types.len());
    println!("  Layouts: {}", schema.layouts.len());
    println!("  AIR Bodies: {}", schema.air_bodies.len());
    println!("  Exports: {}", schema.exports.len());

    if verbose {
        println!("\n=== Build Options ===");
        println!("  Optimize Mode: {}", schema.build_options.optimize_mode);
        println!("  Target: {}", schema.build_options.target);
        println!("  Build Mode: {}", schema.build_options.build_mode);
        println!("  Panic Mode: {}", schema.build_options.panic_mode);
        if let Some(ref entry) = schema.build_options.entry {
            println!("  Entry: {}", entry);
        }
        if let Some(ref libc) = schema.build_options.libc {
            println!("  Libc: {}", libc);
        }
        println!("  CPU Features: {:?}", schema.build_options.cpu_features);

        if !schema.source_files.is_empty() {
            println!("\n=== Source Files ===");
            for sf in &schema.source_files {
                println!(
                    "  [{}] {} (hash: {:?})",
                    sf.id,
                    sf.path,
                    &sf.content_hash[..8]
                );
            }
        }

        if !schema.exports.is_empty() {
            println!("\n=== Exports ===");
            for exp in &schema.exports {
                println!(
                    "  {} (decl_id: {}, linkage: {:?})",
                    exp.name, exp.decl_id, exp.linkage
                );
            }
        }
    }

    Ok(())
}

fn run_validate(file: PathBuf, json: bool) -> Result<()> {
    log::info!("Validating snapshot {:?}", file);

    let mut parser = BinaryParser::new();
    match parser.parse_file(&file) {
        Ok(schema) => {
            if json {
                let report = serde_json::json!({
                    "valid": true,
                    "version": schema.header.schema_version,
                    "target": schema.header.target,
                    "source_file_count": schema.header.source_file_count,
                    "errors": []
                });
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("✓ Snapshot file is valid");
                println!("  Schema version: {}", schema.header.schema_version);
                println!("  Target: {}", schema.header.target);
                println!("  Source files: {}", schema.header.source_file_count);
            }
        }
        Err(e) => {
            if json {
                let report = serde_json::json!({
                    "valid": false,
                    "version": 0,
                    "target": "",
                    "source_file_count": 0,
                    "errors": [e.to_string()]
                });
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("✗ Snapshot file is invalid: {}", e);
                if e.is_version_error() {
                    println!("  This indicates the file was produced by a newer Zig compiler.");
                    println!("  The adapter may need to be updated.");
                } else if e.is_corruption() {
                    println!("  This indicates the file may be corrupted or truncated.");
                }
            }
            anyhow::bail!("validation failed");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_snapshot_command_read_verbose() {
        // Create a minimal valid .zsnap file for testing
        let mut file = NamedTempFile::new().unwrap();
        // Write magic
        file.write_all(ZSNAP_MAGIC).unwrap();
        // Write schema version (1)
        file.write_all(&1u32.to_le_bytes()).unwrap();
        // Write zig commit (20 zeros)
        file.write_all(&[0u8; 20]).unwrap();
        // Write target string (length-prefixed)
        let target = "x86_64-unknown-linux-gnu";
        file.write_all(&(target.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(target.as_bytes()).unwrap();
        // Write backend string
        let backend = "llvm";
        file.write_all(&(backend.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(backend.as_bytes()).unwrap();
        // Write optimize_mode string
        let opt = "ReleaseFast";
        file.write_all(&(opt.len() as u32).to_le_bytes()).unwrap();
        file.write_all(opt.as_bytes()).unwrap();
        // Write timestamp_ns
        file.write_all(&1234567890u64.to_le_bytes()).unwrap();
        // Write source_file_count
        file.write_all(&0u32.to_le_bytes()).unwrap();
        // Write checksum (32 zeros)
        file.write_all(&[0u8; 32]).unwrap();
        // Write empty JSON payload
        file.write_all(b"{}").unwrap();
        file.flush().unwrap();

        let cmd = SnapshotCommand::Read {
            file: file.path().to_path_buf(),
            verbose: false,
        };
        // Just verify it doesn't panic
        let result = run(cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rejects_invalid_magic() {
        let mut file = NamedTempFile::new().unwrap();
        // Write invalid magic
        file.write_all(b"NOTVALID").unwrap();
        file.write_all(&[0u8; 100]).unwrap();
        file.flush().unwrap();

        let cmd = SnapshotCommand::Validate {
            file: file.path().to_path_buf(),
            json: false,
        };
        let result = run(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_truncated_file() {
        let mut file = NamedTempFile::new().unwrap();
        // Write only magic bytes (truncated header)
        file.write_all(ZSNAP_MAGIC).unwrap();
        file.flush().unwrap();

        let cmd = SnapshotCommand::Validate {
            file: file.path().to_path_buf(),
            json: false,
        };
        let result = run(cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_json_output() {
        let mut file = NamedTempFile::new().unwrap();
        // Write magic
        file.write_all(ZSNAP_MAGIC).unwrap();
        // Write schema version
        file.write_all(&1u32.to_le_bytes()).unwrap();
        // Write zig commit
        file.write_all(&[0u8; 20]).unwrap();
        // Write target string
        let target = "x86_64-unknown-linux-gnu";
        file.write_all(&(target.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(target.as_bytes()).unwrap();
        // Write backend string
        let backend = "llvm";
        file.write_all(&(backend.len() as u32).to_le_bytes())
            .unwrap();
        file.write_all(backend.as_bytes()).unwrap();
        // Write optimize_mode string
        let opt = "ReleaseFast";
        file.write_all(&(opt.len() as u32).to_le_bytes()).unwrap();
        file.write_all(opt.as_bytes()).unwrap();
        // Write timestamp_ns
        file.write_all(&1234567890u64.to_le_bytes()).unwrap();
        // Write source_file_count
        file.write_all(&5u32.to_le_bytes()).unwrap();
        // Write checksum
        file.write_all(&[0u8; 32]).unwrap();
        // Write empty JSON payload
        file.write_all(b"{}").unwrap();
        file.flush().unwrap();

        let cmd = SnapshotCommand::Validate {
            file: file.path().to_path_buf(),
            json: true,
        };
        let result = run(cmd);
        assert!(result.is_ok());
    }
}
