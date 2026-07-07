#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ERRORS=0

pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1" >&2; ERRORS=$((ERRORS + 1)); }

check_file() {
    local path="$1"
    local desc="$2"
    if [[ -f "${path}" ]]; then
        pass "${desc}"
    else
        fail "Missing ${desc}: ${path}"
    fi
}

check_grep() {
    local file="$1"
    local pattern="$2"
    local desc="$3"
    if grep -qE "${pattern}" "${file}" 2>/dev/null; then
        pass "${desc}"
    else
        fail "${desc} — pattern '${pattern}' not found in ${file}"
    fi
}

echo "==> XOS Phase 1 & 2 validation"
echo

echo "--- Repository structure ---"
check_file "${REPO_ROOT}/base/profiledef.sh" "archiso profiledef.sh"
check_file "${REPO_ROOT}/base/packages.x86_64" "package list"
check_file "${REPO_ROOT}/base/pacman.conf" "pacman.conf"
check_file "${REPO_ROOT}/build/Dockerfile" "Dockerfile"
check_file "${REPO_ROOT}/build/build.sh" "build.sh"
check_file "${REPO_ROOT}/build/build-iso.sh" "build-iso.sh"
check_file "${REPO_ROOT}/build/install-desktop-configs.sh" "install-desktop-configs.sh"

echo
echo "--- Phase 1 desktop configs ---"
check_file "${REPO_ROOT}/desktop/hypr/hyprland.conf" "Hyprland config"
check_file "${REPO_ROOT}/desktop/hypr/hyprlock.conf" "Hyprlock config"
check_file "${REPO_ROOT}/desktop/waybar/config" "Waybar config"
check_file "${REPO_ROOT}/desktop/waybar/style.css" "Waybar style"
check_file "${REPO_ROOT}/desktop/rofi/config.rasi" "Rofi config"
check_file "${REPO_ROOT}/desktop/mako/config" "Mako config"
check_file "${REPO_ROOT}/desktop/foot/foot.ini" "Foot terminal config"
check_file "${REPO_ROOT}/desktop/regreet/config.toml" "ReGreet config"

echo
echo "--- Phase 1 package list ---"
for pkg in hyprland waybar foot firefox networkmanager greetd greetd-regreet rofi-wayland mako hyprlock swww linux-zen; do
    check_grep "${REPO_ROOT}/base/packages.x86_64" "^${pkg}$" "Package: ${pkg}"
done

echo
echo "--- Hyprland keybindings (spec shortcuts) ---"
HYPR="${REPO_ROOT}/desktop/hypr/hyprland.conf"
check_grep "${HYPR}" "SPACE, exec, rofi" "Super+Space app launcher"
check_grep "${HYPR}" "T, exec, foot" "Super+T terminal"
check_grep "${HYPR}" "L, exec, hyprlock" "Super+L lock screen"
check_grep "${HYPR}" "1, workspace, 1" "Super+1 workspace"
check_grep "${HYPR}" "F, togglefloating" "Super+F toggle float"
check_grep "${HYPR}" "M, fullscreen" "Super+M fullscreen"

echo
echo "--- XOS theme colors ---"
check_grep "${REPO_ROOT}/themes/xos/colors.toml" '#5294E2' "Accent color #5294E2"
check_grep "${REPO_ROOT}/themes/xos/colors.toml" '#1A1A2E' "Background color #1A1A2E"
check_grep "${REPO_ROOT}/desktop/waybar/style.css" '#0F3460' "Panel color in Waybar"

echo
echo "--- Greetd + login ---"
check_file "${REPO_ROOT}/base/airootfs/etc/greetd/config.toml" "greetd config"
check_grep "${REPO_ROOT}/base/airootfs/etc/greetd/config.toml" "regreet" "ReGreet greeter"
check_grep "${REPO_ROOT}/base/airootfs/etc/greetd/environments" "Hyprland" "Hyprland session"

