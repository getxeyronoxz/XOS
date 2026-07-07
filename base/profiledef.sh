#!/usr/bin/env bash
# shellcheck disable=SC2034

iso_name="xos"
iso_label="XOS_$(date +%Y%m)"
iso_publisher="Xeyronox <https://github.com/getxeyronoxz/XOS>"
iso_application="XOS Live Environment"
iso_version="$(date +%Y.%m).0"
install_dir="xos"
buildmodes=('iso')
bootmodes=('bios.syslinux' 'uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/root"]="0:0:750"
  ["/etc/greetd/config.toml"]="0:0:644"
  ["/etc/xos/version"]="0:0:644"
  ["/usr/local/bin/xos-first-login"]="0:0:755"
)
