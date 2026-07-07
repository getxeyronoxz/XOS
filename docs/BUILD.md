# Building XOS

## Prerequisites

- Windows 11 with WSL2 (Arch Linux recommended)
- Docker Desktop (WSL2 backend)
- 8GB RAM allocated to WSL2 (see `.wslconfig` in spec)

### WSL2 packages

```bash
sudo pacman -S base-devel git docker archiso qemu-system-x86_64 imagemagick shellcheck
```

## Build ISO

From the repository root inside WSL2:

```bash
chmod +x build/*.sh
./build/build.sh
```

Output: `output/XOS-<version>.iso` and symlink `output/XOS-latest.iso`

## Test in QEMU

```bash
./build/test-qemu.sh
```

**RAM rule:** Do not run ISO build and QEMU VM simultaneously on 16GB systems.

## Validate configs (without building)

```bash
./build/validate.sh
```

On Windows:

```powershell
pwsh ./build/validate.ps1
```

## Architecture

The build pipeline uses Docker to run `mkarchiso` with the profile in `base/`. Desktop configs from `desktop/` are copied into the airootfs overlay before the ISO is built.