echo
echo "--- File Manager (Phase 2) ---"
check_file "${REPO_ROOT}/apps/file-manager/Cargo.toml" "file-manager Cargo.toml"
check_file "${REPO_ROOT}/apps/file-manager/src/main.rs" "file-manager main.rs"
check_file "${REPO_ROOT}/apps/file-manager/src/window.rs" "file-manager window.rs"
check_file "${REPO_ROOT}/apps/file-manager/src/operations.rs" "file-manager operations.rs"
check_file "${REPO_ROOT}/apps/file-manager/src/sidebar.rs" "file-manager sidebar.rs"
check_file "${REPO_ROOT}/apps/file-manager/src/breadcrumb.rs" "file-manager breadcrumb.rs"
check_grep "${REPO_ROOT}/apps/file-manager/src/operations.rs" "copy_into_directory" "File manager copy"
check_grep "${REPO_ROOT}/apps/file-manager/src/operations.rs" "move_into_directory" "File manager move"
check_grep "${REPO_ROOT}/apps/file-manager/src/operations.rs" "trash_file" "File manager delete/trash"
check_grep "${REPO_ROOT}/apps/file-manager/src/sidebar.rs" "Bookmarks" "File manager sidebar bookmarks"
check_grep "${REPO_ROOT}/apps/file-manager/src/breadcrumb.rs" "update_breadcrumb" "File manager breadcrumb bar"
check_file "${REPO_ROOT}/apps/file-manager/src/preview.rs" "file-manager preview module"
check_file "${REPO_ROOT}/apps/file-manager/src/search.rs" "file-manager search module"
check_grep "${REPO_ROOT}/apps/file-manager/src/preview.rs" "PreviewPanel" "File manager preview panel"
check_grep "${REPO_ROOT}/apps/file-manager/src/search.rs" "SearchFilters" "File manager search filters"
check_grep "${REPO_ROOT}/apps/file-manager/src/search.rs" "search_directory" "File manager recursive search"
check_grep "${REPO_ROOT}/desktop/hypr/hyprland.conf" "xos-file-manager" "Super+E binds file manager"
check_grep "${REPO_ROOT}/desktop/hypr/hyprland.conf" "xos-control-center" "Super+I binds control center"

echo
echo "--- Control Center (Phase 2) ---"
check_file "${REPO_ROOT}/apps/control-center/Cargo.toml" "control-center Cargo.toml"
check_file "${REPO_ROOT}/apps/control-center/src/main.rs" "control-center main.rs"
check_file "${REPO_ROOT}/apps/control-center/src/window.rs" "control-center window.rs"
check_file "${REPO_ROOT}/apps/control-center/src/pages/appearance.rs" "control-center appearance page"
check_file "${REPO_ROOT}/apps/control-center/src/pages/display.rs" "control-center display page"
check_file "${REPO_ROOT}/apps/control-center/src/pages/sound.rs" "control-center sound page"
check_file "${REPO_ROOT}/apps/control-center/src/pages/network.rs" "control-center network page"
check_grep "${REPO_ROOT}/apps/control-center/src/pages/appearance.rs" "Accent color" "Appearance accent setting"
check_grep "${REPO_ROOT}/apps/control-center/src/pages/network.rs" "NetworkManager" "Network section references NetworkManager"

echo
echo "--- Screenshot tool (Phase 2) ---"
check_file "${REPO_ROOT}/apps/screenshot/Cargo.toml" "screenshot Cargo.toml"
check_file "${REPO_ROOT}/apps/screenshot/src/capture.rs" "screenshot capture module"
check_grep "${REPO_ROOT}/apps/screenshot/src/capture.rs" "CaptureMode" "Screenshot capture modes"
check_grep "${REPO_ROOT}/apps/screenshot/src/capture.rs" "slurp" "Screenshot uses slurp"
check_grep "${REPO_ROOT}/apps/screenshot/src/capture.rs" "grim" "Screenshot uses grim"
check_grep "${REPO_ROOT}/desktop/hypr/hyprland.conf" "xos-screenshot" "Print key binds screenshot tool"

