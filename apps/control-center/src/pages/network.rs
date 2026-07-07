use super::preferences_page;

pub fn network_page() -> libadwaita::PreferencesPage {
    preferences_page(
        "Network",
        "WiFi, Ethernet, VPN, proxy, and firewall settings.",
        &[
            ("WiFi", "Managed by NetworkManager"),
            ("Ethernet", "Automatic connection"),
            ("VPN", "Configure in Phase 3"),
            ("Proxy", "System proxy settings"),
            ("Firewall", "ufw enabled (deny in, allow out)"),
        ],
    )
}
