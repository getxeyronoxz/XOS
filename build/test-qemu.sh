#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ISO_PATH="${1:-${REPO_ROOT}/output/XOS-latest.iso}"
RAM="${XOS_QEMU_RAM:-4G}"
SMP="${XOS_QEMU_SMP:-2}"

if [[ ! -f "${ISO_PATH}" ]]; then
    echo "ERROR: ISO not found: ${ISO_PATH}" >&2
    echo "Run ./build/build.sh first." >&2
    exit 1
fi

KVM_FLAG=""
if [[ -e /dev/kvm ]]; then
    KVM_FLAG="-enable-kvm"
fi

echo "==> Booting XOS ISO in QEMU: ${ISO_PATH}"
exec qemu-system-x86_64 \
    ${KVM_FLAG} \
    -m "${RAM}" \
    -smp "${SMP}" \
    -cdrom "${ISO_PATH}" \
    -boot d \
    -vga virtio \
    -display gtk
