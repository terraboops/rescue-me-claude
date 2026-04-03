mod build;
mod burn;
mod download;
mod utils;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "rescue-me-claude",
    about = "Build, download, and flash a bootable Linux rescue environment with Claude Code",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the rescue ISO locally using Docker or Podman
    Build {
        /// Claude API token to bake into the ISO for pre-authentication
        #[arg(long)]
        claude_token: Option<String>,

        /// Output directory for the ISO file
        #[arg(long, short, default_value = "out")]
        output: PathBuf,
    },

    /// Download the latest pre-built ISO from GitHub Releases
    Download {
        /// Output directory for the ISO file
        #[arg(long, short, default_value = "out")]
        output: PathBuf,
    },

    /// Burn an ISO file to a USB drive
    Burn {
        /// Path to the ISO file
        iso: PathBuf,

        /// Target device (e.g., /dev/sdb on Linux, /dev/disk2 on macOS).
        /// If not specified, an interactive device picker is shown.
        #[arg(long)]
        device: Option<String>,
    },

    /// Download (or build) and burn to USB in one step
    Flash {
        /// Claude API token to bake into the ISO for pre-authentication
        #[arg(long)]
        claude_token: Option<String>,

        /// Target device for USB burning
        #[arg(long)]
        device: Option<String>,

        /// Build locally instead of downloading from releases
        #[arg(long)]
        build: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            claude_token,
            output,
        } => build::run(claude_token.as_deref(), &output),

        Commands::Download { output } => download::run(&output),

        Commands::Burn { iso, device } => burn::run(&iso, device.as_deref()),

        Commands::Flash {
            claude_token,
            device,
            build: build_locally,
        } => flash(claude_token.as_deref(), device.as_deref(), build_locally),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn flash(
    claude_token: Option<&str>,
    device: Option<&str>,
    build_locally: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from("out");

    // Step 1: Get the ISO
    if build_locally {
        build::run(claude_token, &output)?;
    } else {
        download::run(&output)?;
    }

    let iso_path = find_iso(&output)?;

    // Step 2: Burn to USB
    burn::run(&iso_path, device)
}

fn find_iso(dir: &std::path::Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut isos: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "iso")
        })
        .collect();

    isos.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

    isos.into_iter()
        .next()
        .map(|e| e.path())
        .ok_or_else(|| "No ISO file found in output directory".into())
}
