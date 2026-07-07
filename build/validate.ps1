#!/usr/bin/env pwsh
# XOS Phase 1 & 2 validation (Windows-compatible)
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Errors = 0

function Pass($msg) { Write-Host "  PASS: $msg" -ForegroundColor Green }
function Fail($msg) { Write-Host "  FAIL: $msg" -ForegroundColor Red; $script:Errors++ }

function Test-FileExists($path, $desc) {
    if (Test-Path $path) { Pass $desc } else { Fail "Missing ${desc}: $path" }
}

function Test-FileContains($path, $pattern, $desc) {
    if ((Test-Path $path) -and (Select-String -Path $path -Pattern $pattern -Quiet)) {
        Pass $desc
    } else {
        Fail "${desc} - pattern not found in $path"
    }
}

Write-Host "==> XOS Phase 1 & 2 validation (PowerShell)" -ForegroundColor Cyan
Write-Host ""

Write-Host "--- Repository structure ---"
Test-FileExists "$RepoRoot\base\profiledef.sh" "archiso profiledef.sh"
Test-FileExists "$RepoRoot\base\packages.x86_64" "package list"
Test-FileExists "$RepoRoot\build\Dockerfile" "Dockerfile"
Test-FileExists "$RepoRoot\build\build.sh" "build.sh"

Write-Host ""
Write-Host "--- Phase 1 desktop configs ---"
Test-FileExists "$RepoRoot\desktop\hypr\hyprland.conf" "Hyprland config"
Test-FileExists "$RepoRoot\desktop\waybar\config" "Waybar config"
Test-FileExists "$RepoRoot\desktop\rofi\config.rasi" "Rofi config"
Test-FileExists "$RepoRoot\desktop\mako\config" "Mako config"
Test-FileExists "$RepoRoot\desktop\hypr\hyprlock.conf" "Hyprlock config"
Test-FileExists "$RepoRoot\apps\file-manager\src\sidebar.rs" "file-manager sidebar.rs"
Test-FileExists "$RepoRoot\apps\file-manager\src\breadcrumb.rs" "file-manager breadcrumb.rs"

Write-Host ""
Write-Host "--- File manager operations ---"
$ops = "$RepoRoot\apps\file-manager\src\operations.rs"
Test-FileContains $ops "copy_into_directory" "File manager copy"
Test-FileContains $ops "move_into_directory" "File manager move"
Test-FileContains $ops "trash_file" "File manager delete/trash"
$preview = "$RepoRoot\apps\file-manager\src\preview.rs"
$search = "$RepoRoot\apps\file-manager\src\search.rs"
Test-FileExists $preview "file-manager preview module"
Test-FileExists $search "file-manager search module"
Test-FileContains $search "SearchFilters" "File manager search filters"

Write-Host ""
Write-Host "--- Build pipeline ---"
Test-FileExists "$RepoRoot\build\apps-build.sh" "apps-build.sh"
Test-FileContains "$RepoRoot\build\build-iso.sh" "apps-build.sh" "ISO build runs apps-build"

Write-Host ""
Write-Host "--- Control Center (Phase 2) ---"
$cc = "$RepoRoot\apps\control-center"
Test-FileExists "$cc\Cargo.toml" "control-center Cargo.toml"
Test-FileExists "$cc\src\window.rs" "control-center window.rs"
Test-FileExists "$cc\src\pages\appearance.rs" "control-center appearance page"
Test-FileExists "$cc\src\pages\display.rs" "control-center display page"
Test-FileExists "$cc\src\pages\sound.rs" "control-center sound page"
Test-FileExists "$cc\src\pages\network.rs" "control-center network page"
$hypr = "$RepoRoot\desktop\hypr\hyprland.conf"
Test-FileContains $hypr "xos-control-center" "Super+I binds control center"

Write-Host ""
Write-Host "--- Screenshot tool (Phase 2) ---"
$shot = "$RepoRoot\apps\screenshot"
Test-FileExists "$shot\Cargo.toml" "screenshot Cargo.toml"
Test-FileExists "$shot\src\capture.rs" "screenshot capture module"
Test-FileContains "$shot\src\capture.rs" "CaptureMode" "Screenshot capture modes"
Test-FileContains $hypr "xos-screenshot" "Print key binds screenshot tool"

