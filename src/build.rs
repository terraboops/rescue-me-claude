use std::path::Path;
use std::process::Command;

pub fn run(claude_token: Option<&str>, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let container_runtime = find_container_runtime()?;
    println!("Using container runtime: {container_runtime}");

    // Ensure output directory exists
    std::fs::create_dir_all(output)?;

    let work_dir = std::env::current_dir()?;
    let output_abs = std::fs::canonicalize(output).unwrap_or_else(|_| work_dir.join(output));

    // Pull the Arch Linux image
    println!("Pulling archlinux:latest...");
    let status = Command::new(&container_runtime)
        .args(["pull", "archlinux:latest"])
        .status()?;
    if !status.success() {
        return Err("Failed to pull archlinux:latest image".into());
    }

    // Build the container command
    let mut args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "--privileged".to_string(),
        "-v".to_string(),
        format!("{}:/work:ro", work_dir.display()),
        "-v".to_string(),
        format!("{}:/output", output_abs.display()),
    ];

    // Pass Claude token as environment variable if provided
    if let Some(token) = claude_token {
        args.push("-e".to_string());
        args.push(format!("CLAUDE_TOKEN={token}"));
    }

    args.push("archlinux:latest".to_string());
    args.push("/bin/bash".to_string());
    args.push("/work/build.sh".to_string());

    println!("Building ISO (this may take a while)...");
    let status = Command::new(&container_runtime).args(&args).status()?;

    if !status.success() {
        return Err("ISO build failed".into());
    }

    println!("ISO built successfully! Output in: {}", output_abs.display());
    Ok(())
}

fn find_container_runtime() -> Result<String, Box<dyn std::error::Error>> {
    if which::which("docker").is_ok() {
        Ok("docker".to_string())
    } else if which::which("podman").is_ok() {
        Ok("podman".to_string())
    } else {
        Err("Neither Docker nor Podman found. Please install one of them to build the ISO.".into())
    }
}
