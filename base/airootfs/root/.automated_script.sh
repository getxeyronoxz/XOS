#!/usr/bin/env bash
set -euo pipefail

# Copy configs to skel first
mkdir -p /etc/skel/.config
cp -r /usr/share/xos/desktop-config/.config/* /etc/skel/.config/ 2>/dev/null || true

# Create user now so they inherit skel configurations
if ! id xos &>/dev/null; then
    useradd -m -G wheel,audio,video,optical,storage,network -s /bin/bash xos
    echo "xos:xos" | chpasswd
    echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" >> /etc/sudoers
fi

systemctl enable greetd.service
systemctl enable NetworkManager.service