Write-Host ""
Write-Host "--- Notes app (Phase 2) ---"
$notes = "$RepoRoot\apps\notes"
Test-FileExists "$notes\Cargo.toml" "notes Cargo.toml"
Test-FileExists "$notes\src\storage.rs" "notes storage module"
Test-FileContains "$notes\src\storage.rs" "create_note" "Notes create"
Test-FileContains "$notes\src\storage.rs" "write_note" "Notes save"

Write-Host ""
Write-Host "--- Archive tool (Phase 2) ---"
$archive = "$RepoRoot\apps\archive-tool"
Test-FileExists "$archive\Cargo.toml" "archive-tool Cargo.toml"
Test-FileExists "$archive\src\archive.rs" "archive-tool archive module"
Test-FileExists "$archive\src\window.rs" "archive-tool window"
Test-FileContains "$RepoRoot\build\apps-build.sh" "archive-tool" "apps-build includes archive-tool"

Write-Host ""
Write-Host "--- System Monitor (Phase 2) ---"
$sysmon = "$RepoRoot\apps\system-monitor"
Test-FileExists "$sysmon\Cargo.toml" "system-monitor Cargo.toml"
Test-FileExists "$sysmon\src\window.rs" "system-monitor window"
Test-FileExists "$sysmon\src\overview.rs" "system-monitor overview page"
Test-FileExists "$sysmon\src\processes.rs" "system-monitor processes page"
Test-FileExists "$sysmon\src\sysdata.rs" "system-monitor sysdata module"
Test-FileContains "$RepoRoot\build\apps-build.sh" "system-monitor" "apps-build includes system-monitor"

Write-Host ""
Write-Host "--- Quick Settings (Phase 2) ---"
$qs = "$RepoRoot\apps\quick-settings"
Test-FileExists "$qs\Cargo.toml" "quick-settings Cargo.toml"
Test-FileExists "$qs\src\window.rs" "quick-settings window"
Test-FileExists "$qs\src\toggles.rs" "quick-settings toggles module"
Test-FileExists "$qs\src\sliders.rs" "quick-settings sliders module"
Test-FileExists "$qs\src\power.rs" "quick-settings power module"
Test-FileContains "$RepoRoot\build\apps-build.sh" "quick-settings" "apps-build includes quick-settings"

Write-Host ""
Write-Host "--- Notification Center (Phase 2) ---"
$nc = "$RepoRoot\apps\notification-center"
Test-FileExists "$nc\Cargo.toml" "notification-center Cargo.toml"
Test-FileExists "$nc\src\window.rs" "notification-center window"
Test-FileExists "$nc\src\notification.rs" "notification-center notification module"
Test-FileExists "$nc\src\notification_row.rs" "notification-center row widget"
Test-FileContains "$RepoRoot\build\apps-build.sh" "notification-center" "apps-build includes notification-center"

Write-Host ""
Write-Host "--- Hyprland keybindings ---"
$hypr = "$RepoRoot\desktop\hypr\hyprland.conf"
Test-FileContains $hypr "SPACE, exec, rofi" "Super+Space app launcher"
Test-FileContains $hypr "T, exec, foot" "Super+T terminal"
Test-FileContains $hypr "L, exec, hyprlock" "Super+L lock screen"

Write-Host ""
Write-Host "--- Package list ---"
$packages = "$RepoRoot\base\packages.x86_64"
foreach ($pkg in @("hyprland", "waybar", "foot", "firefox", "networkmanager", "greetd", "swww")) {
    Test-FileContains $packages "^${pkg}$" "Package: $pkg"
}

Write-Host ""
if ($Errors -eq 0) {
    Write-Host "==> All validation checks passed." -ForegroundColor Green
    exit 0
} else {
    Write-Host "==> $Errors validation check(s) failed." -ForegroundColor Red
    exit 1
}
