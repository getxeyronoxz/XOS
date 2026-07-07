use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

/// Creates a brightness slider row (0–100).
pub fn brightness_row() -> adw::ActionRow {
    slider_row("Brightness", "display-brightness-symbolic", 75.0)
}

/// Creates a volume slider row (0–100).
pub fn volume_row() -> adw::ActionRow {
    slider_row("Volume", "audio-volume-high-symbolic", 50.0)
}

/// Helper that builds an `adw::ActionRow` with a `gtk4::Scale` suffix.
fn slider_row(title: &str, icon_name: &str, initial: f64) -> adw::ActionRow {
    let adjustment = gtk4::Adjustment::new(initial, 0.0, 100.0, 1.0, 10.0, 0.0);

    let scale = gtk4::Scale::new(gtk4::Orientation::Horizontal, Some(&adjustment));
    scale.set_hexpand(true);
    scale.set_size_request(200, -1);
    scale.set_valign(gtk4::Align::Center);
    scale.set_draw_value(false);

    let row = adw::ActionRow::builder()
        .title(title)
        .build();
    row.add_prefix(&gtk4::Image::from_icon_name(icon_name));
    row.add_suffix(&scale);
    row
}
