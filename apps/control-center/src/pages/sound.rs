use super::preferences_page;

pub fn sound_page() -> libadwaita::PreferencesPage {
    preferences_page(
        "Sound",
        "Output devices, volume, and per-application audio.",
        &[
            ("Output device", "Default sink (PipeWire)"),
            ("System volume", "Controlled via Waybar"),
            ("Input device", "Default source"),
            ("Per-app volume", "Available in Phase 3"),
        ],
    )
}
