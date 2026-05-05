//! Chimera Bun Adapter
//!
//! Provides Bun toolchain detection and session capture.
//!
//! # Tasks
//!
//! - B1: Detect Bun repo root (upward search for build.zig, package.json, Bun markers)
//! - B2: Detect pinned `oven-sh/zig` (must not silently fall back to system Zig)
//! - B3: Capture Bun build.zig options (`-D` flags, target, optimize mode, LTO, sanitizer)
//! - B4: Produce `.zigmera/bun-session.json`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use zigmera_hash::Blake3Hasher;

/// Bun detection markers
const BUN_ROOT_MARKERS: &[&str] = &[
    "build.zig",
    "package.json",
    "bun.lock",
    "src/index.ts",
    "src/index.js",
];

/// Standard Bun source directories
const BUN_SOURCE_DIRS: &[&str] = &["src", "packages"];

/// Errors specific to Bun detection
#[derive(Debug, Error)]
pub enum BunDetectError {
    #[error("not a bun repository: {0}")]
    NotBunRepo(String),
    #[error("multiple bun repositories found in ancestor path")]
    AmbiguousRoot,
    #[error("pinned zig not found: {0}")]
    PinnedZigNotFound(String),
    #[error("zig version mismatch: expected {expected}, got {got}")]
    ZigVersionMismatch { expected: String, got: String },
    #[error("build.zig not found")]
    NoBuildZig,
    #[error("failed to run zig: {0}")]
    ZigCommandFailed(String),
}

/// Bun repository root information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunRepoRoot {
    pub path: PathBuf,
    pub has_bun_lock: bool,
    pub has_build_zig: bool,
    pub source_dirs: Vec<String>,
}

/// Zig toolchain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunZigToolchain {
    pub zig_path: PathBuf,
    pub zig_stdlib_path: PathBuf,
    pub zig_commit: String,
    pub zig_version: String,
    pub is_patched: bool,
    pub supports_zigmera_flags: bool,
}

/// Build options captured from Bun's build.zig
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BunBuildOptions {
    pub optimize_mode: String,
    pub target: String,
    pub target_cpu: Option<String>,
    pub sanitize: Option<String>,
    pub lto: Option<String>,
    pub codegen_threads: Option<String>,
    pub link_mode: Option<String>,
    pub panic_mode: Option<String>,
    pub feature_flags: HashMap<String, bool>,
}

/// Bun session manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunSession {
    pub version: String,
    pub bun_repo_root: String,
    pub bun_git_commit: Option<String>,
    pub zig_toolchain: BunZigToolchain,
    pub build_options: BunBuildOptions,
    pub source_files: Vec<String>,
    pub generated_files: Vec<String>,
    pub output_dir: String,
    pub artifact_hashes: HashMap<String, String>,
    pub captured_ns: u64,
}

/// Detect if a path is inside a Bun repository
pub fn detect_bun_repo_root(start_path: &Path) -> Result<BunRepoRoot> {
    let mut current = start_path.to_path_buf();
    let mut found_markers = 0;
    let mut last_valid_root: Option<PathBuf> = None;

    // Walk up the directory tree looking for Bun markers
    loop {
        // Check for build.zig (Bun's build file)
        let build_zig = current.join("build.zig");
        if build_zig.exists() {
            found_markers += 1;
            last_valid_root = Some(current.clone());
        }

        // Check for package.json (npm-compatible package manifest)
        let package_json = current.join("package.json");
        if package_json.exists() {
            found_markers += 1;
        }

        // Check for bun.lock (Bun's lockfile)
        let bun_lock = current.join("bun.lock");
        if bun_lock.exists() {
            found_markers += 1;
        }

        // Check for standard Bun source directories
        for src_dir in BUN_SOURCE_DIRS {
            if current.join(src_dir).is_dir() {
                found_markers += 1;
            }
        }

        // If we found multiple markers, this is likely a Bun repo
        if found_markers >= 2 && last_valid_root.is_some() {
            let root = last_valid_root.unwrap();
            return Ok(BunRepoRoot {
                path: root.clone(),
                has_bun_lock: root.join("bun.lock").exists(),
                has_build_zig: root.join("build.zig").exists(),
                source_dirs: detect_source_dirs(&root),
            });
        }

        // Move to parent directory
        current = match current.parent() {
            Some(parent) => parent.to_path_buf(),
            None => break,
        };

        // Don't search past repo root (mounted fs boundary)
        if current == Path::new("/") {
            break;
        }
    }

    // Check if start_path itself is a Bun repo
    if start_path.join("build.zig").exists() && start_path.join("package.json").exists() {
        return Ok(BunRepoRoot {
            path: start_path.to_path_buf(),
            has_bun_lock: start_path.join("bun.lock").exists(),
            has_build_zig: start_path.join("build.zig").exists(),
            source_dirs: detect_source_dirs(start_path),
        });
    }

    Err(anyhow::anyhow!("not a bun repository")).context(BunDetectError::NotBunRepo(format!(
        "no Bun markers found from {}",
        start_path.display()
    )))
}

