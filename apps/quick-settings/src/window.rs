use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::power;
use crate::sliders;
use crate::toggles;

pub struct QuickSettingsWindow {
    window: adw::ApplicationWindow,
}

impl QuickSettingsWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Quick Settings")
            .default_width(480)
            .default_height(640)
            .build();

        let header = adw::HeaderBar::new();

        // --- Connectivity section ---
        let connectivity_group = adw::PreferencesGroup::builder()
            .title("Connectivity")
            .build();
        connectivity_group.add(&toggles::wifi_row());
        connectivity_group.add(&toggles::bluetooth_row());

        // --- Display & Sound section ---
        let display_sound_group = adw::PreferencesGroup::builder()
            .title("Display & Sound")
            .build();
        display_sound_group.add(&sliders::brightness_row());
        display_sound_group.add(&sliders::volume_row());
        display_sound_group.add(&toggles::night_light_row());

        // --- Do Not Disturb section ---
        let dnd_group = adw::PreferencesGroup::builder()
            .title("Do Not Disturb")
            .build();
        dnd_group.add(&toggles::dnd_row());

        // --- Performance section ---
        let performance_group = adw::PreferencesGroup::builder()
            .title("Performance")
            .build();
        performance_group.add(&power::performance_mode_row());

        // --- Session section ---
        let session_group = adw::PreferencesGroup::builder()
            .title("Session")
            .build();
        let session_box_row = adw::ActionRow::new();
        session_box_row.set_child(Some(&power::session_buttons()));
        session_group.add(&session_box_row);

        // --- Assemble layout ---
        let content_box = GtkBox::new(Orientation::Vertical, 24);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(24);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        content_box.append(&connectivity_group);
        content_box.append(&display_sound_group);
        content_box.append(&dnd_group);
        content_box.append(&performance_group);
        content_box.append(&session_group);

        let clamp = adw::Clamp::builder()
            .maximum_size(600)
            .child(&content_box)
            .build();

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&clamp)
            .build();

        let outer = GtkBox::new(Orientation::Vertical, 0);
        outer.append(&header);
        outer.append(&scrolled);

        window.set_content(Some(&outer));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}
