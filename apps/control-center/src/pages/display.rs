use super::preferences_page;

pub fn display_page() -> libadwaita::PreferencesPage {
    preferences_page(
        "Displays",
        "Resolution, refresh rate, scaling, and multi-monitor layout.",
        &[
            ("Primary display", "Auto-detected output"),
            ("Resolution", "Preferred mode"),
            ("Refresh rate", "Highest available"),
            ("Scaling", "100% (1x)"),
            ("Multi-monitor", "Extend displays"),
        ],
    )
}