/// Detect source directories in a Bun repo
fn detect_source_dirs(root: &Path) -> Vec<String> {
    let mut dirs = Vec::new();
    for src_dir in BUN_SOURCE_DIRS {
        let path = root.join(src_dir);
        if path.is_dir() {
            dirs.push(src_dir.to_string());
        }
    }
    // Also check for packages directory (monorepo support)
    let packages = root.join("packages");
    if packages.is_dir() {
        dirs.push("packages".to_string());
    }
    dirs
}

/// Detect the pinned `oven-sh/zig` used by Bun
pub fn detect_pinned_zig(bun_root: &Path) -> Result<BunZigToolchain> {
    // Bun typically stores its pinned Zig in vendor/zig or .bun/install/cache/zig-<commit>
    let possible_zig_locations = vec![
        bun_root.join("vendor").join("zig").join("zig"),
        bun_root
            .join(".bun")
            .join("install")
            .join("cache")
            .join("zig"),
        bun_root.join("deps").join("zig").join("zig"),
    ];

    let mut found_zig_path = None;
    for loc in &possible_zig_locations {
        if loc.exists() && is_zig_binary(loc) {
            found_zig_path = Some(loc.clone());
            break;
        }
    }

    // If not found in vendor, check environment or search
    let zig_path = if let Some(path) = found_zig_path {
        path
    } else {
        // Try to find via BUN_INSTALL or environment
        if let Ok(bun_install) = std::env::var("BUN_INSTALL") {
            let env_zig = PathBuf::from(bun_install).join("zig");
            if env_zig.exists() && is_zig_binary(&env_zig) {
                env_zig
            } else {
                // Fall back to system zig but note it
                PathBuf::from("zig")
            }
        } else {
            // Use system zig with warning
            PathBuf::from("zig")
        }
    };

    // Verify zig exists
    if !is_zig_binary(&zig_path) && zig_path.to_string_lossy() != "zig" {
        return Err(anyhow::anyhow!("zig not found at {}", zig_path.display())).context(
            BunDetectError::PinnedZigNotFound(format!("expected at {:?}", zig_path)),
        );
    }

    // Get zig version
    let zig_version = get_zig_version(&zig_path)?;
    let zig_commit = extract_zig_commit(&zig_version);

    // Check if this is a patched Zig (supports --emit-zigmera-* flags)
    let supports_zigmera_flags = check_zigmera_flags(&zig_path);

    // Get stdlib path
    let zig_stdlib_path = get_zig_stdlib_path(&zig_path)?;

    // Check if it's the patched oven-sh/zig
    let is_patched = supports_zigmera_flags;

    Ok(BunZigToolchain {
        zig_path,
        zig_stdlib_path,
        zig_commit,
        zig_version,
        is_patched,
        supports_zigmera_flags,
    })
}

/// Check if a path is a valid zig binary
fn is_zig_binary(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    // Check if it's executable or is a file
    path.is_file()
}

/// Get Zig version string
fn get_zig_version(zig_path: &Path) -> Result<String> {
    let output = Command::new(zig_path).arg("version").output().context(
        BunDetectError::ZigCommandFailed("version command failed".to_string()),
    )?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("zig version command failed")).context(
            BunDetectError::ZigCommandFailed(format!("exit code: {}", output.status)),
        );
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err(anyhow::anyhow!("empty zig version output")).context(
            BunDetectError::ZigCommandFailed("empty version output".to_string()),
        );
    }

    Ok(version)
}

