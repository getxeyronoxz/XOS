use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;

/// Creates a performance mode selector `adw::ComboRow`.
pub fn performance_mode_row() -> adw::ComboRow {
    let model = gtk4::StringList::new(&["Battery Focus", "Balanced", "Performance"]);

    let row = adw::ComboRow::builder()
        .title("Performance Mode")
        .model(&model)
        .selected(1) // default to "Balanced"
        .build();
    row.add_prefix(&gtk4::Image::from_icon_name("power-profile-balanced-symbolic"));
    row
}

/// Creates a horizontal box of session action buttons (Lock, Logout, Restart, Shutdown).
pub fn session_buttons() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    container.set_halign(gtk4::Align::Center);
    container.set_margin_top(8);
    container.set_margin_bottom(8);

    let buttons = [
        ("Lock", "system-lock-screen-symbolic"),
        ("Log Out", "system-log-out-symbolic"),
        ("Restart", "system-reboot-symbolic"),
        ("Shut Down", "system-shutdown-symbolic"),
    ];

    for (tooltip, icon) in buttons {
        let btn = gtk4::Button::from_icon_name(icon);
        btn.set_tooltip_text(Some(tooltip));
        btn.add_css_class("circular");
        btn.add_css_class("flat");
        container.append(&btn);
    }

    container
}
