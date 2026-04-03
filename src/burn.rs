use dialoguer::{Confirm, Select};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

pub fn run(iso: &Path, device: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if !iso.exists() {
        return Err(format!("ISO file not found: {}", iso.display()).into());
    }

    let target = match device {
        Some(dev) => dev.to_string(),
        None => pick_device()?,
    };

    // Safety confirmation
    let size_mb = iso.metadata()?.len() / 1_048_576;
    println!("\nAbout to write:");
    println!("  ISO:    {} ({size_mb} MB)", iso.display());
    println!("  Target: {target}");
    println!("\n  WARNING: ALL DATA ON {target} WILL BE ERASED!");

    if !Confirm::new()
        .with_prompt("Are you sure you want to continue?")
        .default(false)
        .interact()?
    {
        println!("Aborted.");
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        burn_macos(iso, &target)?;
    } else {
        burn_linux(iso, &target)?;
    }

    println!("\nDone! You can now boot from {target}.");
    Ok(())
}

fn pick_device() -> Result<String, Box<dyn std::error::Error>> {
    let devices = list_devices()?;

    if devices.is_empty() {
        return Err("No removable devices found. Insert a USB drive and try again.".into());
    }

    let labels: Vec<String> = devices
        .iter()
        .map(|d| format!("{} - {} ({})", d.name, d.model, d.size))
        .collect();

    let selection = Select::new()
        .with_prompt("Select target USB device")
        .items(&labels)
        .interact()?;

    Ok(devices[selection].name.clone())
}

struct BlockDevice {
    name: String,
    size: String,
    model: String,
}

fn list_devices() -> Result<Vec<BlockDevice>, Box<dyn std::error::Error>> {
    if cfg!(target_os = "macos") {
        list_devices_macos()
    } else {
        list_devices_linux()
    }
}

fn list_devices_linux() -> Result<Vec<BlockDevice>, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<LsblkDevice>,
    }

    #[derive(Deserialize)]
    struct LsblkDevice {
        name: String,
        size: Option<String>,
        #[serde(rename = "type")]
        dtype: String,
        rm: Option<bool>,
        model: Option<String>,
    }

    let output = Command::new("lsblk")
        .args(["-J", "-o", "NAME,SIZE,TYPE,RM,MODEL"])
        .output()?;

    let parsed: LsblkOutput = serde_json::from_slice(&output.stdout)?;

    Ok(parsed
        .blockdevices
        .into_iter()
        .filter(|d| d.dtype == "disk" && d.rm == Some(true))
        .map(|d| BlockDevice {
            name: format!("/dev/{}", d.name),
            size: d.size.unwrap_or_default(),
            model: d.model.unwrap_or_else(|| "Unknown".to_string()),
        })
        .collect())
}

fn list_devices_macos() -> Result<Vec<BlockDevice>, Box<dyn std::error::Error>> {
    let output = Command::new("diskutil")
        .args(["list", "external"])
        .output()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in text.lines() {
        if line.starts_with("/dev/disk") {
            let name = line.split_whitespace().next().unwrap_or("").to_string();
            if !name.is_empty() {
                // Get size info
                let info = Command::new("diskutil")
                    .args(["info", &name])
                    .output()?;
                let info_text = String::from_utf8_lossy(&info.stdout);

                let size = info_text
                    .lines()
                    .find(|l| l.contains("Disk Size:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let model = info_text
                    .lines()
                    .find(|l| l.contains("Device / Media Name:"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                devices.push(BlockDevice { name, size, model });
            }
        }
    }

    Ok(devices)
}

fn burn_linux(iso: &Path, device: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Unmount any mounted partitions
    println!("Unmounting partitions on {device}...");
    let _ = Command::new("umount")
        .args([&format!("{device}*")])
        .status();

    println!("Writing ISO to {device}...");
    let status = Command::new("dd")
        .args([
            &format!("if={}", iso.display()),
            &format!("of={device}"),
            "bs=4M",
            "status=progress",
            "conv=fsync",
        ])
        .status()?;

    if !status.success() {
        return Err("dd failed. You may need to run with sudo.".into());
    }

    println!("Syncing...");
    Command::new("sync").status()?;

    Ok(())
}

fn burn_macos(iso: &Path, device: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Use rdisk for faster raw access
    let raw_device = device.replace("/dev/disk", "/dev/rdisk");

    println!("Unmounting {device}...");
    let status = Command::new("diskutil")
        .args(["unmountDisk", device])
        .status()?;
    if !status.success() {
        return Err("Failed to unmount disk".into());
    }

    println!("Writing ISO to {raw_device}...");
    let status = Command::new("sudo")
        .args([
            "dd",
            &format!("if={}", iso.display()),
            &format!("of={raw_device}"),
            "bs=4m",
        ])
        .status()?;

    if !status.success() {
        return Err("dd failed".into());
    }

    println!("Ejecting {device}...");
    Command::new("diskutil")
        .args(["eject", device])
        .status()?;

    Ok(())
}
