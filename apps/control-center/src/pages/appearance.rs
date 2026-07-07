use super::preferences_page;

pub fn appearance_page() -> libadwaita::PreferencesPage {
    preferences_page(
        "Appearance",
        "Theme, accent, fonts, icons, cursor, and wallpaper.",
        &[
            ("Theme", "Dark (default). Light theme optional."),
            ("Accent color", "#5294E2"),
            ("UI font", "Inter"),
            ("Terminal font", "JetBrains Mono"),
            ("Icon theme", "Papirus-Dark"),
            ("Cursor", "Bibata Modern Dark"),
            ("Wallpaper", "Managed by swww"),
        ],
    )
}
