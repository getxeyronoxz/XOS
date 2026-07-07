use gtk4::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use chrono::{DateTime, Local};

use crate::notification::Notification;

pub fn build_notification_row<F: Fn() + 'static>(
    notification: &Notification,
    on_dismiss: F,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(&notification.summary)
        .subtitle(&notification.body)
        .build();

    // App name prefix label
    let app_label = gtk4::Label::builder()
        .label(&notification.app_name)
        .valign(gtk4::Align::Center)
        .build();
    app_label.add_css_class("caption");
    app_label.add_css_class("dim-label");
    row.add_prefix(&app_label);

    // Relative time suffix label
    let time_str = format_relative_time(notification.timestamp);
    let time_label = gtk4::Label::builder()
        .label(&time_str)
        .valign(gtk4::Align::Center)
        .build();
    time_label.add_css_class("caption");
    time_label.add_css_class("dim-label");
    row.add_suffix(&time_label);

    // Dismiss button
    let dismiss_btn = gtk4::Button::builder()
        .icon_name("window-close-symbolic")
        .has_frame(false)
        .valign(gtk4::Align::Center)
        .build();
    dismiss_btn.set_tooltip_text(Some("Dismiss"));
    dismiss_btn.connect_clicked(move |_| {
        on_dismiss();
    });
    row.add_suffix(&dismiss_btn);

    row
}

fn format_relative_time(timestamp: DateTime<Local>) -> String {
    let now = Local::now();
    let diff = now.signed_duration_since(timestamp);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}
