//! Bun integration commands for chimera-cli

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub enum BunCommands {
    /// Detect and display Bun repository root
    Detect {
        #[arg(
            short,
            long,
            help = "Starting path for detection (default: current directory)"
        )]
        path: Option<PathBuf>,
    },

    /// Capture Bun build session
    Session {
        #[arg(short, long, help = "Bun repository root")]
        bun_root: Option<PathBuf>,

        #[arg(short, long, help = "Output path for bun-session.json")]
        output: Option<PathBuf>,
    },

    /// Verify Bun toolchain setup
    Doctor {
        #[arg(short, long, help = "Bun repository root")]
        bun_root: Option<PathBuf>,
    },

    /// Plan Bun incremental build
    Plan {
        #[arg(short, long, help = "Previous session manifest")]
        previous: Option<PathBuf>,

        #[arg(short, long, help = "Current source root")]
        current: Option<PathBuf>,

        #[arg(short, long, help = "Output plan JSON")]
        output: Option<PathBuf>,
    },
}

/// Run Bun command
pub fn run(command: BunCommands) -> Result<()> {
    match command {
        BunCommands::Detect { path } => detect_bun(path),
        BunCommands::Session { bun_root, output } => capture_session(bun_root, output),
        BunCommands::Doctor { bun_root } => doctor_bun(bun_root),
        BunCommands::Plan {
            previous,
            current,
            output,
        } => plan_incremental(previous, current, output),
    }
}

fn detect_bun(path: Option<PathBuf>) -> Result<()> {
    use chimera_bun::detect_bun_repo_root;
    use chimera_bun::detect_pinned_zig;

    let start_path = path.unwrap_or_else(|| PathBuf::from("."));

    println!("Detecting Bun repository root...");

    let bun_root =
        detect_bun_repo_root(&start_path).context("Failed to detect Bun repository root")?;

    println!("✓ Bun repository found: {}", bun_root.path.display());
    println!("  has_bun_lock: {}", bun_root.has_bun_lock);
    println!("  has_build_zig: {}", bun_root.has_build_zig);
    println!("  source_dirs: {:?}", bun_root.source_dirs);

    // Also detect pinned zig
    if let Ok(zig_toolchain) = detect_pinned_zig(&bun_root.path) {
        println!("✓ Pinned Zig found: {}", zig_toolchain.zig_path.display());
        println!("  version: {}", zig_toolchain.zig_version);
        println!("  commit: {}", zig_toolchain.zig_commit);
        println!("  is_patched: {}", zig_toolchain.is_patched);
        println!(
            "  supports_zigmera_flags: {}",
            zig_toolchain.supports_zigmera_flags
        );
    } else {
        println!("⚠ Pinned Zig not detected");
    }

    Ok(())
}

fn capture_session(bun_root: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    use chimera_bun::{capture_bun_build_options, detect_bun_repo_root, detect_pinned_zig};
    use chimera_bun::{create_bun_session, write_bun_session};

    let start_path = bun_root.unwrap_or_else(|| PathBuf::from("."));
    let output_path = output.unwrap_or_else(|| PathBuf::from(".zigmera/bun-session.json"));

    println!("Capturing Bun build session...");

    let bun_repo = detect_bun_repo_root(&start_path).context("Failed to detect Bun repository")?;

    let zig_toolchain = detect_pinned_zig(&bun_repo.path).context("Failed to detect pinned Zig")?;

    let build_options =
        capture_bun_build_options(&bun_repo.path).context("Failed to capture build options")?;

    let session = create_bun_session(&bun_repo, &zig_toolchain, &build_options)
        .context("Failed to create bun session")?;

    write_bun_session(&session, &output_path).context("Failed to write bun session")?;

    println!("✓ Bun session written to: {}", output_path.display());
    println!("  bun_repo_root: {}", session.bun_repo_root);
    println!("  zig_version: {}", session.zig_toolchain.zig_version);
    println!("  source_files: {}", session.source_files.len());
    println!("  optimize_mode: {}", session.build_options.optimize_mode);
    println!("  target: {}", session.build_options.target);

    Ok(())
}

fn doctor_bun(bun_root: Option<PathBuf>) -> Result<()> {
    use chimera_bun::{capture_bun_build_options, detect_bun_repo_root, detect_pinned_zig};

    let start_path = bun_root.unwrap_or_else(|| PathBuf::from("."));

    println!("=== Bun Doctor ===\n");

    let mut all_ok = true;

    // Check Bun repo root
    print!("Bun repository root: ");
    match detect_bun_repo_root(&start_path) {
        Ok(root) => {
            println!("✓ {}", root.path.display());
            if root.has_bun_lock {
                println!("  ✓ bun.lock present");
            } else {
                println!("  ⚠ bun.lock not found");
            }
            if root.has_build_zig {
                println!("  ✓ build.zig present");
            } else {
                println!("  ✗ build.zig not found");
                all_ok = false;
            }
        }
        Err(e) => {
            println!("✗ Not a Bun repository");
            println!("  Error: {}", e);
            all_ok = false;
        }
    }

    println!();

    // Check pinned Zig
    print!("Pinned Zig toolchain: ");
    if let Ok(zig) = detect_pinned_zig(&start_path) {
        println!("✓ {}", zig.zig_path.display());
        println!("  version: {}", zig.zig_version);
        println!("  commit: {}", zig.zig_commit);
        if zig.is_patched {
            println!("  ✓ Patched (supports ZigMera flags)");
        } else {
            println!("  ⚠ Not patched (--emit-zigmera-* flags may not work)");
        }
        if zig.supports_zigmera_flags {
            println!("  ✓ --emit-zigmera-snapshot supported");
        } else {
            println!("  ⚠ --emit-zigmera-snapshot not in --help output");
        }
    } else {
        println!("✗ Pinned Zig not found");
        all_ok = false;
    }

    println!();

    // Check build options
    print!("Build options: ");
    match capture_bun_build_options(&start_path) {
        Ok(opts) => {
            println!("✓ Captured");
            println!("  optimize_mode: {}", opts.optimize_mode);
            println!("  target: {}", opts.target);
            if let Some(cpu) = &opts.target_cpu {
                println!("  cpu: {}", cpu);
            }
            if let Some(sanitize) = &opts.sanitize {
                println!("  sanitize: {}", sanitize);
            }
            if let Some(lto) = &opts.lto {
                println!("  lto: {}", lto);
            }
        }
        Err(e) => {
            println!("⚠ Could not capture (build.zig may not exist)");
            println!("  Error: {}", e);
        }
    }

    println!();

    if all_ok {
        println!("=== All checks passed ===");
    } else {
        println!("=== Some checks failed ===");
    }

    Ok(())
}

