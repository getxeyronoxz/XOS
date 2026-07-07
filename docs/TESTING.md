# Testing XOS

## Config validation

Run before committing config changes:

```bash
./build/validate.sh
```

## ISO build test

1. Build ISO: `./build/build.sh`
2. Verify ISO exists: `ls -la output/`
3. Boot in QEMU: `./build/test-qemu.sh`

## Phase 1 acceptance checklist

- [ ] ISO boots to ReGreet login screen
- [ ] Login starts Hyprland session
- [ ] Waybar shows workspaces, clock, tray
- [ ] Super+Space opens Rofi
- [ ] Super+T opens Foot terminal
- [ ] Super+L shows Hyprlock
- [ ] Mako notifications styled with XOS colors
- [ ] swww wallpaper loads
- [ ] Firefox launches
- [ ] NetworkManager connects to WiFi

## CI

GitHub Actions runs lint and validation on push. Full ISO build runs on `dev` branch pushes (requires self-hosted or larger runner).
