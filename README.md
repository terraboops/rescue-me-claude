# Rescue Me Claude

A bootable Linux rescue environment with [Claude Code](https://docs.anthropic.com/en/docs/claude-code) pre-installed for debugging and fixing Linux systems.

Boot from USB, get an AI-powered terminal ready to diagnose and fix your broken Arch (btw) or any other Linux.

## Features

- Bootable Arch Linux live environment
- Claude Code pre-installed and optionally pre-authenticated
- Lightweight i3 desktop with kitty terminal and Firefox
- Full rescue toolkit: filesystem tools, network diagnostics, hardware info, debuggers
- Cross-platform CLI tool (macOS + Linux) for building and flashing

## Quick Start

### Option 1: Download and Flash (easiest)

```bash
# Download the CLI tool from GitHub Releases, then:
rescue-me-claude flash --device /dev/sdX
```

### Option 2: Download ISO manually

Grab the latest ISO from [Releases](https://github.com/terraboops/rescue-me-claude/releases) and write it to USB:

```bash
rescue-me-claude burn rescue-me-claude-*.iso --device /dev/sdX
```

### Option 3: Build from source

```bash
# Build the CLI
cargo build --release

# Build the ISO (requires Docker or Podman)
./target/release/rescue-me-claude build

# Flash to USB
./target/release/rescue-me-claude burn out/*.iso
```

## Usage

1. Boot from the USB drive
2. Connect to network: `nmtui` (WiFi) or plug in ethernet
3. Authenticate Claude: `claude auth login`
4. Start debugging: `claude` or `rescue-claude`
5. For a desktop: `startx` (gets you i3 + terminal + browser)

### Pre-authentication

Bake your auth token into the ISO at build time:

```bash
rescue-me-claude build --claude-token "your-token"
```

Re-authenticate anytime on the live system with `claude-reauth`.

## What's Included

| Category | Tools |
|----------|-------|
| AI | Claude Code |
| Desktop | i3, kitty, Firefox, dmenu |
| Filesystem | btrfs-progs, e2fsprogs, xfsprogs, lvm2, cryptsetup, parted, ddrescue |
| Network | NetworkManager, openssh, nmap, tcpdump, bind |
| Debug | gdb, strace, ltrace, htop, lsof |
| Editors | neovim, vim, nano |
| System | tmux, git, arch-install-scripts, rsync |
| Hardware | smartmontools, nvme-cli, lshw, dmidecode |

## CLI Commands

```
rescue-me-claude build    Build ISO locally via Docker/Podman
rescue-me-claude download Download pre-built ISO from GitHub Releases
rescue-me-claude burn     Write ISO to USB drive
rescue-me-claude flash    Download + burn in one step
```

## Development

See [CLAUDE.md](CLAUDE.md) for development instructions.

## License

Apache-2.0
