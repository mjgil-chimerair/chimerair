//! ZigMera CLI - General-purpose per-file incremental Zig builder

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

use zigmera_cli::{
    build_project, clean_cache, generate_manifest, init_project, save_manifest, show_status,
    GenConfig,
};

#[derive(Parser)]
#[command(name = "zigmera")]
#[command(about = "Per-file incremental Zig builder")]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Cache directory (default: .zigmera/cache)
    #[arg(short, long)]
    cache_dir: Option<PathBuf>,

    /// Real zig binary path
    #[arg(short, long)]
    zig: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a project for per-file incremental builds
    Init {
        /// Source directory (default: src)
        #[arg(default_value = "src")]
        source_dir: PathBuf,

        /// Output directory (default: zig-out)
        #[arg(default_value = "zig-out")]
        output_dir: PathBuf,
    },

    /// Build with per-file incremental compilation
    Build {
        /// Source directory
        #[arg(short, long)]
        source_dir: Option<PathBuf>,

        /// Build directory
        #[arg(short, long)]
        build_dir: Option<PathBuf>,
    },

    /// Show cache status and changed files
    Status,

    /// Clean build artifacts and cache
    Clean,

    /// Generate manifest only (don't build)
    GenManifest {
        /// Source directory
        #[arg(default_value = "src")]
        source_dir: PathBuf,

        /// Output directory
        #[arg(default_value = "zig-out")]
        output_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();
    let cache_dir = args
        .cache_dir
        .unwrap_or_else(|| PathBuf::from(".zigmera/cache"));

    match args.command {
        Commands::Init {
            source_dir,
            output_dir,
        } => {
            let config = GenConfig {
                source_dir: source_dir.display().to_string(),
                output_dir: output_dir.display().to_string(),
                target: "x86_64-linux-gnu".to_string(),
                optimize: "ReleaseFast".to_string(),
            };

            match init_project(&config, &cache_dir) {
                Ok(_) => println!("Project initialized! Run 'zigmera build' to build."),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return ExitCode::from(1);
                }
            }
        }

        Commands::Build {
            source_dir,
            build_dir,
        } => {
            let source = source_dir.unwrap_or_else(|| PathBuf::from("src"));
            let build = build_dir.unwrap_or_else(|| PathBuf::from("zig-out"));

            let zig_path = args
                .zig
                .or_else(|| std::env::var("ZIGMERA_REAL_ZIG").map(PathBuf::from).ok())
                .unwrap_or_else(|| PathBuf::from("zig"));

            match build_project(&zig_path, &cache_dir, &build) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return ExitCode::from(1);
                }
            }
        }

        Commands::Status => {
            if let Err(e) = show_status(&cache_dir) {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }
        }

        Commands::Clean => {
            if let Err(e) = clean_cache(&cache_dir) {
                eprintln!("Error: {}", e);
                return ExitCode::from(1);
            }
            println!("Cache cleaned!");
        }

        Commands::GenManifest {
            source_dir,
            output_dir,
        } => {
            let config = GenConfig {
                source_dir: source_dir.display().to_string(),
                output_dir: output_dir.display().to_string(),
                target: "x86_64-linux-gnu".to_string(),
                optimize: "ReleaseFast".to_string(),
            };

            match generate_manifest(&config) {
                Ok(manifest) => {
                    let manifest_path = cache_dir.join("manifest.json");
                    match save_manifest(&manifest, &manifest_path) {
                        Ok(_) => {
                            println!("Manifest generated: {} targets", manifest.targets.len());
                            println!("  Path: {}", manifest_path.display());
                        }
                        Err(e) => {
                            eprintln!("Failed to save manifest: {}", e);
                            return ExitCode::from(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to generate manifest: {}", e);
                    return ExitCode::from(1);
                }
            }
        }
    }

    ExitCode::SUCCESS
}
