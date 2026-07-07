use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

/// Creates a WiFi toggle row.
pub fn wifi_row() -> adw::ActionRow {
    toggle_row("Wi-Fi", "network-wireless-symbolic", true)
}

/// Creates a Bluetooth toggle row.
pub fn bluetooth_row() -> adw::ActionRow {
    toggle_row("Bluetooth", "bluetooth-active-symbolic", false)
}

/// Creates a Do Not Disturb toggle row.
pub fn dnd_row() -> adw::ActionRow {
    toggle_row("Do Not Disturb", "notifications-disabled-symbolic", false)
}

/// Creates a Night Light toggle row.
pub fn night_light_row() -> adw::ActionRow {
    toggle_row("Night Light", "night-light-symbolic", false)
}

/// Helper that builds an `adw::ActionRow` with a `gtk4::Switch` suffix.
fn toggle_row(title: &str, icon_name: &str, active: bool) -> adw::ActionRow {
    let switch = gtk4::Switch::new();
    switch.set_active(active);
    switch.set_valign(gtk4::Align::Center);

    let row = adw::ActionRow::builder()
        .title(title)
        .build();
    row.add_prefix(&gtk4::Image::from_icon_name(icon_name));
    row.add_suffix(&switch);
    row.set_activatable_widget(Some(&switch));
    row
}
