#!/usr/bin/env bash
# shellcheck disable=SC2034

iso_name="rescue-me-claude"
iso_label="RESCUE_CLAUDE_$(date --utc +%Y%m)"
iso_publisher="Rescue Me Claude <https://github.com/terraboops/rescue-me-claude>"
iso_application="Rescue Me Claude - Bootable Claude Code Environment"
iso_version="$(date --utc +%Y.%m.%d)"
install_dir="arch"
buildmodes=('iso')
bootmodes=('bios.syslinux.mbr' 'bios.syslinux.eltorito'
            'uefi-ia32.grub.esp' 'uefi-x64.grub.esp'
            'uefi-ia32.grub.eltorito' 'uefi-x64.grub.eltorito')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '15')

file_permissions=(
  ["/usr/local/bin/rescue-claude"]="0:0:755"
  ["/usr/local/bin/claude-reauth"]="0:0:755"
  ["/root/.bashrc"]="0:0:644"
  ["/root/.xinitrc"]="0:0:755"
)
