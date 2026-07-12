# XOS — Your Computer. Your Resources. Your Control. 

## That time was unstable and also not guarantee by me.

XOS is an Arch-based, privacy-first workstation operating system tailored for developers, security auditors, researchers, and power users. It enforces full resource ownership, strict app sandboxing, and complete system telemetry transparency.

## Quick Start

Build the Archiso target image inside the WSL2 environment:
```bash
./build/build-iso.sh
```

To compile and mount all custom applications and system control daemons to the build overlay path:
```bash
./build/apps-build.sh
```

To execute the compliance check validation pipeline:
```bash
./build/validate.sh
```

---

## 🛠️ Repository Architecture Map

### Custom Desktop Applications (`/apps`)
- **[file-manager](file:///Y:/XOS/apps/file-manager/)**: Rust/GTK4 explorer with file search, operation handling, and preview panes.
- **[control-center](file:///Y:/XOS/apps/control-center/)**: Central settings panel for custom hardware controls, cgroup allocations, and system options.
- **[system-monitor](file:///Y:/XOS/apps/system-monitor/)**: Real-time load indicators for memory, partition space, and CPU cores.
- **[quick-settings](file:///Y:/XOS/apps/quick-settings/)**: Panel toggles for audio outputs, brightness, network, and power targets.
- **[notification-center](file:///Y:/XOS/apps/notification-center/)**: Persistent notification stack with grouped application streams.
- **[notes](file:///Y:/XOS/apps/notes/)**: Local markdown notebook editor.
- **[screenshot](file:///Y:/XOS/apps/screenshot/)**: Slurp/Grim-backed utility for screen region snapshots.
- **[archive-tool](file:///Y:/XOS/apps/archive-tool/)**: GUI archiver supporting zip/tar formats.
- **[app-store](file:///Y:/XOS/apps/app-store/)**: Extension center for installing preconfigured developer, research, and security packages.
- **[assistant](file:///Y:/XOS/apps/assistant/)**: Conversational assistant for troubleshooting hardware states and memory cleanup.
- **[installer](file:///Y:/XOS/apps/installer/)**: GTK4 installation wizard with disk formatting plans and LUKS2 setup.

### Background System Daemons (`/daemons` & `/hardware`)
- **[battery-daemon (C++)](file:///Y:/XOS/hardware/battery/)**: Hardware battery controller with configurable charge thresholds.
- **[thermal-daemon (C++)](file:///Y:/XOS/hardware/thermal/)**: Real-time temperature tracker and safety throttle triggers.
- **[fan-control (C++)](file:///Y:/XOS/hardware/fan-control/)**: Computes dynamic RPM curves and manual cooling targets.
- **[resource-daemon (Rust)](file:///Y:/XOS/daemons/resource-daemon/)**: Enforces cgroup limits based on resource budgets.
- **[permission-daemon (Rust)](file:///Y:/XOS/daemons/permission-daemon/)**: Hardware-level webcam, microphone, network, and location blocks.
- **[update-manager (Rust)](file:///Y:/XOS/daemons/update-manager/)**: Atomic upgrade coordinator with Btrfs snapshot triggers.
- **[automation-engine (Rust)](file:///Y:/XOS/daemons/automation-engine/)**: background evaluation loop mapping system metrics to action hooks.

---

## 📜 Build & Compliance Verification

The verification pipeline compiles all 21 system binaries and configuration targets, returning a **100% green compliance check** on `/build/validate.sh`.

- Core Theme: Glassmorphic theme layouts styled using standard XOS colors (`#5294E2`, `#1A1A2E`, `#0F3460`).
- Strict Mode: All shell scripts conform to `set -euo pipefail`.
- Atomic Writes: State reporting outputs write atomically via temporary files to avoid reader thread blocks.

## License

Open-source development — see [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md).