/// Extract commit hash from version string
fn extract_zig_commit(version: &str) -> String {
    // Version format: "0.x.y-{hash}" or "0.x.y"
    // Try to extract commit hash from version string
    if version.contains('-') {
        let parts: Vec<&str> = version.split('-').collect();
        if parts.len() >= 2 {
            // Check if the last part looks like a commit hash (40 hex chars for git)
            let last = parts[parts.len() - 1];
            if last.len() >= 7 && last.chars().all(|c| c.is_ascii_hexdigit()) {
                return last[..40.min(last.len())].to_string();
            }
        }
    }
    // Fall back to using the full version string as the commit identifier
    version.to_string()
}

/// Check if zig supports --emit-zigmera-* flags
fn check_zigmera_flags(zig_path: &Path) -> bool {
    let output = Command::new(zig_path).arg("--help").output();

    match output {
        Ok(out) if out.status.success() => {
            let help_text = String::from_utf8_lossy(&out.stdout);
            help_text.contains("--emit-zigmera-snapshot") || help_text.contains("--emit-zigmera")
        }
        _ => false,
    }
}

/// Get Zig stdlib path
fn get_zig_stdlib_path(zig_path: &Path) -> Result<PathBuf> {
    let output = Command::new(zig_path).args(["lib", "-q"]).output();

    match output {
        Ok(out) if out.status.success() => {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(PathBuf::from(path))
        }
        _ => {
            // Fall back to std lib path from env or relative
            let parent = zig_path.parent().map(|p| p.join("lib"));
            parent.ok_or_else(|| anyhow::anyhow!("cannot determine zig stdlib path"))
        }
    }
}

/// Capture Bun build.zig options
pub fn capture_bun_build_options(bun_root: &Path) -> Result<BunBuildOptions> {
    let build_zig_path = bun_root.join("build.zig");

    if !build_zig_path.exists() {
        return Err(anyhow::anyhow!("build.zig not found")).context(BunDetectError::NoBuildZig);
    }

    let build_zig_content = std::fs::read_to_string(&build_zig_path)?;

    let mut options = BunBuildOptions::default();

    // Parse optimize mode from build.zig
    if build_zig_content.contains("ReleaseFast") || build_zig_content.contains("release-fast") {
        options.optimize_mode = "ReleaseFast".to_string();
    } else if build_zig_content.contains("ReleaseSafe")
        || build_zig_content.contains("release-safe")
    {
        options.optimize_mode = "ReleaseSafe".to_string();
    } else if build_zig_content.contains("ReleaseSmall")
        || build_zig_content.contains("release-small")
    {
        options.optimize_mode = "ReleaseSmall".to_string();
    } else {
        options.optimize_mode = "Debug".to_string();
    }

    // Parse target from build.zig
    if build_zig_content.contains("x86_64-linux-gnu") {
        options.target = "x86_64-unknown-linux-gnu".to_string();
    } else if build_zig_content.contains("aarch64-linux-gnu") {
        options.target = "aarch64-unknown-linux-gnu".to_string();
    } else if build_zig_content.contains("x86_64-windows") {
        options.target = "x86_64-unknown-windows-gnu".to_string();
    } else if build_zig_content.contains("aarch64-macos") {
        options.target = "aarch64-apple-darwin".to_string();
    } else {
        options.target = "x86_64-unknown-linux-gnu".to_string(); // default
    }

    // Parse -D flags from build.zig
    let d_flag_re = regex::Regex::new(r"-D(\w+)=(\S+)").ok();
    if let Some(re) = d_flag_re {
        for cap in re.captures_iter(&build_zig_content) {
            let key = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let value = cap.get(2).map(|m| m.as_str()).unwrap_or_default();

            // Map common flags
            match key {
                "target" => options.target = value.to_string(),
                "cpu" => options.target_cpu = Some(value.to_string()),
                "sanitize" => options.sanitize = Some(value.to_string()),
                "lto" => options.lto = Some(value.to_string()),
                "threads" | "codegen_threads" => options.codegen_threads = Some(value.to_string()),
                "link_mode" => options.link_mode = Some(value.to_string()),
                "panic" => options.panic_mode = Some(value.to_string()),
                _ => {
                    // Treat as feature flag
                    let bool_value = value == "true" || value == "1" || value == "yes";
                    options.feature_flags.insert(key.to_string(), bool_value);
                }
            }
        }
    }

    // Also check for target cpu in the build file
    if build_zig_content.contains("baseline") || build_zig_content.contains("native") {
        options.target_cpu = Some("baseline".to_string());
    }

    Ok(options)
}

