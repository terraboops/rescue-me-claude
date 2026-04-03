# Rescue Me Claude - Development Guide

## Project Structure

- `src/` - Rust CLI tool for building, downloading, and flashing the ISO
- `profile/` - archiso profile (Arch Linux ISO configuration)
- `build.sh` - Inner build script that runs inside an Arch Linux container
- `.github/workflows/` - CI/CD pipelines

## Building

### Prerequisites
- Rust toolchain (`rustup`)
- Docker or Podman (for ISO building)

### Build the CLI tool
```bash
cargo build --release
```

### Build the ISO locally
```bash
# Via the Rust CLI (recommended)
cargo run -- build

# Or directly with Docker
docker run --rm --privileged -v .:/work:ro -v ./out:/output archlinux:latest /work/build.sh

# With pre-baked auth token
cargo run -- build --claude-token "your-token"
```

### Download pre-built ISO
```bash
cargo run -- download
```

### Flash to USB
```bash
cargo run -- flash                    # Download + interactive device selection
cargo run -- burn out/*.iso           # Burn existing ISO interactively
cargo run -- burn out/*.iso --device /dev/sdb  # Burn to specific device
```

## Testing

```bash
# Rust tests
cargo test

# Lint
cargo clippy

# Test ISO in QEMU (after building)
qemu-system-x86_64 -cdrom out/rescue-me-claude-*.iso -m 4G -enable-kvm
```

## Release Process

1. Tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
2. Push: `git push origin v0.1.0`
3. CI builds ISO + CLI binaries and creates a GitHub Release
