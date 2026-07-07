# XOS Architecture

XOS is built as an Archiso live/install image with a Hyprland desktop stack and native Rust/Python applications.

## Layers

1. **base/** — Archiso profile, package list, airootfs overlay (systemd, greetd, zram)
2. **desktop/** — Hyprland, Waybar, Rofi, Mako, Hyprlock, Foot configs
3. **apps/** — Native XOS applications (Rust GTK4 + Python)
4. **daemons/** — Background services (Phase 3+)
5. **build/** — Docker + mkarchiso pipeline

## Boot flow

```
firmware → systemd-boot → greetd → ReGreet → Hyprland → Waybar/Mako/swww
```

## Version

See `base/airootfs/etc/xos/version` — format `YEAR.MONTH.PATCH` with codename.