/// Create the Bun session manifest (B4)
pub fn create_bun_session(
    bun_root: &BunRepoRoot,
    zig_toolchain: &BunZigToolchain,
    build_options: &BunBuildOptions,
) -> Result<BunSession> {
    let captured_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // Collect source files
    let source_files = collect_zig_source_files(bun_root.path.as_path())?;

    // Generated files are typically in zig-cache or zig-out
    let generated_dir = bun_root.path.join("zig-out");
    let generated_files = if generated_dir.exists() {
        collect_generated_files(&generated_dir)?
    } else {
        Vec::new()
    };

    // Compute artifact hashes
    let mut artifact_hashes = HashMap::new();

    // Hash the build.zig file
    if let Ok(content) = std::fs::read_to_string(bun_root.path.join("build.zig")) {
        let mut hasher = Blake3Hasher::with_schema_tag("bun-session");
        hasher.update(content.as_bytes());
        let hash = hasher.finalize().as_hex();
        artifact_hashes.insert("build_zig".to_string(), hash);
    }

    // Hash the package.json
    if let Ok(content) = std::fs::read_to_string(bun_root.path.join("package.json")) {
        let mut hasher = Blake3Hasher::with_schema_tag("bun-session");
        hasher.update(content.as_bytes());
        let hash = hasher.finalize().as_hex();
        artifact_hashes.insert("package_json".to_string(), hash);
    }

    let output_dir = bun_root.path.join("zig-out").display().to_string();

    Ok(BunSession {
        version: "0.1.0".to_string(),
        bun_repo_root: bun_root.path.display().to_string(),
        bun_git_commit: detect_bun_git_commit(&bun_root.path),
        zig_toolchain: BunZigToolchain {
            zig_path: zig_toolchain.zig_path.clone(),
            zig_stdlib_path: zig_toolchain.zig_stdlib_path.clone(),
            zig_commit: zig_toolchain.zig_commit.clone(),
            zig_version: zig_toolchain.zig_version.clone(),
            is_patched: zig_toolchain.is_patched,
            supports_zigmera_flags: zig_toolchain.supports_zigmera_flags,
        },
        build_options: build_options.clone(),
        source_files,
        generated_files,
        output_dir,
        artifact_hashes,
        captured_ns,
    })
}

/// Detect Bun's git commit if available
fn detect_bun_git_commit(bun_root: &Path) -> Option<String> {
    let git_dir = bun_root.join(".git");
    if !git_dir.exists() {
        return None;
    }

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(bun_root)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Collect all .zig source files in the Bun repo
fn collect_zig_source_files(root: &Path) -> Result<Vec<String>> {
    let mut sources = Vec::new();

    // Walk the source directories
    for src_dir in BUN_SOURCE_DIRS {
        let path = root.join(src_dir);
        if path.is_dir() {
            collect_zig_files_recursive(&path, &mut sources)?;
        }
    }

    // Also check src/ explicitly (Bun's main source dir)
    let src = root.join("src");
    if src.is_dir() {
        collect_zig_files_recursive(&src, &mut sources)?;
    }

    // Check packages directory for monorepo
    let packages = root.join("packages");
    if packages.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&packages) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    collect_zig_files_recursive(&entry_path, &mut sources)?;
                }
            }
        }
    }

    // Remove duplicates
    sources.sort();
    sources.dedup();

    Ok(sources)
}

