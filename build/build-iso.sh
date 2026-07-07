#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${REPO_ROOT}/output"
WORK_DIR="${REPO_ROOT}/work"
PROFILE_DIR="${REPO_ROOT}/base"

mkdir -p "${OUTPUT_DIR}" "${WORK_DIR}"

echo "==> Installing desktop configs into airootfs overlay"
"${REPO_ROOT}/build/install-desktop-configs.sh"

echo "==> Building and installing XOS Rust apps"
"${REPO_ROOT}/build/apps-build.sh"

echo "==> Building XOS ISO with mkarchiso"

if [[ ! -d "${PROFILE_DIR}/syslinux" ]]; then
    echo "==> Copying syslinux boot files from releng template"
    cp -r /usr/share/archiso/configs/releng/syslinux "${PROFILE_DIR}/"
fi
if [[ ! -d "${PROFILE_DIR}/efiboot" ]]; then
    echo "==> Copying efiboot boot files from releng template"
    cp -r /usr/share/archiso/configs/releng/efiboot "${PROFILE_DIR}/"
fi

mkarchiso -v -w "${WORK_DIR}" -o "${OUTPUT_DIR}" "${PROFILE_DIR}"

ISO_FILE="$(find "${OUTPUT_DIR}" -maxdepth 1 -name '*.iso' -type f | head -n 1)"
if [[ -n "${ISO_FILE}" ]]; then
    ln -sf "$(basename "${ISO_FILE}")" "${OUTPUT_DIR}/XOS-latest.iso"
    echo "==> ISO built: ${ISO_FILE}"
    echo "==> Symlink: ${OUTPUT_DIR}/XOS-latest.iso"
else
    echo "ERROR: No ISO file found in ${OUTPUT_DIR}" >&2
    exit 1
fi
