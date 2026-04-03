use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;

const REPO: &str = "terraboops/rescue-me-claude";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub fn run(output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching latest release from {REPO}...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("rescue-me-claude")
        .build()?;

    let release: Release = client
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .send()?
        .error_for_status()?
        .json()?;

    println!("Latest release: {}", release.tag_name);

    // Find the ISO asset
    let iso_asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".iso"))
        .ok_or("No ISO file found in the latest release")?;

    // Find optional checksum asset
    let checksum_asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".sha256") || a.name.ends_with(".sha256sum"));

    fs::create_dir_all(output)?;
    let iso_path = output.join(&iso_asset.name);

    // Download ISO with progress bar
    println!("Downloading {}...", iso_asset.name);
    download_with_progress(&client, &iso_asset.browser_download_url, &iso_path, iso_asset.size)?;

    // Verify checksum if available
    if let Some(checksum) = checksum_asset {
        println!("Verifying checksum...");
        let checksum_path = output.join(&checksum.name);
        download_with_progress(&client, &checksum.browser_download_url, &checksum_path, checksum.size)?;
        verify_checksum(&iso_path, &checksum_path)?;
        println!("Checksum verified!");
    }

    println!("Downloaded to: {}", iso_path.display());
    Ok(())
}

fn download_with_progress(
    client: &reqwest::blocking::Client,
    url: &str,
    dest: &Path,
    size: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = client.get(url).send()?.error_for_status()?;

    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut file = fs::File::create(dest)?;
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];

    loop {
        use std::io::Read;
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    Ok(())
}

fn verify_checksum(
    iso_path: &Path,
    checksum_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    // Read expected checksum
    let checksum_content = fs::read_to_string(checksum_path)?;
    let expected = checksum_content
        .split_whitespace()
        .next()
        .ok_or("Invalid checksum file format")?
        .to_lowercase();

    // Calculate actual checksum
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(iso_path)?;
    let mut buffer = [0u8; 65536];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let actual = hex::encode(hasher.finalize());

    if actual != expected {
        return Err(format!("Checksum mismatch!\n  Expected: {expected}\n  Got:      {actual}").into());
    }

    Ok(())
}