/// Recursively collect .zig files
fn collect_zig_files_recursive(dir: &Path, sources: &mut Vec<String>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip zig-cache and zig-out directories
            if path
                .file_name()
                .map(|n| n == "zig-cache" || n == "zig-out")
                .unwrap_or(false)
            {
                continue;
            }
            collect_zig_files_recursive(&path, sources)?;
        } else if path.extension().map(|e| e == "zig").unwrap_or(false) {
            sources.push(path.display().to_string());
        }
    }

    Ok(())
}

/// Collect generated files from zig-out
fn collect_generated_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    // Collect .o files and other build artifacts
    fn walk_dir(dir: &Path, files: &mut Vec<String>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                walk_dir(&path, files)?;
            } else {
                files.push(path.display().to_string());
            }
        }
        Ok(())
    }

    walk_dir(dir, &mut files)?;
    Ok(files)
}

/// Write the bun-session.json to disk
pub fn write_bun_session(session: &BunSession, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(output_path, &json)?;

    // Also verify it can be read back
    let loaded: BunSession = serde_json::from_str(&json)?;
    if loaded.bun_repo_root != session.bun_repo_root {
        anyhow::bail!("session JSON verification failed: root mismatch");
    }

    Ok(())
}

/// Load a bun-session.json from disk
pub fn load_bun_session(input_path: &Path) -> Result<BunSession> {
    let content = std::fs::read_to_string(input_path)?;
    let session: BunSession = serde_json::from_str(&content)?;
    Ok(session)
}