fn plan_incremental(
    previous: Option<PathBuf>,
    current: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    use chimera_bun::load_bun_session;
    use chimera_bun::sessions_match;

    let prev_path = previous.unwrap_or_else(|| PathBuf::from(".zigmera/bun-session.json"));
    let curr_path = current.unwrap_or_else(|| PathBuf::from(".zigmera/bun-session.json"));
    let output_path = output.unwrap_or_else(|| PathBuf::from(".zigmera/build-plan.json"));

    println!("Planning Bun incremental build...");

    // Load previous session
    let prev_session = match load_bun_session(&prev_path) {
        Ok(s) => {
            println!("✓ Previous session loaded: {}", prev_path.display());
            s
        }
        Err(e) => {
            println!(
                "⚠ No previous session found at {}, will do full rebuild",
                prev_path.display()
            );
            println!("  Error: {}", e);
            // Return early with full rebuild plan
            let plan = serde_json::json!({
                "decision": "rebuild",
                "reason": "no previous session found",
                "changed_nodes": [],
                "abi_changed": false,
                "objects_restored": []
            });
            std::fs::write(&output_path, serde_json::to_string_pretty(&plan)?)?;
            println!("✓ Build plan written to: {}", output_path.display());
            return Ok(());
        }
    };

    // Load current session
    let curr_session = load_bun_session(&curr_path).context("Failed to load current session")?;

    // Compare sessions
    let sessions_match = sessions_match(&prev_session, &curr_session);

    let plan = if sessions_match {
        serde_json::json!({
            "decision": "reuse",
            "reason": "no semantic changes detected",
            "changed_nodes": [],
            "abi_changed": false,
            "objects_restored": ["zig-out/*.o"]
        })
    } else {
        serde_json::json!({
            "decision": "rebuild",
            "reason": "session mismatch - rebuild required",
            "changed_nodes": ["build.zig", "source files"],
            "abi_changed": true,
            "objects_restored": []
        })
    };

    std::fs::write(&output_path, serde_json::to_string_pretty(&plan)?)?;
    println!("✓ Build plan written to: {}", output_path.display());
    println!("  decision: {}", plan["decision"]);
    println!("  reason: {}", plan["reason"]);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_bun::{BunBuildOptions, BunRepoRoot, BunSession, BunZigToolchain};
    use std::collections::HashMap;

    #[test]
    fn test_bun_session_json_structure() {
        let mut artifact_hashes = HashMap::new();
        artifact_hashes.insert("build_zig".to_string(), "abc123".to_string());

        let session = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: Some("def456".to_string()),
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/test/zig"),
                zig_stdlib_path: PathBuf::from("/test/lib"),
                zig_commit: "v0.13.0".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec!["src/main.zig".to_string()],
            generated_files: vec!["zig-out/main.o".to_string()],
            output_dir: "zig-out".to_string(),
            artifact_hashes,
            captured_ns: 1000000000,
        };

        let json = serde_json::to_string_pretty(&session).unwrap();
        assert!(json.contains("\"version\": \"0.1.0\""));
        assert!(json.contains("\"bun_repo_root\": \"/test/bun\""));
        assert!(json.contains("\"zig_version\": \"0.13.0\""));
        assert!(json.contains("\"source_files\""));
        assert!(json.contains("\"artifact_hashes\""));
    }

    #[test]
    fn test_bun_build_options_serialization() {
        let mut options = BunBuildOptions::default();
        options.optimize_mode = "ReleaseFast".to_string();
        options.target = "x86_64-unknown-linux-gnu".to_string();
        options.target_cpu = Some("baseline".to_string());
        options.sanitize = Some("none".to_string());
        options.lto = Some("thin".to_string());
        options
            .feature_flags
            .insert("filter_none".to_string(), true);

        let json = serde_json::to_string_pretty(&options).unwrap();
        assert!(json.contains("\"optimize_mode\": \"ReleaseFast\""));
        assert!(json.contains("\"target\": \"x86_64-unknown-linux-gnu\""));
        assert!(json.contains("\"target_cpu\": \"baseline\""));
        assert!(json.contains("\"sanitize\": \"none\""));
        assert!(json.contains("\"lto\": \"thin\""));
        assert!(json.contains("\"feature_flags\""));
    }
}