echo
echo "--- Notes app (Phase 2) ---"
check_file "${REPO_ROOT}/apps/notes/Cargo.toml" "notes Cargo.toml"
check_file "${REPO_ROOT}/apps/notes/src/storage.rs" "notes storage module"
check_file "${REPO_ROOT}/apps/notes/src/window.rs" "notes window"
check_grep "${REPO_ROOT}/apps/notes/src/storage.rs" "create_note" "Notes create"
check_grep "${REPO_ROOT}/apps/notes/src/storage.rs" "write_note" "Notes save"
check_grep "${REPO_ROOT}/apps/notes/src/storage.rs" "delete_note" "Notes delete"

echo
echo "--- Archive tool (Phase 2) ---"
check_file "${REPO_ROOT}/apps/archive-tool/Cargo.toml" "archive-tool Cargo.toml"
check_file "${REPO_ROOT}/apps/archive-tool/src/archive.rs" "archive-tool archive module"
check_file "${REPO_ROOT}/apps/archive-tool/src/window.rs" "archive-tool window"
check_grep "${REPO_ROOT}/build/apps-build.sh" "archive-tool" "apps-build includes archive-tool"

echo
echo "--- System Monitor (Phase 2) ---"
check_file "${REPO_ROOT}/apps/system-monitor/Cargo.toml" "system-monitor Cargo.toml"
check_file "${REPO_ROOT}/apps/system-monitor/src/window.rs" "system-monitor window"
check_file "${REPO_ROOT}/apps/system-monitor/src/overview.rs" "system-monitor overview page"
check_file "${REPO_ROOT}/apps/system-monitor/src/processes.rs" "system-monitor processes page"
check_file "${REPO_ROOT}/apps/system-monitor/src/sysdata.rs" "system-monitor sysdata module"
check_grep "${REPO_ROOT}/build/apps-build.sh" "system-monitor" "apps-build includes system-monitor"

echo
echo "--- Quick Settings (Phase 2) ---"
check_file "${REPO_ROOT}/apps/quick-settings/Cargo.toml" "quick-settings Cargo.toml"
check_file "${REPO_ROOT}/apps/quick-settings/src/window.rs" "quick-settings window"
check_file "${REPO_ROOT}/apps/quick-settings/src/toggles.rs" "quick-settings toggles module"
check_file "${REPO_ROOT}/apps/quick-settings/src/sliders.rs" "quick-settings sliders module"
check_file "${REPO_ROOT}/apps/quick-settings/src/power.rs" "quick-settings power module"
check_grep "${REPO_ROOT}/build/apps-build.sh" "quick-settings" "apps-build includes quick-settings"

echo
echo "--- Notification Center (Phase 2) ---"
check_file "${REPO_ROOT}/apps/notification-center/Cargo.toml" "notification-center Cargo.toml"
check_file "${REPO_ROOT}/apps/notification-center/src/window.rs" "notification-center window"
check_file "${REPO_ROOT}/apps/notification-center/src/notification.rs" "notification-center notification module"
check_file "${REPO_ROOT}/apps/notification-center/src/notification_row.rs" "notification-center row widget"
check_grep "${REPO_ROOT}/build/apps-build.sh" "notification-center" "apps-build includes notification-center"

echo
echo "--- Build pipeline ---"
check_file "${REPO_ROOT}/build/apps-build.sh" "apps-build.sh"
check_grep "${REPO_ROOT}/build/build-iso.sh" "apps-build.sh" "ISO build runs apps-build.sh"
check_grep "${REPO_ROOT}/build/apps-build.sh" "file-manager" "apps-build includes file-manager"
check_grep "${REPO_ROOT}/build/apps-build.sh" "control-center" "apps-build includes control-center"

echo
echo "--- Shell scripts: set -euo pipefail ---"
for script in build/build.sh build/build-iso.sh build/install-desktop-configs.sh build/apps-build.sh build/test-qemu.sh build/validate.sh; do
    check_grep "${REPO_ROOT}/${script}" 'set -euo pipefail' "${script} has strict mode"
done

echo
if [[ ${ERRORS} -eq 0 ]]; then
    echo "==> All validation checks passed."
    exit 0
else
    echo "==> ${ERRORS} validation check(s) failed." >&2
    exit 1
fi