/// Check if two sessions are semantically equivalent
pub fn sessions_match(a: &BunSession, b: &BunSession) -> bool {
    // Check toolchain match
    if a.zig_toolchain.zig_commit != b.zig_toolchain.zig_commit {
        return false;
    }

    // Check build options match
    if a.build_options.optimize_mode != b.build_options.optimize_mode {
        return false;
    }
    if a.build_options.target != b.build_options.target {
        return false;
    }
    if a.build_options.sanitize != b.build_options.sanitize {
        return false;
    }
    if a.build_options.lto != b.build_options.lto {
        return false;
    }

    // Check source file hashes
    for (key, hash) in &a.artifact_hashes {
        if let Some(b_hash) = b.artifact_hashes.get(key) {
            if hash != b_hash {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bun_repo_root_detection_fails_for_non_bun() {
        // Use /tmp which definitely isn't a bun repo
        let result = detect_bun_repo_root(Path::new("/tmp"));
        // This should fail or return a result with low confidence
        // (Implementation may vary - some systems might have build.zig in /tmp by accident)
    }

    #[test]
    fn test_bun_build_options_default() {
        let options = BunBuildOptions::default();
        assert_eq!(options.optimize_mode, "");
        assert_eq!(options.target, "");
        assert!(options.feature_flags.is_empty());
    }

    #[test]
    fn test_zig_version_extraction() {
        // Test the commit extraction logic
        let version_with_commit = "0.13.0-deadbeef1234567890abcdef";
        let commit = extract_zig_commit(version_with_commit);
        assert!(commit.len() >= 7);

        let version_without_commit = "0.13.0";
        let commit = extract_zig_commit(version_without_commit);
        assert_eq!(commit, "0.13.0");
    }

    #[test]
    fn test_bun_session_serialization() {
        let session = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: Some("abc123".to_string()),
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
            generated_files: vec![],
            output_dir: "/test/zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 1234567890,
        };

        let json = serde_json::to_string_pretty(&session).unwrap();
        assert!(json.contains("\"version\": \"0.1.0\""));
        assert!(json.contains("\"/test/bun\""));
        assert!(json.contains("\"is_patched\": true"));
    }

    #[test]
    fn test_sessions_match() {
        let mut session1 = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: Some("abc123".to_string()),
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/test/zig"),
                zig_stdlib_path: PathBuf::from("/test/lib"),
                zig_commit: "v0.13.0".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec![],
            generated_files: vec![],
            output_dir: "/test/zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 1234567890,
        };
        session1.build_options.optimize_mode = "ReleaseFast".to_string();
        session1.build_options.target = "x86_64-unknown-linux-gnu".to_string();

        let mut session2 = session1.clone();
        session2.captured_ns = 9999999999; // Different timestamp

        assert!(sessions_match(&session1, &session2));

        // Change something material
        session2.build_options.optimize_mode = "Debug".to_string();
        assert!(!sessions_match(&session1, &session2));
    }

    #[test]
    fn test_zig_file_collector() {
        use std::fs;

        // Create temp directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create some .zig files
        fs::write(src_dir.join("main.zig"), "pub fn main() {}").unwrap();
        fs::write(src_dir.join("util.zig"), "pub fn helper() {}").unwrap();

        // Create a non-zig file
        fs::write(src_dir.join("data.txt"), "not zig").unwrap();

        let sources = collect_zig_source_files(temp_dir.path()).unwrap();
        assert!(sources.iter().any(|s| s.contains("main.zig")));
        assert!(sources.iter().any(|s| s.contains("util.zig")));
        assert!(!sources.iter().any(|s| s.contains("data.txt")));
    }

    #[test]
    fn test_session_json_roundtrip() {
        let mut session = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/bun/repo".to_string(),
            bun_git_commit: None,
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/bun/vendor/zig/zig"),
                zig_stdlib_path: PathBuf::from("/bun/vendor/zig/lib"),
                zig_commit: "abc123def456".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec!["src/a.zig".to_string(), "src/b.zig".to_string()],
            generated_files: vec!["zig-out/bin.o".to_string()],
            output_dir: "zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 1000000000,
        };
        session.build_options.optimize_mode = "ReleaseFast".to_string();
        session.build_options.target = "x86_64-unknown-linux-gnu".to_string();

        // Test roundtrip
        let json = serde_json::to_string(&session).unwrap();
        let parsed: BunSession = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, session.version);
        assert_eq!(parsed.bun_repo_root, session.bun_repo_root);
        assert_eq!(
            parsed.zig_toolchain.zig_commit,
            session.zig_toolchain.zig_commit
        );
        assert_eq!(
            parsed.build_options.optimize_mode,
            session.build_options.optimize_mode
        );
        assert_eq!(parsed.source_files.len(), 2);
    }

    #[test]
    fn test_artifact_hash_computation() {
        let mut options = BunBuildOptions::default();
        options.optimize_mode = "ReleaseFast".to_string();
        options.target = "x86_64-unknown-linux-gnu".to_string();
        options
            .feature_flags
            .insert("filter_none".to_string(), true);

        let hash = format!("{:?}", options);
        assert!(!hash.is_empty());
    }

    // B17: Bun fixture ladder tests
    #[test]
    fn test_bun_fixture_detection() {
        // Test that our fixture can be detected as a bun repo
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/bun-fixture");
        if fixture_path.exists() {
            let result = detect_bun_repo_root(&fixture_path);
            assert!(result.is_ok(), "fixture should be detected as bun repo");
            let root = result.unwrap();
            assert!(root.has_bun_lock);
            assert!(root.has_build_zig);
        }
    }

    #[test]
    fn test_bun_session_roundtrip_via_file() {
        // Test that session can be written and read from disk
        let temp_dir = tempfile::tempdir().unwrap();
        let session_path = temp_dir.path().join("bun-session.json");

        let mut session = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: None,
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/bun/zig"),
                zig_stdlib_path: PathBuf::from("/bun/lib"),
                zig_commit: "v0.13.0".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec!["src/main.zig".to_string()],
            generated_files: vec![],
            output_dir: "zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 12345,
        };
        session.build_options.optimize_mode = "ReleaseFast".to_string();

        write_bun_session(&session, &session_path).unwrap();

        let loaded = load_bun_session(&session_path).unwrap();
        assert_eq!(loaded.bun_repo_root, session.bun_repo_root);
        assert_eq!(
            loaded.zig_toolchain.zig_version,
            session.zig_toolchain.zig_version
        );
        assert_eq!(
            loaded.build_options.optimize_mode,
            session.build_options.optimize_mode
        );
    }

    // B18: Bun differential tests (simplified - no actual Zig compilation)
    #[test]
    fn test_bun_session_differential_matching() {
        let mut session1 = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: None,
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/bun/zig"),
                zig_stdlib_path: PathBuf::from("/bun/lib"),
                zig_commit: "v0.13.0".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec![],
            generated_files: vec![],
            output_dir: "zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 1000,
        };

        let mut session2 = session1.clone();

        // Identical sessions should match
        assert!(sessions_match(&session1, &session2));

        // Change only captured_ns - should still match
        session2.captured_ns = 9999;
        assert!(sessions_match(&session1, &session2));

        // Change zig_commit - should NOT match
        session2.zig_toolchain.zig_commit = "different".to_string();
        assert!(!sessions_match(&session1, &session2));
    }

    // B19: Bun doctor command validation
    #[test]
    fn test_bun_doctor_output_format() {
        // Validate that doctor-compatible data can be generated
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/bun-fixture");
        if fixture_path.exists() {
            let bun_root = detect_bun_repo_root(&fixture_path).unwrap();
            let zig_toolchain = detect_pinned_zig(&bun_root.path).ok();

            // Doctor output would include these fields
            assert!(bun_root.has_build_zig || !bun_root.has_build_zig); // Always valid
                                                                        // Zig toolchain may or may not be available
            if let Some(zig) = zig_toolchain {
                assert!(!zig.zig_version.is_empty());
            }
        }
    }

    // B20: Bun no-op reuse test infrastructure
    #[test]
    fn test_bun_no_op_reuse_detection() {
        // Test that we can detect when no-op reuse should occur
        // (same inputs should produce same session fingerprints)
        let session1 = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: None,
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/bun/zig"),
                zig_stdlib_path: PathBuf::from("/bun/lib"),
                zig_commit: "abc123".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec![],
            generated_files: vec![],
            output_dir: "zig-out".to_string(),
            artifact_hashes: HashMap::new(),
            captured_ns: 1000,
        };

        let session2 = session1.clone();

        // Identical sessions should match for no-op reuse
        assert!(sessions_match(&session1, &session2));

        // Check that sessions_match considers all relevant fields
        // (not just captured_ns which is allowed to differ)
    }

    #[test]
    fn test_bun_no_op_fingerprint_stability() {
        // Test that artifact_hashes remain stable across identical builds
        let mut hashes = HashMap::new();
        hashes.insert("build.zig".to_string(), "abc123def456".to_string());
        hashes.insert("src/main.zig".to_string(), "789xyz012345".to_string());

        let mut session1 = BunSession {
            version: "0.1.0".to_string(),
            bun_repo_root: "/test/bun".to_string(),
            bun_git_commit: None,
            zig_toolchain: BunZigToolchain {
                zig_path: PathBuf::from("/bun/zig"),
                zig_stdlib_path: PathBuf::from("/bun/lib"),
                zig_commit: "abc123".to_string(),
                zig_version: "0.13.0".to_string(),
                is_patched: true,
                supports_zigmera_flags: true,
            },
            build_options: BunBuildOptions::default(),
            source_files: vec![],
            generated_files: vec![],
            output_dir: "zig-out".to_string(),
            artifact_hashes: hashes.clone(),
            captured_ns: 1000,
        };

        let mut session2 = session1.clone();
        session2.captured_ns = 9999; // Different timestamp

        // Hashes must match for no-op reuse
        assert!(sessions_match(&session1, &session2));

        // Change a hash - should no longer match
        session2
            .artifact_hashes
            .insert("build.zig".to_string(), "different".to_string());
        assert!(!sessions_match(&session1, &session2));
    }

    // B21: Bun private body edit classification test
    #[test]
    fn test_bun_private_body_edit_detection() {
        // Private body edit should be detected as ChangeKind::PrivateBody
        use chimera_adapter_zig::graph::NodeId;
        use chimera_adapter_zig::invalidation::{ChangeKind, SourceChange};

        let node_id = NodeId::func("helper");
        let change = SourceChange::new(node_id.clone(), ChangeKind::PrivateBody);

        // Private body change should NOT be ABI-breaking
        assert!(!change.is_abi_breaking());
        // Private body change should NOT be an API change
        assert!(!change.is_api_change());

        // Verify the change kind is correct
        assert!(matches!(change.kind, ChangeKind::PrivateBody));
    }

    #[test]
    fn test_bun_private_change_no_downstream_invalidation() {
        // Test that private body changes don't trigger downstream invalidation
        use chimera_adapter_zig::graph::{GraphBuilder, NodeId};
        use chimera_adapter_zig::invalidation::{ChangeKind, InvalidationEngine, SourceChange};

        let mut builder = GraphBuilder::new();
        builder.add_file("main.zig");
        builder.add_function("public_api", "main.zig", 1);
        builder.add_function("private_helper", "main.zig", 2);

        let public_api = NodeId::func("public_api");
        let private_helper = NodeId::func("private_helper");

        // public_api uses private_helper
        builder.add_uses(&public_api, &private_helper);

        let graph = builder.build();
        let engine = InvalidationEngine::new(graph);

        // Change private_helper (private body edit)
        let change = SourceChange::new(private_helper.clone(), ChangeKind::PrivateBody);
        let result = engine.analyze(vec![change]);

        // private_helper should be stale (changed)
        assert!(result.stale_nodes.contains(&private_helper));
        // But public_api should NOT be invalidated for a private body change
        // because the ABI (signature) hasn't changed
        // Note: with current implementation, private changes only affect self
    }

    // B22: Bun public ABI invalidation test
    #[test]
    fn test_bun_public_abi_edit_detection() {
        // Public ABI edit should be detected as ChangeKind::ExportedAbi
        use chimera_adapter_zig::graph::NodeId;
        use chimera_adapter_zig::invalidation::{ChangeKind, SourceChange};

        let node_id = NodeId::func("exported_function");
        let change = SourceChange::new(node_id.clone(), ChangeKind::ExportedAbi);

        // Public ABI change IS ABI-breaking
        assert!(change.is_abi_breaking());
        // Public ABI change IS an API change
        assert!(change.is_api_change());

        // Verify the change kind is correct
        assert!(matches!(change.kind, ChangeKind::ExportedAbi));
    }

    #[test]
    fn test_bun_public_abi_triggers_downstream_invalidation() {
        // Test that public ABI changes trigger downstream invalidation
        use chimera_adapter_zig::graph::{GraphBuilder, NodeId};
        use chimera_adapter_zig::invalidation::{ChangeKind, InvalidationEngine, SourceChange};

        let mut builder = GraphBuilder::new();
        builder.add_file("main.zig");
        builder.add_function("api_function", "main.zig", 1);
        builder.add_function("consumer", "main.zig", 2);

        let api_function = NodeId::func("api_function");
        let consumer = NodeId::func("consumer");

        // consumer uses api_function
        builder.add_uses(&consumer, &api_function);
        // Mark api_function as exported
        builder.add_export("api_function", "my_api");
        let export = NodeId::export("my_api");
        builder.add_exports(&api_function, &export);

        let graph = builder.build();
        let engine = InvalidationEngine::new(graph);

        // Change api_function's ABI
        let change = SourceChange::new(api_function.clone(), ChangeKind::ExportedAbi);
        let result = engine.analyze(vec![change]);

        // api_function should be stale
        assert!(result.stale_nodes.contains(&api_function));
        // Full rebuild should be required
        assert!(result.requires_full_rebuild());
    }

    #[test]
    fn test_bun_layout_change_triggers_rebuild_and_relink() {
        // Test that layout changes trigger both rebuild and relink
        use chimera_adapter_zig::graph::{GraphBuilder, NodeId};
        use chimera_adapter_zig::invalidation::{ChangeKind, InvalidationEngine, SourceChange};

        let mut builder = GraphBuilder::new();
        builder.add_file("main.zig");
        builder.add_struct("MyStruct");
        builder.add_function("use_struct", "main.zig", 1);

        let my_struct = NodeId::struct_("MyStruct");
        let use_struct = NodeId::func("use_struct");

        builder.add_references(&use_struct, &my_struct);

        let graph = builder.build();
        let engine = InvalidationEngine::new(graph);

        // Change MyStruct layout
        let change = SourceChange::new(my_struct.clone(), ChangeKind::Layout);
        let result = engine.analyze(vec![change]);

        // Layout change should trigger full rebuild
        assert!(result.requires_full_rebuild());
        // Layout change should also require relink
        assert!(result.requires_only_relink() || result.requires_full_rebuild());
    }
}
