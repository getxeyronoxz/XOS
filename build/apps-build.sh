#!/usr/bin/env bash
set -euo pipefail

# Build XOS Rust applications and install into the archiso airootfs overlay.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${REPO_ROOT}/base/airootfs/usr/local/bin"
APPS=(file-manager control-center screenshot notes archive-tool system-monitor quick-settings notification-center automation-builder app-store)

if ! command -v cargo &>/dev/null; then
    echo "WARNING: cargo not found — skipping Rust app build." >&2
    echo "Install rust, cargo, gtk4, and libadwaita to build XOS apps." >&2
    exit 0
fi

mkdir -p "${INSTALL_DIR}"

echo "==> Building XOS Rust applications (release)"
for app in "${APPS[@]}"; do
    app_dir="${REPO_ROOT}/apps/${app}"
    if [[ ! -f "${app_dir}/Cargo.toml" ]]; then
        echo "ERROR: Missing ${app_dir}/Cargo.toml" >&2
        exit 1
    fi

    echo "    -> ${app}"
    cargo build --release --manifest-path "${app_dir}/Cargo.toml"

    package_name="$(grep '^name = ' "${app_dir}/Cargo.toml" | head -n1 | cut -d'"' -f2)"
    binary="${app_dir}/target/release/${package_name}"
    if [[ ! -f "${binary}" ]]; then
        echo "ERROR: Expected binary not found: ${binary}" >&2
        exit 1
    fi

    install -m 755 "${binary}" "${INSTALL_DIR}/${package_name}"
    echo "       installed ${INSTALL_DIR}/${package_name}"
done

# Build XOS Rust daemons (release)
DAEMONS=(resource-daemon permission-daemon update-manager automation-engine)
echo "==> Building XOS Rust daemons (release)"
for daemon in "${DAEMONS[@]}"; do
    daemon_dir="${REPO_ROOT}/daemons/${daemon}"
    if [[ ! -f "${daemon_dir}/Cargo.toml" ]]; then
        echo "ERROR: Missing ${daemon_dir}/Cargo.toml" >&2
        exit 1
    fi

    echo "    -> ${daemon}"
    cargo build --release --manifest-path "${daemon_dir}/Cargo.toml"

    package_name="$(grep '^name = ' "${daemon_dir}/Cargo.toml" | head -n1 | cut -d'"' -f2)"
    binary="${daemon_dir}/target/release/${package_name}"
    if [[ ! -f "${binary}" ]]; then
        echo "ERROR: Expected binary not found: ${binary}" >&2
        exit 1
    fi

    install -m 755 "${binary}" "${INSTALL_DIR}/${package_name}"
    echo "       installed ${INSTALL_DIR}/${package_name}"
done

# Build XOS C++ hardware daemons
HW_DAEMONS=(battery thermal fan-control)
echo "==> Building XOS C++ hardware daemons"
for hw in "${HW_DAEMONS[@]}"; do
    hw_dir="${REPO_ROOT}/hardware/${hw}"
    echo "    -> ${hw}"
    make -C "${hw_dir}" clean
    make -C "${hw_dir}"
    
    binary_name="xos-${hw}"
    if [[ "${hw}" == "fan-control" ]]; then
        binary_name="xos-fan-control"
    else
        binary_name="xos-${hw}-daemon"
    fi
    
    install -m 755 "${hw_dir}/${binary_name}" "${INSTALL_DIR}/${binary_name}"
    echo "       installed ${INSTALL_DIR}/${binary_name}"
done

# Install Python Assistant app
echo "==> Installing Python XOS Assistant"
install -m 755 "${REPO_ROOT}/apps/assistant/main.py" "${INSTALL_DIR}/xos-assistant"
echo "       installed ${INSTALL_DIR}/xos-assistant"

# Install Python Installer app
echo "==> Installing Python XOS Installer"
install -m 755 "${REPO_ROOT}/apps/installer/main.py" "${INSTALL_DIR}/xos-installer"
echo "       installed ${INSTALL_DIR}/xos-installer"

# Install rollback helper
echo "==> Installing Rollback Helper Script"
install -m 755 "${REPO_ROOT}/build/rollback-helper.sh" "${INSTALL_DIR}/xos-rollback"
echo "       installed ${INSTALL_DIR}/xos-rollback"

echo "==> XOS apps and daemons installed to ${INSTALL_DIR}"
