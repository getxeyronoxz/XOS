#!/usr/bin/env bash
set -euo pipefail

# Install desktop configs into the archiso airootfs overlay.
# Called during Docker ISO build.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${REPO_ROOT}/base/airootfs/usr/share/xos/desktop-config/.config"
WALLPAPER_DEST="${REPO_ROOT}/base/airootfs/usr/share/xos/wallpapers"

mkdir -p "${DEST}/hypr" "${DEST}/waybar" "${DEST}/rofi" "${DEST}/mako" "${DEST}/foot"
mkdir -p "${REPO_ROOT}/base/airootfs/etc/regreet" "${REPO_ROOT}/base/airootfs/etc/xos"
mkdir -p "${WALLPAPER_DEST}"

cp "${REPO_ROOT}/desktop/hypr/hyprland.conf" "${DEST}/hypr/"
cp "${REPO_ROOT}/desktop/hypr/hyprlock.conf" "${DEST}/hypr/"
cp "${REPO_ROOT}/desktop/waybar/config" "${DEST}/waybar/"
cp "${REPO_ROOT}/desktop/waybar/style.css" "${DEST}/waybar/"
cp "${REPO_ROOT}/desktop/rofi/config.rasi" "${DEST}/rofi/"
cp "${REPO_ROOT}/desktop/rofi/xos.rasi" "${DEST}/rofi/"
cp "${REPO_ROOT}/desktop/mako/config" "${DEST}/mako/"
cp "${REPO_ROOT}/desktop/foot/foot.ini" "${DEST}/foot/"
cp "${REPO_ROOT}/desktop/regreet/config.toml" "${REPO_ROOT}/base/airootfs/etc/regreet/config.toml"
cp "${REPO_ROOT}/themes/xos/colors.toml" "${REPO_ROOT}/base/airootfs/etc/xos/colors.toml"

# Generate a simple gradient wallpaper if ImageMagick is available
if command -v convert &>/dev/null; then
    convert -size 1920x1080 "gradient:#1A1A2E-#0F3460" \
        "${WALLPAPER_DEST}/default.jpg"
else
    # Fallback: copy a minimal placeholder (1x1 PNG converted to path)
    printf '%s\n' "Wallpaper generated at build time" > "${WALLPAPER_DEST}/.placeholder"
fi

echo "Desktop configs installed to airootfs overlay."
