#!/bin/bash
# build.sh - Build the Rescue Me Claude ISO
# This script runs inside an Arch Linux container (or on an Arch host).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_DIR="/tmp/archiso-work"
OUTPUT_DIR="${SCRIPT_DIR}/out"
PROFILE_DIR="${SCRIPT_DIR}/profile"
CLAUDE_TOKEN="${CLAUDE_TOKEN:-}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --claude-token)
            CLAUDE_TOKEN="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--claude-token TOKEN] [--output DIR]"
            exit 1
            ;;
    esac
done

echo "=== Rescue Me Claude ISO Builder ==="
echo ""

# Install dependencies if running in container
if command -v pacman &>/dev/null; then
    echo "Installing build dependencies..."
    pacman -Sy --noconfirm archiso npm 2>/dev/null || true
fi

# Verify archiso is available
if ! command -v mkarchiso &>/dev/null; then
    echo "ERROR: mkarchiso not found. Install archiso package."
    exit 1
fi

# Create a working copy of the profile
WORK_PROFILE="/tmp/rescue-claude-profile"
rm -rf "$WORK_PROFILE"
cp -r "$PROFILE_DIR" "$WORK_PROFILE"

# Inject Claude token if provided
if [ -n "$CLAUDE_TOKEN" ]; then
    echo "Injecting Claude authentication token..."
    mkdir -p "$WORK_PROFILE/airootfs/root/.claude"
    cat > "$WORK_PROFILE/airootfs/root/.claude/.credentials.json" << CREDEOF
{
  "claudeAiOauth": {
    "token": "${CLAUDE_TOKEN}"
  }
}
CREDEOF
    chmod 600 "$WORK_PROFILE/airootfs/root/.claude/.credentials.json"

    # Add permissions to profiledef.sh
    echo '  ["/root/.claude/.credentials.json"]="0:0:600"' >> "$WORK_PROFILE/profiledef.sh"
fi

# Create the customize_airootfs script to install Claude Code
mkdir -p "$WORK_PROFILE/airootfs/root"
cat > "$WORK_PROFILE/airootfs/root/customize_airootfs.sh" << 'CUSTOMIZE_EOF'
#!/bin/bash
set -e

# Generate locale
echo "en_US.UTF-8 UTF-8" > /etc/locale.gen
locale-gen

# Enable services
systemctl enable NetworkManager
systemctl enable sshd
systemctl enable claude-setup.service

# Install Claude Code globally
echo "Installing Claude Code..."
npm install -g @anthropic-ai/claude-code || echo "WARNING: Failed to install Claude Code. It will be installed on first boot."

# Clean up
rm -f /root/customize_airootfs.sh
CUSTOMIZE_EOF
chmod 755 "$WORK_PROFILE/airootfs/root/customize_airootfs.sh"

# Clean previous build
rm -rf "$WORK_DIR"
mkdir -p "$OUTPUT_DIR"

echo "Building ISO..."
mkarchiso -v -w "$WORK_DIR" -o "$OUTPUT_DIR" "$WORK_PROFILE"

# Generate checksum
ISO_FILE=$(find "$OUTPUT_DIR" -maxdepth 1 -name '*.iso' -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
if [ -n "$ISO_FILE" ]; then
    echo "Generating checksum..."
    sha256sum "$ISO_FILE" > "${ISO_FILE}.sha256"
    echo ""
    echo "=== Build Complete ==="
    echo "ISO: $ISO_FILE"
    echo "SHA256: $(cat "${ISO_FILE}.sha256")"
    echo "Size: $(du -h "$ISO_FILE" | cut -f1)"
fi

# Clean up working copy (especially any credentials)
rm -rf "$WORK_PROFILE"
